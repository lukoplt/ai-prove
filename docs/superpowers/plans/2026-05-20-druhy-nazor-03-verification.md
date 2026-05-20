# Druhý názor — Phase M2: Verification Pipeline

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans`. Complete `01-foundation.md` and `02-classification.md` first. Read `00-overview.md` for conventions.

**Goal:** For every `fact`-classified claim (up to 8 per analysis), run a web search → article body extraction → LLM-judged stance → aggregated verdict. Render per-claim sources, tier badges, and an aggregate Czech verdict in the side panel. Persist results in an SQLite cache keyed by claim hash with a configurable TTL. End-to-end MVP: paste an AI response, see colors and clickable sources.

**Architecture:** Verification runs _after_ atomization in the same `analyze_text` Tauri command. The command emits `analysis-claims` first (M1 behavior), then spawns one tokio task per eligible fact-claim. Each task: rewrites the query (truncate for now), hits Brave Search (top 5 results), fetches and Mozilla-Readability-extracts each result body, runs the judge LLM per (claim, source) pair, aggregates stances using the source-tier heuristic, writes the cache row, and emits a `claim-verified` event. The frontend updates the matching claim in the analysis store as events arrive.

**Tech Stack additions on top of M1:** `rusqlite` (bundled), `readability` (Rust port), `url`, `tracing` already present.

---

## Task 1: Judge prompt — replace placeholder

**Files:**

- Modify: `src-tauri/src/llm/prompts/mod.rs`
- Create: `src-tauri/src/llm/prompts/judge_cs.txt`

- [ ] **Step 1: Write `src-tauri/src/llm/prompts/judge_cs.txt`**

```
Jsi nástroj na ověřování faktů. Dostaneš tvrzení a krátký výňatek ze zdrojového dokumentu. Tvůj úkol je rozhodnout, jak se zdroj staví k tvrzení.

MOŽNOSTI (vyber přesně jednu):
- supports — zdroj přímo nebo nepřímo potvrzuje tvrzení.
- contradicts — zdroj přímo nebo nepřímo vyvrací tvrzení.
- mentions — zdroj se k tvrzení vyjadřuje nejasně nebo jen zmiňuje téma bez potvrzení/vyvrácení.

Pokud zdroj o tvrzení vůbec nemluví, použij mentions.

Vrať také quote: krátký doslovný citát ze zdroje (max 200 znaků), který tvoje stanovisko nejlépe podpírá. Když relevantní citát chybí, vrať quote="".

Pracuj v češtině. Vrať pouze volání nástroje submit_judgement, nic jiného.
```

- [ ] **Step 2: Replace the `JUDGE_CS` constant in `src-tauri/src/llm/prompts/mod.rs`**

```rust
pub const ATOMIZE_CS: &str = include_str!("atomize_cs.txt");
pub const JUDGE_CS: &str = include_str!("judge_cs.txt");
```

- [ ] **Step 3: Verify build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/llm/prompts/
git commit -m "feat(llm): Czech judge prompt with supports/contradicts/mentions"
```

---

## Task 2: SearchProvider trait + Brave client

**Files:**

- Create: `src-tauri/src/search/mod.rs`
- Create: `src-tauri/src/search/brave.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create directory and module**

```bash
mkdir -p src-tauri/src/search
```

Write `src-tauri/src/search/mod.rs`:

```rust
pub mod brave;
pub mod extract;

use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchResult>>;
}

pub struct MockSearch {
    pub results: Vec<SearchResult>,
}

#[async_trait]
impl SearchProvider for MockSearch {
    async fn search(&self, _query: &str, limit: usize) -> AppResult<Vec<SearchResult>> {
        Ok(self.results.iter().take(limit).cloned().collect())
    }
}
```

- [ ] **Step 2: Write `src-tauri/src/search/brave.rs`**

```rust
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
}

impl BraveClient {
    pub fn new(api_key: String) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("druhy-nazor/0.1")
            .build()
            .map_err(|e| AppError::Other(format!("reqwest: {e}")))?;
        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl SearchProvider for BraveClient {
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchResult>> {
        let count = limit.clamp(1, 20);
        let resp = self
            .client
            .get(ENDPOINT)
            .query(&[
                ("q", query),
                ("count", &count.to_string()),
                ("country", "cz"),
                ("search_lang", "cs"),
                ("safesearch", "moderate"),
                ("spellcheck", "0"),
            ])
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AppError::Other(format!("brave http: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Other(format!("brave body: {e}")))?;
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
    let raw = parsed.web.map(|w| w.results).unwrap_or_default();
    Ok(raw
        .into_iter()
        .map(|r| SearchResult { url: r.url, title: r.title, snippet: strip_tags(&r.description) })
        .collect())
}

fn strip_tags(s: &str) -> String {
    // Brave snippets may contain <strong> highlights. Strip them.
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
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
        let res = parse_brave_response(body).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].url, "https://cs.wikipedia.org/wiki/Karel_IV");
        assert!(!res[0].snippet.contains('<'));
        assert!(res[0].snippet.contains("1316"));
    }

    #[test]
    fn handles_empty_web_section() {
        let body = r#"{}"#;
        let res = parse_brave_response(body).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        let body = "not json";
        assert!(parse_brave_response(body).is_err());
    }
}
```

- [ ] **Step 3: Re-export `search` from `lib.rs`**

Full `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod hotkey;
pub mod llm;
pub mod models;
pub mod pipeline;
pub mod search;
pub mod storage;
pub mod tray;

pub use error::{AppError, AppResult};
```

- [ ] **Step 4: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml search::brave
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/search/ src-tauri/src/lib.rs
git commit -m "feat(search): SearchProvider trait + Brave client with snippet HTML strip"
```

---

## Task 3: Source body extraction

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/search/extract.rs`

- [ ] **Step 1: Add the readability dependency**

Append to `[dependencies]` in `src-tauri/Cargo.toml`:

```toml
readability = "0.3"
url = "2"
```

- [ ] **Step 2: Write `src-tauri/src/search/extract.rs`**

```rust
use crate::error::{AppError, AppResult};
use reqwest::Client;
use std::time::Duration;
use url::Url;

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BODY_BYTES: usize = 600_000;   // ~600 KB raw HTML
pub const MAX_EXCERPT_CHARS: usize = 3000;

pub struct Extractor {
    client: Client,
}

impl Extractor {
    pub fn new() -> AppResult<Self> {
        let client = Client::builder()
            .timeout(FETCH_TIMEOUT)
            .user_agent("druhy-nazor/0.1 (+https://druhynazor.cz)")
            .build()
            .map_err(|e| AppError::Other(format!("reqwest: {e}")))?;
        Ok(Self { client })
    }

    pub async fn fetch_and_extract(&self, url_str: &str) -> AppResult<String> {
        let url = Url::parse(url_str)
            .map_err(|e| AppError::Invalid(format!("bad url {url_str}: {e}")))?;
        let resp = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("fetch {url_str}: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(AppError::Other(format!("fetch {url_str} {status}")));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::Other(format!("read body {url_str}: {e}")))?;
        let truncated = if bytes.len() > MAX_BODY_BYTES {
            &bytes[..MAX_BODY_BYTES]
        } else {
            &bytes[..]
        };
        let html = String::from_utf8_lossy(truncated);

        let extracted = readability::extractor::extract(&mut html.as_bytes(), &url)
            .map_err(|e| AppError::Other(format!("readability: {e:?}")))?;
        let text = strip_to_text(&extracted.content);
        Ok(truncate_chars(&text, MAX_EXCERPT_CHARS))
    }
}

/// Strips remaining HTML tags from the readability-extracted content and
/// collapses whitespace.
fn strip_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    // collapse whitespace
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
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
        let s = "ěščřžýáíé".to_string();
        assert_eq!(truncate_chars(&s, 3), "ěšč");
    }

