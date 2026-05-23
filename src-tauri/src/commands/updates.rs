//! Opt-in latest-release check against the public GitHub Releases API.
//!
//! Behavior:
//! - Only invoked when the user opts in via Settings (`check_updates_on_launch`).
//! - Single `GET /repos/<owner>/<repo>/releases/latest`. Stops there.
//! - No download, no install, no telemetry. The frontend opens the release
//!   page in the user's browser if they choose to update.
//! - Pre-releases and drafts are ignored (the `/releases/latest` endpoint
//!   already excludes them).

use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const REPO_OWNER: &str = "tsechis";
pub const REPO_NAME: &str = "druhy-nazor";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LatestRelease {
    pub current_version: String,
    pub latest_version: String,
    pub is_newer: bool,
    pub html_url: String,
    pub published_at: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    published_at: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

#[tauri::command]
pub async fn check_latest_release() -> AppResult<LatestRelease> {
    fetch_latest(&format!(
        "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest"
    ))
    .await
}

async fn fetch_latest(url: &str) -> AppResult<LatestRelease> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("druhy-nazor/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| AppError::Other(format!("reqwest builder: {error}")))?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|error| AppError::Other(format!("github http: {error}")))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| AppError::Other(format!("github body: {error}")))?;

    if !status.is_success() {
        return Err(AppError::Other(format!("github {status}: {body}")));
    }

    parse_release(&body)
}

fn parse_release(body: &str) -> AppResult<LatestRelease> {
    let release: GhRelease = serde_json::from_str(body)?;
    if release.draft || release.prerelease {
        return Err(AppError::NotFound("no stable release".into()));
    }

    let latest = release.tag_name.trim_start_matches('v').to_string();
    let is_newer = version_gt(&latest, APP_VERSION);

    Ok(LatestRelease {
        current_version: APP_VERSION.to_string(),
        latest_version: latest,
        is_newer,
        html_url: release.html_url,
        published_at: release.published_at,
        body: release.body,
    })
}

/// Tolerant semver comparison: returns true if `a > b`. Missing minor/patch
/// segments are treated as zero. Pre-release tags (`-rc.1` etc.) are stripped.
fn version_gt(a: &str, b: &str) -> bool {
    parse_version(a) > parse_version(b)
}

fn parse_version(value: &str) -> (u32, u32, u32) {
    let core = value.split('-').next().unwrap_or("");
    let mut parts = core.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gt_basic() {
        assert!(version_gt("0.2.0", "0.1.0"));
        assert!(version_gt("0.1.1", "0.1.0"));
        assert!(version_gt("1.0.0", "0.99.99"));
        assert!(!version_gt("0.1.0", "0.1.0"));
        assert!(!version_gt("0.1.0", "0.2.0"));
    }

    #[test]
    fn version_gt_missing_parts() {
        assert!(version_gt("0.2", "0.1"));
        assert!(version_gt("1", "0.99.99"));
        assert!(!version_gt("0.1", "0.1.0"));
    }

    #[test]
    fn version_gt_strips_pre_release_suffix() {
        assert!(!version_gt("0.1.0-rc.1", "0.1.0"));
        assert!(version_gt("0.2.0-beta", "0.1.0"));
    }

    #[test]
    fn version_gt_handles_garbage_segments() {
        assert!(!version_gt("not.a.version", "0.0.0"));
        assert!(version_gt("0.1.0", "garbage"));
    }

    #[test]
    fn parse_release_strips_v_prefix() {
        let body = r#"{
            "tag_name": "v0.99.0",
            "html_url": "https://github.com/x/y/releases/tag/v0.99.0",
            "published_at": "2026-05-21T12:00:00Z",
            "body": "Bug fixes",
            "draft": false,
            "prerelease": false
        }"#;
        let release = parse_release(body).unwrap();
        assert_eq!(release.latest_version, "0.99.0");
        assert!(release.is_newer);
        assert_eq!(release.body, "Bug fixes");
    }

    #[test]
    fn parse_release_handles_missing_v_prefix() {
        let body = r#"{
            "tag_name": "0.5.0",
            "html_url": "https://github.com/x/y/releases/tag/0.5.0",
            "published_at": "",
            "body": ""
        }"#;
        let release = parse_release(body).unwrap();
        assert_eq!(release.latest_version, "0.5.0");
    }

    #[test]
    fn parse_release_rejects_prerelease() {
        let body = r#"{
            "tag_name": "v1.0.0",
            "html_url": "https://x",
            "prerelease": true
        }"#;
        let err = parse_release(body).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn parse_release_rejects_draft() {
        let body = r#"{
            "tag_name": "v1.0.0",
            "html_url": "https://x",
            "draft": true
        }"#;
        assert!(parse_release(body).is_err());
    }

    #[test]
    fn parse_release_reports_current_equals_latest() {
        let body = format!(r#"{{ "tag_name": "v{APP_VERSION}", "html_url": "https://x" }}"#);
        let release = parse_release(&body).unwrap();
        assert_eq!(release.current_version, APP_VERSION);
        assert_eq!(release.latest_version, APP_VERSION);
        assert!(!release.is_newer);
    }

    #[test]
    fn parse_release_rejects_invalid_json() {
        assert!(parse_release("not json").is_err());
    }
}
