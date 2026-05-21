use super::{SearchProvider, SearchResult};
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";

pub struct BraveClient {
    client: Client,
    api_key: String,
    locale: String,
}

impl BraveClient {
    pub fn new(api_key: String, locale: String) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("druhy-nazor/0.1")
            .build()
            .map_err(|error| AppError::Other(format!("reqwest builder: {error}")))?;

        Ok(Self {
            client,
            api_key,
            locale,
        })
    }

    fn country_and_lang(&self) -> (&'static str, &'static str) {
        match self.locale.as_str() {
            "cs" => ("cz", "cs"),
            _ => ("us", "en"),
        }
    }
}

#[async_trait]
impl SearchProvider for BraveClient {
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchResult>> {
        let count = limit.clamp(1, 20);
        let (country, search_lang) = self.country_and_lang();
        let response = self
            .client
            .get(ENDPOINT)
            .query(&[
                ("q", query),
                ("count", &count.to_string()),
                ("country", country),
                ("search_lang", search_lang),
                ("safesearch", "moderate"),
                ("spellcheck", "0"),
            ])
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|error| AppError::Other(format!("brave http: {error}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| AppError::Other(format!("brave body: {error}")))?;

        if !status.is_success() {
            return Err(AppError::Other(format!("brave {status}: {body}")));
        }

        parse_brave_response(&body)
    }
}

#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<WebSection>,
}

#[derive(Debug, Deserialize)]
struct WebSection {
    results: Vec<RawResult>,
}

#[derive(Debug, Deserialize)]
struct RawResult {
    url: String,
    title: String,
    description: String,
}

fn parse_brave_response(body: &str) -> AppResult<Vec<SearchResult>> {
    let parsed: BraveResponse = serde_json::from_str(body)?;
    let raw = parsed.web.map_or_else(Vec::new, |web| web.results);

    Ok(raw
        .into_iter()
        .map(|result| SearchResult {
            url: result.url,
            title: result.title,
            snippet: strip_tags(&result.description),
        })
        .collect())
}

fn strip_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => output.push(c),
            _ => {}
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_brave_response() {
        let body = r#"{
            "web": {
                "results": [
                    {"url": "https://cs.wikipedia.org/wiki/Karel_IV", "title": "Karel IV.", "description": "Karel IV. se <strong>narodil v roce 1316</strong> v Praze."},
                    {"url": "https://example.com", "title": "Example", "description": "Other."}
                ]
            }
        }"#;

        let results = parse_brave_response(body).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].url, "https://cs.wikipedia.org/wiki/Karel_IV");
        assert!(!results[0].snippet.contains('<'));
        assert!(results[0].snippet.contains("1316"));
    }

    #[test]
    fn handles_empty_web_section() {
        let results = parse_brave_response(r"{}").unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_brave_response("not json").is_err());
    }
}