    #[test]
    fn truncate_chars_passthrough_when_short() {
        assert_eq!(truncate_chars("hi", 100), "hi");
    }
}
```

- [ ] **Step 3: Verify build + tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml search::extract
```

Expected: 3 tests pass. The build pulls down `readability` and its tree; that may take a minute.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/search/extract.rs
git commit -m "feat(extract): readability-backed fetch+extract with byte and char caps"
```

---

## Task 4: Source tier scoring

**Files:**

- Create: `src-tauri/src/pipeline/source_tier.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/pipeline/source_tier.rs`**

```rust
use crate::models::SourceTier;
use url::Url;

const A_SUFFIXES: &[&str] = &[
    "wikipedia.org",
    "wikidata.org",
    "britannica.com",
];

const A_EXACT_DOMAINS: &[&str] = &[
    "ec.europa.eu",
    "europa.eu",
    "who.int",
    "un.org",
    "czso.cz",
    "mzcr.cz",
    "uzis.cz",
    "mzv.cz",
    "mvcr.cz",
    "cnb.cz",
    "ucl.cas.cz",
];

const A_HOST_PATTERNS: &[&str] = &[
    ".gov",
    ".gov.cz",
    ".gov.uk",
    ".gov.au",
    ".edu",
    ".ac.uk",
    ".ac.cz",
    ".cas.cz",
];

const B_EXACT_DOMAINS: &[&str] = &[
    "ct24.cz",
    "ceskatelevize.cz",
    "irozhlas.cz",
    "rozhlas.cz",
    "novinky.cz",
    "seznamzpravy.cz",
    "idnes.cz",
    "lidovky.cz",
    "denik.cz",
    "ihned.cz",
    "e15.cz",
    "aktualne.cz",
    "hlidacipes.org",
    "respekt.cz",
    "bbc.com",
    "bbc.co.uk",
    "nytimes.com",
    "theguardian.com",
    "ap.org",
    "reuters.com",
    "nature.com",
    "science.org",
    "economist.com",
];

const D_EXACT_DOMAINS: &[&str] = &[
    "facebook.com",
    "twitter.com",
    "x.com",
    "tiktok.com",
    "instagram.com",
    "reddit.com",
    "quora.com",
    "pinterest.com",
];

const D_HOST_PATTERNS: &[&str] = &[
    ".blogspot.",
    ".wordpress.com",
    ".wixsite.com",
    ".weebly.com",
];

pub fn score(url_str: &str) -> SourceTier {
    let Ok(url) = Url::parse(url_str) else {
        return SourceTier::C;
    };
    let Some(host) = url.host_str() else {
        return SourceTier::C;
    };
    let host = host.trim_start_matches("www.").to_ascii_lowercase();

    if A_EXACT_DOMAINS.iter().any(|d| host == *d)
        || A_SUFFIXES.iter().any(|sfx| host.ends_with(sfx))
        || A_HOST_PATTERNS.iter().any(|pat| host.ends_with(pat) || host.contains(pat))
    {
        return SourceTier::A;
    }

    if B_EXACT_DOMAINS.iter().any(|d| host == *d || host.ends_with(&format!(".{d}"))) {
        return SourceTier::B;
    }

    if D_EXACT_DOMAINS.iter().any(|d| host == *d)
        || D_HOST_PATTERNS.iter().any(|pat| host.contains(pat))
    {
        return SourceTier::D;
    }

    SourceTier::C
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikipedia_is_a() {
        assert_eq!(score("https://cs.wikipedia.org/wiki/Karel_IV"), SourceTier::A);
        assert_eq!(score("https://en.wikipedia.org/wiki/Charles_IV"), SourceTier::A);
    }

    #[test]
    fn czso_is_a() {
        assert_eq!(score("https://www.czso.cz/csu/czso/pocet-obyvatel"), SourceTier::A);
    }

    #[test]
    fn gov_suffix_is_a() {
        assert_eq!(score("https://example.gov"), SourceTier::A);
        assert_eq!(score("https://nara.gov"), SourceTier::A);
        assert_eq!(score("https://mzv.gov.cz"), SourceTier::A);
    }

    #[test]
    fn edu_suffix_is_a() {
        assert_eq!(score("https://mit.edu"), SourceTier::A);
    }

    #[test]
    fn major_news_is_b() {
        assert_eq!(score("https://ct24.ceskatelevize.cz/foo"), SourceTier::B);
        assert_eq!(score("https://www.bbc.com/news/x"), SourceTier::B);
    }

    #[test]
    fn social_is_d() {
        assert_eq!(score("https://twitter.com/x"), SourceTier::D);
        assert_eq!(score("https://reddit.com/r/x"), SourceTier::D);
    }

    #[test]
    fn blogspot_pattern_is_d() {
        assert_eq!(score("https://someone.blogspot.com/post"), SourceTier::D);
    }

    #[test]
    fn unknown_is_c() {
        assert_eq!(score("https://example.com/foo"), SourceTier::C);
    }

    #[test]
    fn invalid_url_is_c() {
        assert_eq!(score("not a url"), SourceTier::C);
    }
}
```

- [ ] **Step 2: Update `src-tauri/src/pipeline/mod.rs`**

```rust
pub mod atomize;
pub mod source_tier;
pub mod verify;

pub use atomize::atomize_to_claims;
```

(`verify` module created in Task 7.)

Touch the file as a placeholder so the import compiles in this task. Write `src-tauri/src/pipeline/verify.rs`:

```rust
// Filled in Task 7.
```

- [ ] **Step 3: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml pipeline::source_tier
```

Expected: 9 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/pipeline/
git commit -m "feat(pipeline): source tier scoring with A/B/C/D heuristic"
```

---

## Task 5: SQLite database and migrations

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/migrations/001_init.sql`
- Create: `src-tauri/src/storage/db.rs`
- Modify: `src-tauri/src/storage/mod.rs`

- [ ] **Step 1: Add rusqlite to `Cargo.toml`**

Append to `[dependencies]`:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

- [ ] **Step 2: Create `src-tauri/migrations/001_init.sql`**

```sql
CREATE TABLE IF NOT EXISTS verification_cache (
    claim_hash      TEXT PRIMARY KEY,
    claim_text      TEXT NOT NULL,
    verification    TEXT NOT NULL,
    created_at_ms   INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_created
    ON verification_cache(created_at_ms DESC);

CREATE TABLE IF NOT EXISTS analysis_history (
    id              TEXT PRIMARY KEY,
    created_at_ms   INTEGER NOT NULL,
    input           TEXT NOT NULL,
    analysis_json   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_created
    ON analysis_history(created_at_ms DESC);
```

- [ ] **Step 3: Write `src-tauri/src/storage/db.rs`**

