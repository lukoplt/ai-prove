use crate::error::{AppError, AppResult};
use reqwest::Client;
use std::io::Cursor;
use std::time::Duration;
use url::Url;

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BODY_BYTES: usize = 600_000;
pub const MAX_EXCERPT_CHARS: usize = 3_000;

pub struct Extractor {
    client: Client,
}

impl Extractor {
    pub fn new() -> AppResult<Self> {
        let client = Client::builder()
            .timeout(FETCH_TIMEOUT)
            .user_agent(concat!("prove/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| AppError::Other(format!("reqwest builder: {error}")))?;

        Ok(Self { client })
    }

    pub async fn fetch_and_extract(&self, url_str: &str) -> AppResult<String> {
        let url =
            Url::parse(url_str).map_err(|error| AppError::Invalid(format!("bad url: {error}")))?;
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
}
