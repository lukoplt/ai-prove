use crate::error::{AppError, AppResult};
use reqwest::{redirect, Client};
use std::io::Cursor;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use url::{Host, Url};

const MAX_REDIRECTS: usize = 10;

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BODY_BYTES: usize = 600_000;
pub const MAX_EXCERPT_CHARS: usize = 3_000;

pub struct Extractor {
    client: Client,
}

impl Extractor {
    pub fn new() -> AppResult<Self> {
        // The initial-host check in `fetch_and_extract` only guards the first
        // request. reqwest follows redirects by default, so a public page could
        // 3xx to a loopback/private host and bypass that check (SSRF). Re-run the
        // internal-host guard (and scheme guard) on every redirect hop.
        let redirect_policy = redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error(SsrfBlocked::TooManyRedirects);
            }
            let url = attempt.url();
            if !matches!(url.scheme(), "http" | "https") {
                return attempt.error(SsrfBlocked::BadScheme);
            }
            match url.host() {
                Some(host) if host_is_internal(&host) => attempt.error(SsrfBlocked::InternalHost),
                _ => attempt.follow(),
            }
        });

        let client = Client::builder()
            .timeout(FETCH_TIMEOUT)
            .redirect(redirect_policy)
            .user_agent(concat!("prove/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| AppError::Other(format!("reqwest builder: {error}")))?;

        Ok(Self { client })
    }

    pub async fn fetch_and_extract(&self, url_str: &str) -> AppResult<String> {
        let url =
            Url::parse(url_str).map_err(|error| AppError::Invalid(format!("bad url: {error}")))?;

        // Defense in depth: only ever fetch over http(s). Source URLs come from
        // the search provider, but never let a stray file://, ftp://, or custom
        // scheme reach the HTTP client.
        if !matches!(url.scheme(), "http" | "https") {
            return Err(AppError::Invalid(format!(
                "refusing non-http(s) url scheme: {}",
                url.scheme()
            )));
        }

        // Defense in depth: source URLs come from the search provider, which is
        // semi-trusted. Refuse hosts that point back at the local machine or a
        // private network (SSRF). Legitimate fact-check sources are on the public
        // web. Note: literal-IP and obvious-localhost cases only; a hostname that
        // resolves to a private IP via DNS rebinding is not covered here.
        if let Some(host) = url.host() {
            if host_is_internal(&host) {
                return Err(AppError::Invalid(format!(
                    "refusing internal/private host: {host}"
                )));
            }
        }

        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| AppError::Other(format!("fetch {url_str}: {error}")))?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::Other(format!("fetch {url_str}: {status}")));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|error| AppError::Other(format!("read body {url_str}: {error}")))?;
        let truncated = if bytes.len() > MAX_BODY_BYTES {
            &bytes[..MAX_BODY_BYTES]
        } else {
            &bytes[..]
        };
        let html = String::from_utf8_lossy(truncated);
        let mut cursor = Cursor::new(html.as_bytes());

        let extracted = readability::extractor::extract(&mut cursor, &url)
            .map_err(|error| AppError::Other(format!("readability: {error:?}")))?;
        let text = strip_to_text(&extracted.content);

        Ok(truncate_chars(&text, MAX_EXCERPT_CHARS))
    }
}

/// Reasons a redirect hop is refused. Surfaced through reqwest's redirect
/// policy so the failure shows up as a normal fetch error to the caller.
#[derive(Debug)]
enum SsrfBlocked {
    InternalHost,
    BadScheme,
    TooManyRedirects,
}

impl std::fmt::Display for SsrfBlocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalHost => write!(f, "refusing redirect to internal/private host"),
            Self::BadScheme => write!(f, "refusing redirect to non-http(s) scheme"),
            Self::TooManyRedirects => write!(f, "too many redirects"),
        }
    }
}

impl std::error::Error for SsrfBlocked {}

/// True for hosts that must never be fetched: loopback, private, link-local,
/// and unspecified addresses, plus `localhost`/`*.local` hostnames.
fn host_is_internal(host: &Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => ipv4_is_internal(*ip),
        Host::Ipv6(ip) => ipv6_is_internal(*ip),
        Host::Domain(name) => {
            let lower = name.to_ascii_lowercase();
            let last_label = lower.rsplit('.').next().unwrap_or("");
            lower == "localhost" || lower.ends_with(".localhost") || last_label == "local"
        }
    }
}

fn ipv4_is_internal(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        // Carrier-grade NAT 100.64.0.0/10 (not yet stable in std).
        || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
}

fn ipv6_is_internal(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return true;
    }
    // Unique local addresses fc00::/7 (is_unique_local() is unstable in std).
    if (ip.segments()[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    // Link-local fe80::/10.
    if (ip.segments()[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // IPv4-mapped/compatible: re-check the embedded v4 address.
    if let Some(v4) = ip.to_ipv4() {
        return ipv4_is_internal(v4);
    }
    false
}

fn strip_to_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => output.push(c),
            _ => {}
        }
    }

    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }

    input.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_to_text_removes_tags_and_collapses_whitespace() {
        let html = "<p>Hello <strong>world</strong>!\n\n   Multiple   spaces.</p>";

        assert_eq!(strip_to_text(html), "Hello world! Multiple spaces.");
    }

    #[test]
    fn truncate_chars_respects_unicode() {
        assert_eq!(truncate_chars("ěščřžýáíé", 3), "ěšč");
    }

    #[test]
    fn truncate_chars_passthrough_when_short() {
        assert_eq!(truncate_chars("hi", 100), "hi");
    }

    #[tokio::test]
    async fn rejects_internal_and_private_hosts() {
        let extractor = Extractor::new().unwrap();
        for url in [
            "http://localhost/x",
            "http://LocalHost:8080/admin",
            "http://router.local/",
            "http://127.0.0.1/",
            "http://10.0.0.5/",
            "http://192.168.1.1/",
            "http://172.16.0.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://100.64.0.1/",
            "http://0.0.0.0/",
            "http://[::1]/",
            "http://[fe80::1]/",
            "http://[fc00::1]/",
        ] {
            let error = extractor.fetch_and_extract(url).await.unwrap_err();
            assert!(
                error.to_string().contains("internal/private host"),
                "expected internal-host rejection for {url}, got: {error}"
            );
        }
    }

    #[test]
    fn public_hosts_are_not_internal() {
        for url in [
            "https://example.com/",
            "https://8.8.8.8/",
            "https://en.wikipedia.org/wiki/Rust",
        ] {
            let parsed = Url::parse(url).unwrap();
            assert!(
                !host_is_internal(&parsed.host().unwrap()),
                "public host wrongly flagged internal: {url}"
            );
        }
    }

    #[tokio::test]
    async fn rejects_non_http_schemes() {
        let extractor = Extractor::new().unwrap();
        for url in [
            "file:///etc/passwd",
            "ftp://example.com/x",
            "data:text/html,x",
        ] {
            let error = extractor.fetch_and_extract(url).await.unwrap_err();
            assert!(
                error.to_string().contains("non-http(s) url scheme"),
                "expected scheme rejection for {url}, got: {error}"
            );
        }
    }
}