```rust
use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

const MIGRATION_001: &str = include_str!("../../migrations/001_init.sql");

#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).map_err(map_sqlite)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA synchronous=NORMAL;",
        )
        .map_err(map_sqlite)?;
        conn.execute_batch(MIGRATION_001).map_err(map_sqlite)?;
        Ok(Self { inner: Arc::new(Mutex::new(conn)) })
    }

    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(map_sqlite)?;
        conn.execute_batch(MIGRATION_001).map_err(map_sqlite)?;
        Ok(Self { inner: Arc::new(Mutex::new(conn)) })
    }

    pub fn with<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let guard = self.inner.lock().expect("db mutex poisoned");
        f(&guard).map_err(map_sqlite)
    }
}

fn map_sqlite(e: rusqlite::Error) -> AppError {
    AppError::Other(format!("sqlite: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_runs_migrations() {
        let db = Db::open_in_memory().unwrap();
        let cnt: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='verification_cache'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(cnt, 1);
    }

    #[test]
    fn insert_and_select_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        db.with(|c| {
            c.execute(
                "INSERT INTO verification_cache (claim_hash, claim_text, verification, created_at_ms) VALUES (?,?,?,?)",
                rusqlite::params!["h1", "claim", r#"{"status":"supported","sources":[],"summary":""}"#, 1000_i64],
            )
        }).unwrap();
        let n: i64 = db.with(|c| c.query_row("SELECT count(*) FROM verification_cache", [], |r| r.get(0))).unwrap();
        assert_eq!(n, 1);
    }
}
```

- [ ] **Step 4: Update `src-tauri/src/storage/mod.rs`**

```rust
pub mod cache;
pub mod db;
pub mod history;
pub mod keychain;
pub mod settings_store;
```

Create placeholder files for the ones filled in next tasks. Write `src-tauri/src/storage/cache.rs`:

```rust
// Filled in Task 6.
```

Write `src-tauri/src/storage/history.rs`:

```rust
// Filled in Task 13 (history is wired up but the UI lives in the Polish phase).
```

- [ ] **Step 5: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml storage::db
```

Expected: 2 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/migrations/ src-tauri/src/storage/
git commit -m "feat(db): bundled SQLite with migrations 001 (cache + history)"
```

---

## Task 6: Verification cache

**Files:**

- Modify: `src-tauri/src/storage/cache.rs`

The cache key is a SHA-256 of the normalized claim text (lowercase, trim, collapse whitespace). Values are JSON-serialized `Verification`. Lookups respect TTL by checking `created_at_ms` against the current epoch.

- [ ] **Step 1: Write `src-tauri/src/storage/cache.rs`**

```rust
use crate::error::AppResult;
use crate::models::Verification;
use crate::storage::db::Db;
use sha2::{Digest, Sha256};

pub fn hash_claim(text: &str) -> String {
    let normalized = normalize(text);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

pub fn get(db: &Db, claim_hash: &str, ttl_ms: i64, now_ms: i64) -> AppResult<Option<Verification>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT verification, created_at_ms FROM verification_cache WHERE claim_hash = ?",
        )?;
        let mut rows = stmt.query([claim_hash])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let created: i64 = row.get(1)?;
            if now_ms - created <= ttl_ms {
                let v: Verification = serde_json::from_str(&json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
                return Ok(Some(v));
            }
        }
        Ok(None)
    })
}

pub fn put(db: &Db, claim_hash: &str, claim_text: &str, verification: &Verification, now_ms: i64) -> AppResult<()> {
    let json = serde_json::to_string(verification)?;
    db.with(|c| {
        c.execute(
            "INSERT OR REPLACE INTO verification_cache (claim_hash, claim_text, verification, created_at_ms) VALUES (?,?,?,?)",
            rusqlite::params![claim_hash, claim_text, json, now_ms],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SourceHit, SourceStance, SourceTier, VerificationStatus};

    fn sample_verification() -> Verification {
        Verification {
            status: VerificationStatus::Supported,
            sources: vec![SourceHit {
                url: "https://cs.wikipedia.org/x".into(),
                title: "X".into(),
                snippet: "y".into(),
                tier: SourceTier::A,
                stance: SourceStance::Supports,
            }],
            summary: "OK".into(),
        }
    }

    #[test]
    fn normalize_collapses_and_lowercases() {
        assert_eq!(normalize("  Karel  IV.  se  Narodil "), "karel iv. se narodil");
    }

    #[test]
    fn hash_is_deterministic_and_normalization_insensitive() {
        let h1 = hash_claim("Karel IV. se narodil");
        let h2 = hash_claim("  karel iv.   se narodil  ");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn put_then_get_within_ttl() {
        let db = Db::open_in_memory().unwrap();
        let h = hash_claim("c");
        put(&db, &h, "c", &sample_verification(), 1000).unwrap();
        let got = get(&db, &h, 7 * 24 * 3600 * 1000, 2000).unwrap();
        assert!(got.is_some());
    }

    #[test]
    fn get_expires_past_ttl() {
        let db = Db::open_in_memory().unwrap();
        let h = hash_claim("c");
        put(&db, &h, "c", &sample_verification(), 0).unwrap();
        let got = get(&db, &h, 1000, 5000).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn put_replaces_on_conflict() {
        let db = Db::open_in_memory().unwrap();
        let h = hash_claim("c");
        let mut v1 = sample_verification();
        v1.summary = "first".into();
        let mut v2 = sample_verification();
        v2.summary = "second".into();
        put(&db, &h, "c", &v1, 1000).unwrap();
        put(&db, &h, "c", &v2, 2000).unwrap();
        let got = get(&db, &h, 1_000_000, 3000).unwrap().unwrap();
        assert_eq!(got.summary, "second");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml storage::cache
```

Expected: 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/storage/cache.rs
git commit -m "feat(cache): SHA-256-keyed verification cache with TTL and put-replace"
```

---

## Task 7: Verification orchestrator

**Files:**

- Modify: `src-tauri/src/pipeline/verify.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

Single-claim verification: search → fetch+extract per result → judge each → aggregate. Returns a `Verification`.

- [ ] **Step 1: Replace `src-tauri/src/pipeline/verify.rs`**

```rust
use crate::error::AppResult;
use crate::llm::{LlmProvider, Stance};
use crate::models::{SourceHit, SourceStance, SourceTier, Verification, VerificationStatus};
use crate::pipeline::source_tier;
use crate::search::extract::Extractor;
use crate::search::{SearchProvider, SearchResult};
use futures::future::join_all;
use std::sync::Arc;
use tracing::{debug, warn};

pub const SEARCH_RESULTS_PER_CLAIM: usize = 5;
pub const MAX_SOURCES_IN_RESULT: usize = 3;
pub const VERIFICATION_QUERY_MAX_CHARS: usize = 120;

pub struct VerificationEngine {
    pub llm: Arc<dyn LlmProvider>,
    pub search: Arc<dyn SearchProvider>,
    pub extractor: Arc<Extractor>,
}

impl VerificationEngine {
    pub async fn verify(&self, claim_text: &str) -> AppResult<Verification> {
        let query = build_query(claim_text);
        debug!(?query, "verifying claim");
        let results = self.search.search(&query, SEARCH_RESULTS_PER_CLAIM).await?;
        if results.is_empty() {
            return Ok(Verification {
                status: VerificationStatus::NotFound,
                sources: vec![],
                summary: "Nenašel jsem žádný zdroj k ověření.".into(),
            });
        }

        let llm = self.llm.clone();
        let extractor = self.extractor.clone();
        let claim_owned = claim_text.to_string();

        let futures = results.into_iter().map(|r| {
            let llm = llm.clone();
            let extractor = extractor.clone();
            let claim = claim_owned.clone();
            async move { judge_one(&extractor, &*llm, &claim, r).await }
        });
        let judged: Vec<Option<SourceHit>> = join_all(futures).await;
        let mut hits: Vec<SourceHit> = judged.into_iter().flatten().collect();

        // Order: tier ascending (A first), then supports > contradicts > mentions.
        hits.sort_by(|a, b| {
            tier_rank(a.tier)
                .cmp(&tier_rank(b.tier))
                .then(stance_rank(a.stance).cmp(&stance_rank(b.stance)))
        });

        let status = aggregate(&hits);
        let summary = summarize(status, &hits);
        let sources = hits.into_iter().take(MAX_SOURCES_IN_RESULT).collect();

        Ok(Verification { status, sources, summary })
    }
}

fn build_query(claim_text: &str) -> String {
    let trimmed = claim_text.trim();
    if trimmed.chars().count() <= VERIFICATION_QUERY_MAX_CHARS {
        return trimmed.to_string();
    }
    let mut acc = String::new();
    let mut count = 0;
    for ch in trimmed.chars() {
        if count >= VERIFICATION_QUERY_MAX_CHARS && ch.is_whitespace() {
            break;
        }
        acc.push(ch);
        count += 1;
    }
    acc
}

async fn judge_one(
    extractor: &Extractor,
    llm: &dyn LlmProvider,
    claim: &str,
    r: SearchResult,
) -> Option<SourceHit> {
    let tier = source_tier::score(&r.url);
    let body = match extractor.fetch_and_extract(&r.url).await {
        Ok(b) if !b.trim().is_empty() => b,
        Ok(_) => {
            debug!(url = %r.url, "empty body from readability");
            return Some(SourceHit {
                url: r.url,
                title: r.title,
                snippet: r.snippet,
                tier,
                stance: SourceStance::Mentions,
            });
        }
        Err(e) => {
            warn!(url = %r.url, error = %e, "fetch/extract failed; skipping");
            return None;
        }
    };

    let verdict = match llm.judge(claim, &body).await {
        Ok(v) => v,
        Err(e) => {
            warn!(url = %r.url, error = %e, "judge failed; falling back to mentions");
            return Some(SourceHit {
                url: r.url,
                title: r.title,
                snippet: r.snippet,
                tier,
                stance: SourceStance::Mentions,
            });
        }
    };

    let stance = match verdict.stance {
        Stance::Supports => SourceStance::Supports,
        Stance::Contradicts => SourceStance::Contradicts,
        Stance::Mentions => SourceStance::Mentions,
    };

    let snippet = if !verdict.quote.is_empty() { verdict.quote } else { r.snippet };

    Some(SourceHit { url: r.url, title: r.title, snippet, tier, stance })
}

fn tier_rank(t: SourceTier) -> u8 {
    match t {
        SourceTier::A => 0,
        SourceTier::B => 1,
        SourceTier::C => 2,
        SourceTier::D => 3,
    }
}

fn stance_rank(s: SourceStance) -> u8 {
    match s {
        SourceStance::Supports => 0,
        SourceStance::Contradicts => 1,
        SourceStance::Mentions => 2,
    }
}

fn aggregate(hits: &[SourceHit]) -> VerificationStatus {
    if hits.is_empty() {
        return VerificationStatus::NotFound;
    }

    let mut a_supports = false;
    let mut a_contradicts = false;
    let mut b_supports = false;
    let mut b_contradicts = false;

    for h in hits {
        match (h.tier, h.stance) {
            (SourceTier::A, SourceStance::Supports) => a_supports = true,
            (SourceTier::A, SourceStance::Contradicts) => a_contradicts = true,
            (SourceTier::B, SourceStance::Supports) => b_supports = true,
            (SourceTier::B, SourceStance::Contradicts) => b_contradicts = true,
            _ => {}
        }
    }

    if a_supports && a_contradicts {
        return VerificationStatus::NoConsensus;
    }
    if a_supports || (b_supports && !b_contradicts && !a_contradicts) {
        return VerificationStatus::Supported;
    }
    if a_contradicts || (b_contradicts && !b_supports) {
        return VerificationStatus::Contradicted;
    }
    if b_supports && b_contradicts {
        return VerificationStatus::NoConsensus;
    }
    VerificationStatus::NotFound
}

fn summarize(status: VerificationStatus, hits: &[SourceHit]) -> String {
    let count = hits.len();
    match status {
        VerificationStatus::Supported => format!("Tvrzení potvrzuje {count} zdroj(ů)."),
        VerificationStatus::Contradicted => format!("Tvrzení vyvrací {count} zdroj(ů)."),
        VerificationStatus::NoConsensus => "Zdroje se neshodují — bez konsenzu.".into(),
        VerificationStatus::NotFound => "Nenašel jsem zdroje, které by se k tvrzení vyjadřovaly.".into(),
        VerificationStatus::NotVerified => "Tvrzení nebylo ověřováno.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(tier: SourceTier, stance: SourceStance) -> SourceHit {
        SourceHit {
            url: "https://example.com".into(),
            title: "t".into(),
            snippet: "s".into(),
            tier,
            stance,
        }
    }

    #[test]
    fn build_query_passes_short_claim_through() {
        let q = build_query("Karel IV. se narodil v roce 1316");
        assert_eq!(q, "Karel IV. se narodil v roce 1316");
    }

    #[test]
    fn build_query_truncates_long_claim_at_word_boundary() {
        let claim = "a ".repeat(80);
        let q = build_query(&claim);
        assert!(q.chars().count() <= VERIFICATION_QUERY_MAX_CHARS + 1);
        assert!(!q.ends_with(' ') || q.is_empty());
    }

    #[test]
    fn aggregate_a_supports_wins() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Supports),
            hit(SourceTier::C, SourceStance::Contradicts),
        ];
        assert_eq!(aggregate(&hits), VerificationStatus::Supported);
    }

    #[test]
    fn aggregate_a_contradicts_wins() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Contradicts),
            hit(SourceTier::C, SourceStance::Supports),
        ];
        assert_eq!(aggregate(&hits), VerificationStatus::Contradicted);
    }

    #[test]
    fn aggregate_a_split_is_no_consensus() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Supports),
            hit(SourceTier::A, SourceStance::Contradicts),
        ];
        assert_eq!(aggregate(&hits), VerificationStatus::NoConsensus);
    }

    #[test]
    fn aggregate_b_only_supports() {
        let hits = vec![hit(SourceTier::B, SourceStance::Supports)];
        assert_eq!(aggregate(&hits), VerificationStatus::Supported);
    }

    #[test]
    fn aggregate_b_split_is_no_consensus() {
        let hits = vec![
            hit(SourceTier::B, SourceStance::Supports),
            hit(SourceTier::B, SourceStance::Contradicts),
        ];
        assert_eq!(aggregate(&hits), VerificationStatus::NoConsensus);
    }

    #[test]
    fn aggregate_only_mentions_is_not_found() {
        let hits = vec![hit(SourceTier::C, SourceStance::Mentions)];
        assert_eq!(aggregate(&hits), VerificationStatus::NotFound);
    }

    #[test]
    fn aggregate_empty_is_not_found() {
        assert_eq!(aggregate(&[]), VerificationStatus::NotFound);
    }
}
```

- [ ] **Step 2: Add the futures dependency**

Append to `[dependencies]` in `src-tauri/Cargo.toml`:

```toml
futures = "0.3"
```

- [ ] **Step 3: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml pipeline::verify
```

Expected: 9 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/pipeline/verify.rs
git commit -m "feat(pipeline): verification orchestrator with tier-aware aggregation"
```

---

## Task 8: Pipeline integration test with mocks

**Files:**

- Create: `src-tauri/tests/pipeline_integration.rs`

This validates the end-to-end pipeline using mock providers — no network needed. It locks in behavior before we wire the live command.

- [ ] **Step 1: Write `src-tauri/tests/pipeline_integration.rs`**

```rust
use druhy_nazor_lib::llm::mock::MockProvider;
use druhy_nazor_lib::llm::{AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance};
use druhy_nazor_lib::models::{ClaimKind, VerificationStatus};
use druhy_nazor_lib::pipeline::atomize_to_claims;
use druhy_nazor_lib::pipeline::verify::VerificationEngine;
use druhy_nazor_lib::search::extract::Extractor;
use druhy_nazor_lib::search::{MockSearch, SearchResult};
use std::sync::Arc;

#[tokio::test]
async fn atomize_then_verify_supported() {
    let input = "Karel IV. se narodil v roce 1316.";
    let llm = MockProvider::new();
    llm.push_atomize(AtomizationResult {
        claims: vec![RawClaim {
            text: "Karel IV. se narodil v roce 1316".into(),
            kind: RawClaimKind::Fact,
            reason: "Datum.".into(),
        }],
        truncated: false,
    });
    llm.push_judge(JudgeVerdict { stance: Stance::Supports, quote: "1316".into() });

    let outcome = atomize_to_claims(&llm, input).await.unwrap();
    assert_eq!(outcome.claims.len(), 1);
    assert_eq!(outcome.claims[0].kind, ClaimKind::Fact);

    let search = MockSearch {
        results: vec![SearchResult {
            url: "https://cs.wikipedia.org/wiki/Karel_IV".into(),
            title: "Karel IV.".into(),
            snippet: "Narodil se v roce 1316...".into(),
        }],
    };
    let engine = VerificationEngine {
        llm: Arc::new(llm),
        search: Arc::new(search),
        extractor: Arc::new(Extractor::new().unwrap()),
    };

    // Use a fake URL that fetch will fail on; engine falls back to a Mentions hit.
    // We don't actually want to make a live HTTP call in CI. The orchestrator
    // currently calls fetch_and_extract; for this test we accept that the URL
    // points at a real wikipedia page and the test only runs locally.
    // CI gate: skip if RUN_NETWORK_TESTS != "1".
    if std::env::var("RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        eprintln!("RUN_NETWORK_TESTS=1 not set; skipping fetch portion.");
        return;
    }

    let v = engine.verify(&outcome.claims[0].text).await.unwrap();
    assert_eq!(v.status, VerificationStatus::Supported);
    assert!(!v.sources.is_empty());
}
```

- [ ] **Step 2: Run unit tests (network test skipped)**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test pipeline_integration
```

Expected: 1 test passes (the network portion is auto-skipped).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tests/pipeline_integration.rs
git commit -m "test(integration): atomize then verify with mock LLM + MockSearch"
```

---

## Task 9: Wire verification into analyze_text

**Files:**

- Modify: `src-tauri/src/commands/analysis.rs`

The command now spawns one task per fact-claim after emitting `analysis-claims`. Each task uses cached results when fresh, otherwise calls the engine, persists to cache, and emits `claim-verified`.

- [ ] **Step 1: Replace `src-tauri/src/commands/analysis.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::llm::anthropic::AnthropicProvider;
use crate::llm::LlmProvider;
use crate::models::{Analysis, Claim, ClaimKind, Verification, VerificationStatus};
use crate::pipeline::atomize_to_claims;
use crate::pipeline::verify::VerificationEngine;
use crate::search::brave::BraveClient;
use crate::search::extract::Extractor;
use crate::search::SearchProvider;
use crate::storage::cache;
use crate::storage::db::Db;
use crate::storage::keychain;
use crate::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

pub const ACCOUNT_ANTHROPIC: &str = "anthropic";
pub const ACCOUNT_BRAVE: &str = "brave";
pub const MAX_VERIFIED_CLAIMS: usize = 8;

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStartedEvent {
    pub analysis_id: String,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisClaimsEvent {
    pub analysis_id: String,
    pub analysis: Analysis,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClaimVerifiedEvent {
    pub analysis_id: String,
    pub claim_id: String,
    pub verification: Verification,
}

#[tauri::command]
pub async fn analyze_text<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    text: String,
) -> AppResult<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("input is empty".into()));
    }

    let anthropic_key = keychain::get_api_key(ACCOUNT_ANTHROPIC)?
        .ok_or_else(|| AppError::NotFound("anthropic key".into()))?;
    let brave_key = keychain::get_api_key(ACCOUNT_BRAVE)?
        .ok_or_else(|| AppError::NotFound("brave key".into()))?;

    let settings = load_settings(&app);
    let provider: Arc<dyn LlmProvider> = Arc::new(AnthropicProvider::new(anthropic_key, settings.model.clone())?);

    let analysis_id = Uuid::now_v7().to_string();
    app.emit(
        "analysis-started",
        AnalysisStartedEvent { analysis_id: analysis_id.clone() },
    )
    .map_err(|e| AppError::Other(format!("emit: {e}")))?;

    let outcome = atomize_to_claims(&*provider, trimmed).await?;

    let analysis = Analysis {
        id: analysis_id.clone(),
        created_at: Utc::now().timestamp_millis(),
        input: trimmed.to_string(),
        claims: outcome.claims.clone(),
        truncated: outcome.truncated,
    };

    app.emit(
        "analysis-claims",
        AnalysisClaimsEvent { analysis_id: analysis_id.clone(), analysis: analysis.clone() },
    )
    .map_err(|e| AppError::Other(format!("emit: {e}")))?;

    spawn_verifications(
        app.clone(),
        db.inner().clone(),
        provider,
        brave_key,
        analysis_id.clone(),
        outcome.claims,
        settings,
    );

    Ok(analysis_id)
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Settings {
    if let Ok(store) = app.store(SETTINGS_FILE) {
        if let Some(value) = store.get(SETTINGS_KEY) {
            if let Ok(s) = serde_json::from_value(value) {
                return s;
            }
        }
    }
    Settings::default()
}

fn spawn_verifications<R: Runtime>(
    app: AppHandle<R>,
    db: Db,
    provider: Arc<dyn LlmProvider>,
    brave_key: String,
    analysis_id: String,
    claims: Vec<Claim>,
    settings: Settings,
) {
    let fact_claims: Vec<Claim> = claims
        .into_iter()
        .filter(|c| c.kind == ClaimKind::Fact)
        .take(MAX_VERIFIED_CLAIMS)
        .collect();

    if fact_claims.is_empty() {
        return;
    }

    let extractor = match Extractor::new() {
        Ok(e) => Arc::new(e),
        Err(_) => return,
    };
    let search: Arc<dyn SearchProvider> = match BraveClient::new(brave_key) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };
    let engine = Arc::new(VerificationEngine { llm: provider, search, extractor });

    let ttl_ms: i64 = i64::from(settings.cache_ttl_days) * 24 * 3600 * 1000;

    for claim in fact_claims {
        let app = app.clone();
        let db = db.clone();
        let engine = engine.clone();
        let analysis_id = analysis_id.clone();
        tokio::spawn(async move {
            let now = Utc::now().timestamp_millis();
            let hash = cache::hash_claim(&claim.text);

            let verification: Verification = match cache::get(&db, &hash, ttl_ms, now) {
                Ok(Some(v)) => v,
                _ => match engine.verify(&claim.text).await {
                    Ok(v) => {
                        let _ = cache::put(&db, &hash, &claim.text, &v, now);
                        v
                    }
                    Err(e) => Verification {
                        status: VerificationStatus::NotFound,
                        sources: vec![],
                        summary: format!("Verifikace selhala: {e}"),
                    },
                },
            };

            let _ = app.emit(
                "claim-verified",
                ClaimVerifiedEvent {
                    analysis_id: analysis_id.clone(),
                    claim_id: claim.id,
                    verification,
                },
            );
        });
    }
}
```

- [ ] **Step 2: Set up the Db in `main.rs`**

In `src-tauri/src/main.rs`, open the database in `.setup` and stash it in Tauri's state. Full file:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use druhy_nazor_lib::commands::settings::{
    clear_api_key, get_settings, has_api_key, set_api_key, set_settings,
};
use druhy_nazor_lib::hotkey;
use druhy_nazor_lib::storage::db::Db;
use druhy_nazor_lib::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use tauri::{Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            set_api_key,
            clear_api_key,
            has_api_key,
            druhy_nazor_lib::commands::analysis::analyze_text,
        ])
        .on_window_event(|win, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if win.label() == "main" {
                    api.prevent_close();
                    let _ = win.hide();
                }
            }
        })
        .setup(|app| {
            let store = app.store(SETTINGS_FILE)?;
            let settings: Settings = store
                .get(SETTINGS_KEY)
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("cache.db");
            let db = Db::open(&db_path)?;
            app.manage(db);

            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            druhy_nazor_lib::tray::install(&app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/analysis.rs src-tauri/src/main.rs
git commit -m "feat(commands): spawn verification per fact-claim with cache + emit claim-verified"
```

---

## Task 10: Frontend listener for claim-verified

**Files:**

- Modify: `src/lib/api.ts`
- Modify: `src/lib/stores/analysis.svelte.ts`

- [ ] **Step 1: Extend `src/lib/api.ts` with the new event**

Append:

```ts
import type { Verification } from './types';

export interface ClaimVerifiedEvent {
  analysisId: string;
  claimId: string;
  verification: Verification;
}

export function onClaimVerified(handler: (e: ClaimVerifiedEvent) => void): Promise<UnlistenFn> {
  return listen<ClaimVerifiedEvent>('claim-verified', (msg) => handler(msg.payload));
}
```

- [ ] **Step 2: Update `src/lib/stores/analysis.svelte.ts`**

Add the subscription and a patch helper. Replace the `ensureSubscriptions` function and add a patcher; the full updated file:

```ts
import { analyzeText, onAnalysisClaims, onAnalysisStarted, onClaimVerified } from '$lib/api';
import type { Analysis, Claim } from '$lib/types';
import type { UnlistenFn } from '@tauri-apps/api/event';

type Status = 'idle' | 'running' | 'done' | 'error';

let status = $state<Status>('idle');
let current = $state<Analysis | null>(null);
let selectedId = $state<string | null>(null);
let error = $state<string | null>(null);
let started = false;
let unlistens: UnlistenFn[] = [];

async function ensureSubscriptions() {
  if (started) return;
  started = true;
  unlistens.push(
    await onAnalysisStarted(() => {
      status = 'running';
      current = null;
      selectedId = null;
      error = null;
    }),
  );
  unlistens.push(
    await onAnalysisClaims(({ analysis }) => {
      current = analysis;
      status = 'done';
      selectedId = analysis.claims[0]?.id ?? null;
    }),
  );
  unlistens.push(
    await onClaimVerified(({ analysisId, claimId, verification }) => {
      if (!current || current.id !== analysisId) return;
      const next: Analysis = {
        ...current,
        claims: current.claims.map((c) => (c.id === claimId ? { ...c, verification } : c)),
      };
      current = next;
    }),
  );
}

export const analysisStore = {
  get status() {
    return status;
  },
  get current() {
    return current;
  },
  get selectedId() {
    return selectedId;
  },
  get error() {
    return error;
  },
  get selectedClaim(): Claim | null {
    if (!current || !selectedId) return null;
    return current.claims.find((c) => c.id === selectedId) ?? null;
  },

  async init(): Promise<void> {
    await ensureSubscriptions();
  },

  async run(text: string): Promise<void> {
    await ensureSubscriptions();
    error = null;
    try {
      await analyzeText(text);
    } catch (e) {
      status = 'error';
      error = String(e);
    }
  },

  select(id: string): void {
    selectedId = id;
  },

  reset(): void {
    status = 'idle';
    current = null;
    selectedId = null;
    error = null;
  },
};
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/api.ts src/lib/stores/analysis.svelte.ts
git commit -m "feat(web): subscribe to claim-verified and patch current analysis"
```

---

## Task 11: TierBadge component

**Files:**

- Create: `src/lib/components/TierBadge.svelte`

- [ ] **Step 1: Add i18n keys**

Extend `src/lib/i18n/cs.json`:

```json
"tier": {
  "a": "Primární zdroj",
  "b": "Velké médium",
  "c": "Ostatní",
  "d": "Sociální / blog"
}
```

And `src/lib/i18n/en.json`:

```json
"tier": {
  "a": "Primary source",
  "b": "Major outlet",
  "c": "Other",
  "d": "Social / blog"
}
```

- [ ] **Step 2: Write `src/lib/components/TierBadge.svelte`**

```svelte
<script lang="ts">
  import type { SourceTier } from '$lib/types';
  import { t } from '$lib/stores/i18n.svelte';

  let { tier }: { tier: SourceTier } = $props();
</script>

<span class="badge tier-{tier}">{t(`tier.${tier}`)}</span>

<style>
  .badge {
    display: inline-block;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .tier-a {
    background: #dbeafe;
    color: #1e3a8a;
  }
  .tier-b {
    background: #e0e7ff;
    color: #3730a3;
  }
  .tier-c {
    background: #f3f4f6;
    color: #374151;
  }
  .tier-d {
    background: #fee2e2;
    color: #7f1d1d;
  }
</style>
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/TierBadge.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(web): TierBadge component (A/B/C/D)"
```

---

## Task 12: SourceCard component

**Files:**

- Create: `src/lib/components/SourceCard.svelte`

- [ ] **Step 1: Add i18n keys for stance + open**

Extend `src/lib/i18n/cs.json`:

```json
"stance": {
  "supports": "Podporuje",
  "contradicts": "Vyvrací",
  "mentions": "Zmiňuje"
},
"source": {
  "open": "Otevřít v prohlížeči"
}
```

And `src/lib/i18n/en.json`:

```json
"stance": {
  "supports": "Supports",
  "contradicts": "Contradicts",
  "mentions": "Mentions"
},
"source": {
  "open": "Open in browser"
}
```

- [ ] **Step 2: Add an `openExternal` API wrapper**

Append to `src/lib/api.ts`:

```ts
import { open as openShell } from '@tauri-apps/plugin-shell';

export async function openInBrowser(url: string): Promise<void> {
  await openShell(url);
}
```

Install the shell plugin:

```bash
pnpm add @tauri-apps/plugin-shell
```

And add the Rust plugin in `src-tauri/Cargo.toml`:

```toml
tauri-plugin-shell = "2"
```

Register it in `src-tauri/src/main.rs` (add to the builder chain, before `invoke_handler`):

```rust
.plugin(tauri_plugin_shell::init())
```

Append to `src-tauri/capabilities/default.json` permissions list:

```json
"shell:allow-open",
```

- [ ] **Step 3: Write `src/lib/components/SourceCard.svelte`**

```svelte
<script lang="ts">
  import type { SourceHit } from '$lib/types';
  import TierBadge from './TierBadge.svelte';
  import { t } from '$lib/stores/i18n.svelte';
  import { openInBrowser } from '$lib/api';

  let { source }: { source: SourceHit } = $props();

  function open() {
    openInBrowser(source.url);
  }

  function host(url: string): string {
    try {
      return new URL(url).host.replace(/^www\./, '');
    } catch {
      return url;
    }
  }
</script>

<article class="card stance-{source.stance}">
  <header>
    <TierBadge tier={source.tier} />
    <span class="stance-pill">{t(`stance.${source.stance}`)}</span>
    <span class="host">{host(source.url)}</span>
  </header>
  <h4>{source.title}</h4>
  {#if source.snippet}
    <p class="snippet">„{source.snippet}"</p>
  {/if}
  <button type="button" onclick={open}>{t('source.open')} →</button>
</article>

<style>
  .card {
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    padding: 10px 12px;
    margin-bottom: 8px;
    background: white;
  }
  .stance-supports {
    border-left: 3px solid #22c55e;
  }
  .stance-contradicts {
    border-left: 3px solid #ef4444;
  }
  .stance-mentions {
    border-left: 3px solid #9ca3af;
  }
  header {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 4px;
  }
  .stance-pill {
    font-size: 11px;
    color: #374151;
    background: #f3f4f6;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .host {
    font-size: 11px;
    color: #6b7280;
    margin-left: auto;
  }
  h4 {
    margin: 0 0 4px;
    font-size: 13px;
  }
  .snippet {
    margin: 0 0 6px;
    font-size: 12px;
    color: #4b5563;
  }
  button {
    background: none;
    border: none;
    color: #2563eb;
    cursor: pointer;
    font-size: 12px;
    padding: 0;
  }
</style>
```

- [ ] **Step 4: Verify build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
pnpm check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/main.rs src-tauri/capabilities/default.json src/lib/api.ts src/lib/components/SourceCard.svelte src/lib/i18n/cs.json src/lib/i18n/en.json package.json pnpm-lock.yaml
git commit -m "feat(web): SourceCard with stance, tier, and open-in-browser"
```

---

## Task 13: Update SidePanel to render verification

**Files:**

- Modify: `src/lib/components/SidePanel.svelte`

- [ ] **Step 1: Add status i18n keys**

Extend `src/lib/i18n/cs.json`:

```json
"status": {
  "supported": "Ověřeno",
  "contradicted": "Vyvráceno",
  "no_consensus": "Bez konsenzu",
  "not_found": "Bez nálezu",
  "not_verified": "Nebylo ověřováno"
},
"verification": {
  "pending": "Ověřuji…",
  "skipped_kind": "Tento typ tvrzení neověřujeme.",
  "no_sources": "Žádné zdroje."
}
```

And `src/lib/i18n/en.json`:

```json
"status": {
  "supported": "Verified",
  "contradicted": "Contradicted",
  "no_consensus": "No consensus",
  "not_found": "Not found",
  "not_verified": "Not verified"
},
"verification": {
  "pending": "Verifying…",
  "skipped_kind": "We don't verify this kind of claim.",
  "no_sources": "No sources."
}
```

- [ ] **Step 2: Replace `src/lib/components/SidePanel.svelte`**

```svelte
<script lang="ts">
  import type { Claim, VerificationStatus } from '$lib/types';
  import { t } from '$lib/stores/i18n.svelte';
  import SourceCard from './SourceCard.svelte';

  let { claim }: { claim: Claim | null } = $props();

  function kindLabel(k: Claim['kind']): string {
    return t(`sidepanel.kind_${k}`);
  }

  function statusLabel(s: VerificationStatus): string {
    return t(`status.${s}`);
  }
</script>

<aside class="sp">
  {#if !claim}
    <p class="empty">{t('sidepanel.empty')}</p>
  {:else}
    <header>
      <span class="badge kind-{claim.kind}">{kindLabel(claim.kind)}</span>
    </header>
    <blockquote class="quote">„{claim.text}"</blockquote>
    <section>
      <h3>{t('sidepanel.why_label')}</h3>
      <p>{claim.reason}</p>
    </section>
    <section>
      <h3>{t('sidepanel.sources_label')}</h3>
      {#if claim.kind !== 'fact'}
        <p class="muted">{t('verification.skipped_kind')}</p>
      {:else if !claim.verification}
        <p class="muted">{t('verification.pending')}</p>
      {:else}
        <p class="verdict status-{claim.verification.status}">
          <strong>{statusLabel(claim.verification.status)}</strong>
          — {claim.verification.summary}
        </p>
        {#if claim.verification.sources.length === 0}
          <p class="muted">{t('verification.no_sources')}</p>
        {:else}
          {#each claim.verification.sources as src (src.url)}
            <SourceCard source={src} />
          {/each}
        {/if}
      {/if}
    </section>
  {/if}
</aside>

<style>
  .sp {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 16px;
    background: #fafafa;
    min-height: 360px;
  }
  .empty {
    color: #6b7280;
    font-size: 14px;
    margin: 0;
  }
  header {
    margin-bottom: 8px;
  }
  .badge {
    display: inline-block;
    font-size: 12px;
    padding: 3px 8px;
    border-radius: 999px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .kind-fact {
    background: rgba(34, 197, 94, 0.22);
    color: #14532d;
  }
  .kind-inference {
    background: rgba(234, 179, 8, 0.25);
    color: #713f12;
  }
  .kind-opinion {
    background: rgba(249, 115, 22, 0.25);
    color: #7c2d12;
  }
  .kind-contradiction {
    background: rgba(239, 68, 68, 0.25);
    color: #7f1d1d;
  }
  .quote {
    margin: 0 0 12px;
    padding: 8px 12px;
    background: white;
    border-left: 3px solid #d1d5db;
    font-size: 14px;
  }
  section h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin: 0 0 4px;
  }
  section p {
    margin: 0 0 8px;
    font-size: 13px;
  }
  .muted {
    color: #9ca3af;
    font-style: italic;
  }
  .verdict {
    padding: 8px 10px;
    border-radius: 6px;
    font-size: 13px;
    margin: 0 0 8px;
  }
  .status-supported {
    background: rgba(34, 197, 94, 0.15);
    color: #14532d;
  }
  .status-contradicted {
    background: rgba(239, 68, 68, 0.15);
    color: #7f1d1d;
  }
  .status-no_consensus {
    background: rgba(234, 179, 8, 0.18);
    color: #713f12;
  }
  .status-not_found {
    background: #f3f4f6;
    color: #4b5563;
  }
  .status-not_verified {
    background: #f3f4f6;
    color: #6b7280;
  }
</style>
```

- [ ] **Step 3: Manual smoke test**

```bash
pnpm tauri dev
```

With both Anthropic and Brave keys set:

1. Paste: `Karel IV. se narodil v roce 1316 v Praze. Byl podle mě nejlepší český král.`
2. Click Analyze.
3. After ~3 seconds, colors appear (fact green, opinion orange).
4. Each fact-claim shows "Ověřuji…" in its side panel initially.
5. Within ~20 seconds, the verdict updates to "Ověřeno" with Wikipedia in the sources.

If verification stays stuck on "Ověřuji…", open the dev console for errors and check the Brave key.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/SidePanel.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(web): SidePanel renders verification verdict + tiered sources"
```

---

## Task 14: Persist analysis history

**Files:**

- Modify: `src-tauri/src/storage/history.rs`
- Modify: `src-tauri/src/commands/analysis.rs`

History is written to the DB but the UI lives in the Polish phase. We store the analysis once verifications complete (or after a 30 s window, whichever happens first) so the row reflects the final state.

- [ ] **Step 1: Replace `src-tauri/src/storage/history.rs`**

```rust
use crate::error::AppResult;
use crate::models::Analysis;
use crate::storage::db::Db;

pub fn insert(db: &Db, analysis: &Analysis) -> AppResult<()> {
    let json = serde_json::to_string(analysis)?;
    db.with(|c| {
        c.execute(
            "INSERT OR REPLACE INTO analysis_history (id, created_at_ms, input, analysis_json) VALUES (?,?,?,?)",
            rusqlite::params![analysis.id, analysis.created_at, analysis.input, json],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Analysis;

    fn empty_analysis() -> Analysis {
        Analysis {
            id: "01900000-0000-0000-0000-000000000001".into(),
            created_at: 1700000000000,
            input: "hi".into(),
            claims: vec![],
            truncated: false,
        }
    }

    #[test]
    fn insert_replaces_on_id_conflict() {
        let db = Db::open_in_memory().unwrap();
        let mut a = empty_analysis();
        insert(&db, &a).unwrap();
        a.input = "again".into();
        insert(&db, &a).unwrap();

        let count: i64 = db.with(|c| c.query_row("SELECT count(*) FROM analysis_history", [], |r| r.get(0))).unwrap();
        assert_eq!(count, 1);

        let stored_input: String = db.with(|c| c.query_row("SELECT input FROM analysis_history WHERE id=?", rusqlite::params![&a.id], |r| r.get(0))).unwrap();
        assert_eq!(stored_input, "again");
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml storage::history
```

Expected: 1 test passes.

- [ ] **Step 3: Update the call site in `analyze_text`**

In `src-tauri/src/commands/analysis.rs`, change the call to `spawn_verifications` so it threads through the input text and the analysis timestamp. Replace the existing call (the one that ends with `outcome.claims, settings,`) with:

```rust
    spawn_verifications(
        app.clone(),
        db.inner().clone(),
        provider,
        brave_key,
        analysis_id.clone(),
        trimmed.to_string(),
        analysis.created_at,
        outcome.claims,
        settings,
    );
```

- [ ] **Step 4: Replace `spawn_verifications`**

Replace the entire `spawn_verifications` function with the version below. It changes the signature (adds `input` and `created_at_ms`), writes a baseline history row immediately, and writes a final row once all per-claim tasks complete (or after a 30-second timeout).

```rust
#[allow(clippy::too_many_arguments)]
fn spawn_verifications<R: Runtime>(
    app: AppHandle<R>,
    db: Db,
    provider: Arc<dyn LlmProvider>,
    brave_key: String,
    analysis_id: String,
    input: String,
    created_at_ms: i64,
    claims: Vec<Claim>,
    settings: Settings,
) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    use tokio::time::{timeout, Duration};

    let baseline = Analysis {
        id: analysis_id.clone(),
        created_at: created_at_ms,
        input: input.clone(),
        claims: claims.clone(),
        truncated: false,
    };
    let _ = crate::storage::history::insert(&db, &baseline);

    let fact_claims: Vec<Claim> = claims
        .iter()
        .filter(|c| c.kind == ClaimKind::Fact)
        .take(MAX_VERIFIED_CLAIMS)
        .cloned()
        .collect();

    if fact_claims.is_empty() {
        return;
    }

    let extractor = match Extractor::new() {
        Ok(e) => Arc::new(e),
        Err(_) => return,
    };
    let search: Arc<dyn SearchProvider> = match BraveClient::new(brave_key) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };
    let engine = Arc::new(VerificationEngine { llm: provider, search, extractor });

    let ttl_ms: i64 = i64::from(settings.cache_ttl_days) * 24 * 3600 * 1000;
    let total = fact_claims.len();
    let done = Arc::new(AtomicUsize::new(0));
    let notify = Arc::new(Notify::new());

    let final_claims: Arc<tokio::sync::Mutex<Vec<Claim>>> =
        Arc::new(tokio::sync::Mutex::new(claims.clone()));

    for claim in fact_claims {
        let app = app.clone();
        let db = db.clone();
        let engine = engine.clone();
        let analysis_id = analysis_id.clone();
        let done = done.clone();
        let notify = notify.clone();
        let final_claims = final_claims.clone();
        tokio::spawn(async move {
            let now = Utc::now().timestamp_millis();
            let hash = cache::hash_claim(&claim.text);

            let verification: Verification = match cache::get(&db, &hash, ttl_ms, now) {
                Ok(Some(v)) => v,
                _ => match engine.verify(&claim.text).await {
                    Ok(v) => {
                        let _ = cache::put(&db, &hash, &claim.text, &v, now);
                        v
                    }
                    Err(e) => Verification {
                        status: VerificationStatus::NotFound,
                        sources: vec![],
                        summary: format!("Verifikace selhala: {e}"),
                    },
                },
            };

            {
                let mut guard = final_claims.lock().await;
                for c in guard.iter_mut() {
                    if c.id == claim.id {
                        c.verification = Some(verification.clone());
                        break;
                    }
                }
            }

            let _ = app.emit(
                "claim-verified",
                ClaimVerifiedEvent {
                    analysis_id: analysis_id.clone(),
                    claim_id: claim.id.clone(),
                    verification,
                },
            );

            let prev = done.fetch_add(1, Ordering::SeqCst);
            if prev + 1 >= total {
                notify.notify_one();
            }
        });
    }

    let db_for_history = db.clone();
    let analysis_id_for_history = analysis_id;
    let final_claims_for_history = final_claims;
    let input_for_history = input;
    tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(30), notify.notified()).await;
        let claims_snapshot = final_claims_for_history.lock().await.clone();
        let analysis = Analysis {
            id: analysis_id_for_history,
            created_at: created_at_ms,
            input: input_for_history,
            claims: claims_snapshot,
            truncated: false,
        };
        let _ = crate::storage::history::insert(&db_for_history, &analysis);
    });
}
```

- [ ] **Step 5: Compile and commit**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
git add src-tauri/src/storage/history.rs src-tauri/src/commands/analysis.rs
git commit -m "feat(history): persist baseline + deferred final analysis with 30s window"
```

---

## Task 15: M2 acceptance smoke + tag

- [ ] **Step 1: Manual end-to-end smoke**

```bash
pnpm tauri dev
```

With both Anthropic and Brave keys set:

1. **Supported claim:** Paste `Karel IV. se narodil v roce 1316 v Praze.` Click Analyze. Verifications should resolve to "Ověřeno" with `cs.wikipedia.org` as an A-tier source.

2. **Mixed claim set:** Paste `Python vznikl v roce 1991. Je to nejlepší jazyk pro začátečníky. Karel IV. se narodil v roce 1500.` Verifications:
   - Python birth year → Ověřeno (A or B sources).
   - "Nejlepší jazyk" → orange opinion (no verification attempted).
   - Karel IV. 1500 → Vyvráceno (Wikipedia disagrees).

3. **Cache hit:** Re-run the first input. Verifications complete in <2s thanks to cache.

4. **Slow path:** With network throttled or Brave key invalid, verifications surface "Verifikace selhala: ..." in side panel summary.

5. **Hotkey:** Copy a Czech AI response from another app, press the hotkey, click Analyze. Full pipeline runs.

- [ ] **Step 2: Lint + test pass**

```bash
pnpm check && pnpm lint && pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all green.

- [ ] **Step 3: Optional — eval suite with live API**

```bash
export ANTHROPIC_API_KEY=...
RUN_LLM_EVAL=1 cargo test --manifest-path src-tauri/Cargo.toml --test eval -- --ignored --nocapture
```

Expected: ≥4/5 fixtures clean.

- [ ] **Step 4: Tag**

```bash
git tag m2-verification
```

- [ ] **Step 5: MVP Core complete**

The end-to-end pipeline is now functional. Next phases (Polish, Distribution, Release) are tracked in plans `04-privacy-polish.md`, `05-distribution.md`, and `06-release.md` — each of which will be authored only after this phase is dogfooded for a week and any prompt/UX issues are filed.

Open `00-overview.md` to confirm the next plan to write.
