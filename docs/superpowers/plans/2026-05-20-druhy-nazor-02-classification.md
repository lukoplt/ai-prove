# Druhý názor — Phase M1: Classification Pipeline

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans`. Read `2026-05-20-druhy-nazor-00-overview.md` and complete `2026-05-20-druhy-nazor-01-foundation.md` first.

**Goal:** Add LLM-driven atomization + classification of any pasted AI response. The user pastes text, hits Analyze, and within ~3 seconds sees the text re-rendered with color-coded tints per claim. Clicking a claim opens a side panel with the LLM-supplied Czech reason for the classification. No web verification yet — that ships in M2.

**Architecture:** A `LlmProvider` trait isolates the LLM dependency behind two methods (`atomize`, `judge`). Phase M1 implements the Anthropic adapter and uses only `atomize`. Atomization is a single Anthropic Messages API call that returns a structured tool-call payload (claims + truncated flag). Rust resolves each returned claim's verbatim substring back into start/end character offsets and assigns stable ids (`c1`, `c2`, …). The Tauri command emits a single `analysis-claims` event when finished; the frontend renders accordingly. A bench-style eval suite runs the prompt against curated CZ fixtures to catch regressions.

**Tech Stack additions on top of M0:** `async-trait`, `reqwest` (rustls-tls), `chrono`.

---

## Task 1: Add HTTP and async-trait dependencies

**Files:**

- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Append to `[dependencies]`**

```toml
async-trait = "0.1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream", "gzip"] }
chrono = { version = "0.4", default-features = false, features = ["clock", "serde"] }
sha2 = "0.10"
hex = "0.4"
```

- [ ] **Step 2: Verify build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: clean build.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore(deps): reqwest, async-trait, chrono, sha2 for M1"
```

---

## Task 2: LLM provider trait

**Files:**

- Create: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the module directory**

```bash
mkdir -p src-tauri/src/llm
```

- [ ] **Step 2: Write `src-tauri/src/llm/mod.rs`**

```rust
pub mod anthropic;
pub mod mock;
pub mod prompts;

use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// One raw claim as returned by the LLM. Offsets are computed by the caller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawClaim {
    pub text: String,
    pub kind: RawClaimKind,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawClaimKind {
    Fact,
    Inference,
    Opinion,
    Contradiction,
}

impl From<RawClaimKind> for crate::models::ClaimKind {
    fn from(k: RawClaimKind) -> Self {
        match k {
            RawClaimKind::Fact => Self::Fact,
            RawClaimKind::Inference => Self::Inference,
            RawClaimKind::Opinion => Self::Opinion,
            RawClaimKind::Contradiction => Self::Contradiction,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomizationResult {
    pub claims: Vec<RawClaim>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Supports,
    Contradicts,
    Mentions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeVerdict {
    pub stance: Stance,
    pub quote: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult>;
    async fn judge(&self, claim: &str, source_text: &str) -> AppResult<JudgeVerdict>;
}
```

- [ ] **Step 3: Add `llm` to `lib.rs`**

The full `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod hotkey;
pub mod llm;
pub mod models;
pub mod storage;
pub mod tray;

pub use error::{AppError, AppResult};
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/llm/mod.rs src-tauri/src/lib.rs
git commit -m "feat(llm): LlmProvider trait with atomize + judge"
```

---

## Task 3: Prompts module — bilingual atomize + judge scaffolding

**Files:**

- Create: `src-tauri/src/llm/prompts/mod.rs`
- Create: `src-tauri/src/llm/prompts/atomize_cs.txt`
- Create: `src-tauri/src/llm/prompts/atomize_en.txt`
- Create: `src-tauri/src/llm/prompts/judge_cs.txt`
- Create: `src-tauri/src/llm/prompts/judge_en.txt`

The atomize prompts are filled with real content here. The judge prompts ship as one-line placeholders and are replaced with their real bodies in M2 Task 1.

- [ ] **Step 1: Create the directory**

```bash
mkdir -p src-tauri/src/llm/prompts
```

- [ ] **Step 2: Write `src-tauri/src/llm/prompts/atomize_cs.txt`**

```
Jsi analytický nástroj „Druhý názor". Tvůj úkol je rozebrat odpověď AI na atomická tvrzení a každé z nich klasifikovat podle epistemického typu. Pracuj v češtině.

POSTUP:
1. Najdi všechna samostatná tvrzení v textu. Atomické = jedno tvrzení obsahuje jeden ověřitelný fakt nebo jednu myšlenku. Slož větu s více fakty rozdělíš na více tvrzení.
2. Pro každé tvrzení vyber přesně jeden epistemický typ:
   - fact — ověřitelné tvrzení, které lze nezávisle dohledat (jména, data, čísla, definice, historické události).
   - inference — logický závěr, interpretace nebo zobecnění; vychází z faktů, ale nemusí platit.
   - opinion — subjektivní hodnocení, doporučení nebo nepodložené tvrzení bez konkrétního důkazu.
   - contradiction — tvrzení, které si protiřečí s jiným tvrzením v té samé odpovědi. Použij jen tehdy, je-li rozpor jasný.
3. Pro každé tvrzení napiš krátkou (1 věta) českou reason, proč je právě tato klasifikace správná.
4. Vrať text tvrzení doslova tak, jak se vyskytuje ve vstupu (verbatim, včetně diakritiky a interpunkce). Žádné parafráze.
5. Pokud má text víc než 25 tvrzení, vrať prvních 25 v pořadí výskytu a nastav truncated=true. Jinak truncated=false.
6. Pokud je vstup prázdný nebo bez tvrzení, vrať prázdný seznam claims a truncated=false.

ZÁVAZNÝ FORMÁT VÝSTUPU:
Volej nástroj `submit_analysis` s argumenty {claims: [...], truncated: bool}. Nepoužívej žádný jiný výstup. Žádný text mimo nástroj.

PŘÍKLAD:
Vstup: „Karel IV. se narodil v roce 1316 v Praze a založil pražskou univerzitu. Jeho otec Jan Lucemburský byl podle mě nejlepší český král."

Výstup (jako tool input):
{
  "claims": [
    {"text": "Karel IV. se narodil v roce 1316", "kind": "fact", "reason": "Ověřitelné historické datum, dá se dohledat."},
    {"text": "v Praze", "kind": "fact", "reason": "Ověřitelný údaj o místě narození."},
    {"text": "založil pražskou univerzitu", "kind": "fact", "reason": "Historicky doložená událost."},
    {"text": "Jeho otec Jan Lucemburský byl podle mě nejlepší český král", "kind": "opinion", "reason": "Subjektivní hodnocení („podle mě\")."}
  ],
  "truncated": false
}
```

- [ ] **Step 3: Write `src-tauri/src/llm/prompts/atomize_en.txt`**

```
You are an analytical tool called "Druhý názor" (a second opinion). Your task is to break down an AI response into atomic claims and classify each one by epistemic type. Work in English.

PROCEDURE:
1. Find every standalone claim in the text. Atomic means one claim contains one verifiable fact or one idea. If a sentence packs several facts together, split it into several claims.
2. For each claim, pick exactly one epistemic type:
   - fact — verifiable statement that can be independently looked up (names, dates, numbers, definitions, historical events).
   - inference — a logical conclusion, interpretation, or generalization; follows from facts but may not hold.
   - opinion — subjective judgment, recommendation, or unsupported assertion without concrete evidence.
   - contradiction — a claim that contradicts another claim within the same response. Use only when the contradiction is unambiguous.
3. For each claim write a short (one-sentence) English reason explaining why this classification is correct.
4. Return the claim text verbatim as it appears in the input (including punctuation and casing). No paraphrasing.
5. If the text has more than 25 claims, return the first 25 in order of appearance and set truncated=true. Otherwise truncated=false.
6. If the input is empty or has no claims, return an empty claims list and truncated=false.

REQUIRED OUTPUT FORMAT:
Call the tool `submit_analysis` with arguments {claims: [...], truncated: bool}. Use no other output. No prose outside the tool call.

EXAMPLE:
Input: "Charles IV was born in 1316 in Prague and founded Prague University. His father John of Luxembourg was, in my view, the best Czech king."

Output (as tool input):
{
  "claims": [
    {"text": "Charles IV was born in 1316", "kind": "fact", "reason": "Verifiable historical date."},
    {"text": "in Prague", "kind": "fact", "reason": "Verifiable place of birth."},
    {"text": "founded Prague University", "kind": "fact", "reason": "Historically documented event."},
    {"text": "His father John of Luxembourg was, in my view, the best Czech king", "kind": "opinion", "reason": "Subjective judgment (\"in my view\")."}
  ],
  "truncated": false
}
```

- [ ] **Step 4: Write `src-tauri/src/llm/prompts/judge_cs.txt`**

```
Placeholder. Skutečný judge prompt je doplněn v M2 Task 1.
```

- [ ] **Step 5: Write `src-tauri/src/llm/prompts/judge_en.txt`**

```
Placeholder. The real judge prompt is filled in M2 Task 1.
```

- [ ] **Step 6: Write `src-tauri/src/llm/prompts/mod.rs`**

```rust
const ATOMIZE_CS: &str = include_str!("atomize_cs.txt");
const ATOMIZE_EN: &str = include_str!("atomize_en.txt");
const JUDGE_CS: &str = include_str!("judge_cs.txt");
const JUDGE_EN: &str = include_str!("judge_en.txt");

/// Returns the atomization+classification system prompt for the given UI locale.
/// Falls back to English for unsupported locales.
pub fn atomize_prompt(locale: &str) -> &'static str {
    match locale {
        "cs" => ATOMIZE_CS,
        _ => ATOMIZE_EN,
    }
}

/// Returns the source-judging system prompt for the given UI locale.
/// Falls back to English for unsupported locales.
pub fn judge_prompt(locale: &str) -> &'static str {
    match locale {
        "cs" => JUDGE_CS,
        _ => JUDGE_EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomize_prompt_returns_czech_for_cs() {
        assert!(atomize_prompt("cs").contains("Pracuj v češtině"));
    }

    #[test]
    fn atomize_prompt_returns_english_for_en() {
        assert!(atomize_prompt("en").contains("Work in English"));
    }

    #[test]
    fn atomize_prompt_falls_back_to_english_for_unknown() {
        assert_eq!(atomize_prompt("de"), atomize_prompt("en"));
        assert_eq!(atomize_prompt(""), atomize_prompt("en"));
    }

    #[test]
    fn judge_prompt_returns_a_non_empty_string_for_supported_locales() {
        assert!(!judge_prompt("cs").trim().is_empty());
        assert!(!judge_prompt("en").trim().is_empty());
    }
}
```

- [ ] **Step 7: Verify compilation and run prompt tests**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml llm::prompts
```

Expected: clean build, 4 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/llm/prompts/
git commit -m "feat(llm): bilingual atomize + judge prompt scaffolding with locale router"
```

---

## Task 4: Mock LLM provider

**Files:**

- Create: `src-tauri/src/llm/mock.rs`

The mock provider lets every later test run without hitting the network. It returns canned data based on simple string heuristics.

- [ ] **Step 1: Write `src-tauri/src/llm/mock.rs`**

```rust
use super::{AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance};
use crate::error::AppResult;
use async_trait::async_trait;
use std::sync::Mutex;

/// Programmable mock. Tests preload responses; the provider returns them in FIFO order.
#[derive(Default)]
pub struct MockProvider {
    atomize_queue: Mutex<Vec<AtomizationResult>>,
    judge_queue: Mutex<Vec<JudgeVerdict>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_atomize(&self, result: AtomizationResult) {
        self.atomize_queue.lock().unwrap().push(result);
    }

    pub fn push_judge(&self, verdict: JudgeVerdict) {
        self.judge_queue.lock().unwrap().push(verdict);
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn atomize(&self, _input: &str) -> AppResult<AtomizationResult> {
        let mut q = self.atomize_queue.lock().unwrap();
        if q.is_empty() {
            return Ok(AtomizationResult { claims: vec![], truncated: false });
        }
        Ok(q.remove(0))
    }

    async fn judge(&self, _claim: &str, _src: &str) -> AppResult<JudgeVerdict> {
        let mut q = self.judge_queue.lock().unwrap();
        if q.is_empty() {
            return Ok(JudgeVerdict { stance: Stance::Mentions, quote: String::new() });
        }
        Ok(q.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fifo_atomize() {
        let m = MockProvider::new();
        m.push_atomize(AtomizationResult {
            claims: vec![RawClaim {
                text: "A".into(),
                kind: RawClaimKind::Fact,
                reason: "r".into(),
            }],
            truncated: false,
        });
        m.push_atomize(AtomizationResult { claims: vec![], truncated: true });

        let first = m.atomize("ignored").await.unwrap();
        let second = m.atomize("ignored").await.unwrap();

        assert_eq!(first.claims.len(), 1);
        assert!(!first.truncated);
        assert!(second.claims.is_empty());
        assert!(second.truncated);
    }

    #[tokio::test]
    async fn empty_returns_empty() {
        let m = MockProvider::new();
        let res = m.atomize("x").await.unwrap();
        assert!(res.claims.is_empty());
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml llm::mock
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/mock.rs
git commit -m "feat(llm): programmable mock provider with FIFO queues"
```

---

## Task 5: Anthropic provider — request shape and tool schema

**Files:**

- Create: `src-tauri/src/llm/anthropic.rs`

We split this task in two: first the data types and tool schema (this task), then the actual `LlmProvider` impl (Task 6). Splitting keeps tests focused.

- [ ] **Step 1: Write `src-tauri/src/llm/anthropic.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::llm::{AtomizationResult, JudgeVerdict, RawClaim, RawClaimKind, Stance};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
pub const API_VERSION: &str = "2023-06-01";
pub const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";

pub const TOOL_NAME: &str = "submit_analysis";
pub const JUDGE_TOOL_NAME: &str = "submit_judgement";

#[derive(Debug, Serialize)]
pub(crate) struct Request<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    pub system: &'a str,
    pub tools: Vec<Value>,
    pub tool_choice: Value,
    pub messages: Vec<Message<'a>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
    ToolUse { name: String, input: Value },
}

pub(crate) fn build_atomize_request<'a>(model: &'a str, system: &'a str, user_msg: &'a Message<'a>) -> Request<'a> {
    Request {
        model,
        max_tokens: 4096,
        system,
        tools: vec![atomize_tool_schema()],
        tool_choice: json!({"type": "tool", "name": TOOL_NAME}),
        messages: vec![Message { role: user_msg.role, content: user_msg.content }],
    }
}

pub(crate) fn atomize_tool_schema() -> Value {
    json!({
        "name": TOOL_NAME,
        "description": "Submit the atomized and classified claims for the analyzed AI response.",
        "input_schema": {
            "type": "object",
            "properties": {
                "claims": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "text": {"type": "string", "description": "Verbatim substring of the input."},
                            "kind": {"type": "string", "enum": ["fact", "inference", "opinion", "contradiction"]},
                            "reason": {"type": "string", "description": "One Czech sentence."}
                        },
                        "required": ["text", "kind", "reason"]
                    }
                },
                "truncated": {"type": "boolean"}
            },
            "required": ["claims", "truncated"]
        }
    })
}

pub(crate) fn judge_tool_schema() -> Value {
    json!({
        "name": JUDGE_TOOL_NAME,
        "description": "Submit a stance verdict for a claim given a source excerpt.",
        "input_schema": {
            "type": "object",
            "properties": {
                "stance": {"type": "string", "enum": ["supports", "contradicts", "mentions"]},
                "quote": {"type": "string", "description": "Short quote from the source supporting the verdict."}
            },
            "required": ["stance", "quote"]
        }
    })
}

pub(crate) fn parse_atomize_response(body: &str) -> AppResult<AtomizationResult> {
    let resp: Response = serde_json::from_str(body)?;
    if !matches!(resp.stop_reason.as_deref(), Some("tool_use") | Some("end_turn")) {
        return Err(AppError::Other(format!(
            "anthropic returned unexpected stop_reason: {:?}",
            resp.stop_reason
        )));
    }
    for block in resp.content {
        if let ContentBlock::ToolUse { name, input } = block {
            if name == TOOL_NAME {
                return parse_atomize_input(&input);
            }
        }
    }
    Err(AppError::Other("anthropic response missing tool_use".into()))
}

fn parse_atomize_input(value: &Value) -> AppResult<AtomizationResult> {
    let claims_value = value
        .get("claims")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Other("tool input missing claims[]".into()))?;
    let truncated = value
        .get("truncated")
        .and_then(Value::as_bool)
        .ok_or_else(|| AppError::Other("tool input missing truncated bool".into()))?;

    let mut claims = Vec::with_capacity(claims_value.len());
    for c in claims_value {
        let text = c
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Other("claim missing text".into()))?
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        let kind_str = c
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Other("claim missing kind".into()))?;
        let kind = parse_kind(kind_str)?;
        let reason = c
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        claims.push(RawClaim { text, kind, reason });
    }
    Ok(AtomizationResult { claims, truncated })
}

pub(crate) fn parse_judge_response(body: &str) -> AppResult<JudgeVerdict> {
    let resp: Response = serde_json::from_str(body)?;
    for block in resp.content {
        if let ContentBlock::ToolUse { name, input } = block {
            if name == JUDGE_TOOL_NAME {
                let stance = input
                    .get("stance")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::Other("judge missing stance".into()))?;
                let quote = input
                    .get("quote")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                return Ok(JudgeVerdict { stance: parse_stance(stance)?, quote });
            }
        }
    }
    Err(AppError::Other("anthropic response missing judge tool_use".into()))
}

fn parse_kind(s: &str) -> AppResult<RawClaimKind> {
    match s {
        "fact" => Ok(RawClaimKind::Fact),
        "inference" => Ok(RawClaimKind::Inference),
        "opinion" => Ok(RawClaimKind::Opinion),
        "contradiction" => Ok(RawClaimKind::Contradiction),
        other => Err(AppError::Other(format!("unknown claim kind: {other}"))),
    }
}

fn parse_stance(s: &str) -> AppResult<Stance> {
    match s {
        "supports" => Ok(Stance::Supports),
        "contradicts" => Ok(Stance::Contradicts),
        "mentions" => Ok(Stance::Mentions),
        other => Err(AppError::Other(format!("unknown stance: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_atomize_response() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [
                        {"text": "Karel IV. se narodil v roce 1316", "kind": "fact", "reason": "Historické datum."},
                        {"text": "byl podle mě nejlepší král", "kind": "opinion", "reason": "Subjektivní hodnocení."}
                    ],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let result = parse_atomize_response(body).unwrap();
        assert_eq!(result.claims.len(), 2);
        assert!(!result.truncated);
        assert_eq!(result.claims[0].kind, RawClaimKind::Fact);
        assert_eq!(result.claims[1].kind, RawClaimKind::Opinion);
    }

    #[test]
    fn rejects_response_without_tool_use() {
        let body = r#"{"content": [{"type": "text", "text": "hello"}], "stop_reason": "end_turn"}"#;
        assert!(parse_atomize_response(body).is_err());
    }

    #[test]
    fn drops_empty_text_claims() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [
                        {"text": "  ", "kind": "fact", "reason": "x"},
                        {"text": "Real claim", "kind": "fact", "reason": "x"}
                    ],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let result = parse_atomize_response(body).unwrap();
        assert_eq!(result.claims.len(), 1);
        assert_eq!(result.claims[0].text, "Real claim");
    }

    #[test]
    fn rejects_unknown_kind() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_analysis", "input": {
                    "claims": [{"text": "x", "kind": "lies", "reason": "y"}],
                    "truncated": false
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        assert!(parse_atomize_response(body).is_err());
    }

    #[test]
    fn parses_judge_response() {
        let body = r#"{
            "content": [
                {"type": "tool_use", "name": "submit_judgement", "input": {
                    "stance": "supports", "quote": "Wikipedia confirms 1316."
                }}
            ],
            "stop_reason": "tool_use"
        }"#;
        let v = parse_judge_response(body).unwrap();
        assert_eq!(v.stance, Stance::Supports);
        assert_eq!(v.quote, "Wikipedia confirms 1316.");
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml llm::anthropic
```

Expected: 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/anthropic.rs
git commit -m "feat(anthropic): request shape, tool schema, response parsers"
```

---

## Task 6: Anthropic provider — networked LlmProvider impl with locale routing

**Files:**

- Modify: `src-tauri/src/llm/anthropic.rs`

The provider now carries a `locale` field. Both `atomize` and `judge` look up their system prompt via the bilingual helpers added in Task 3.

- [ ] **Step 1: Append the provider implementation**

Add to `src-tauri/src/llm/anthropic.rs` (after the existing code, before the `#[cfg(test)]` block):

```rust
use crate::llm::LlmProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    locale: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, locale: String) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::Other(format!("reqwest builder: {e}")))?;
        Ok(Self { client, api_key, model, locale })
    }

    async fn post(&self, body: &Value) -> AppResult<String> {
        let resp = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("anthropic http: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| AppError::Other(format!("anthropic body: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Other(format!("anthropic {status}: {text}")));
        }
        Ok(text)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult> {
        let system = crate::llm::prompts::atomize_prompt(&self.locale);
        let body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system,
            "tools": [atomize_tool_schema()],
            "tool_choice": {"type": "tool", "name": TOOL_NAME},
            "messages": [{"role": "user", "content": input}]
        });
        let text = self.post(&body).await?;
        parse_atomize_response(&text)
    }

    async fn judge(&self, claim: &str, source_text: &str) -> AppResult<JudgeVerdict> {
        let system = crate::llm::prompts::judge_prompt(&self.locale);
        let user_msg = match self.locale.as_str() {
            "cs" => format!(
                "Tvrzení:\n{claim}\n\nZdrojový text:\n{source_text}\n\nUrči stanovisko zdroje k tvrzení."
            ),
            _ => format!(
                "Claim:\n{claim}\n\nSource text:\n{source_text}\n\nDecide the source's stance toward the claim."
            ),
        };
        let body = json!({
            "model": self.model,
            "max_tokens": 512,
            "system": system,
            "tools": [judge_tool_schema()],
            "tool_choice": {"type": "tool", "name": JUDGE_TOOL_NAME},
            "messages": [{"role": "user", "content": user_msg}]
        });
        let text = self.post(&body).await?;
        parse_judge_response(&text)
    }
}
```

Both judge prompts already exist as placeholder files (M1 Task 3 created them); the real bodies arrive in M2 Task 1. The crate compiles either way because `judge_prompt(locale)` returns a non-empty static `&str` for every supported locale.

- [ ] **Step 2: Verify compilation**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/anthropic.rs
git commit -m "feat(anthropic): networked LlmProvider impl with locale-aware prompt routing"
```

---

## Task 7: Pipeline — atomize service with offset resolution

**Files:**

- Create: `src-tauri/src/pipeline/mod.rs`
- Create: `src-tauri/src/pipeline/atomize.rs`
- Modify: `src-tauri/src/lib.rs`

The atomize service takes a raw input string and a provider, calls the provider, then resolves each `RawClaim.text` back into start/end byte offsets via first-match substring search. Claims whose text is not found verbatim are dropped (model misquoted).

- [ ] **Step 1: Create the directory**

```bash
mkdir -p src-tauri/src/pipeline
```

- [ ] **Step 2: Write `src-tauri/src/pipeline/mod.rs`**

```rust
pub mod atomize;

pub use atomize::atomize_to_claims;
```

- [ ] **Step 3: Write `src-tauri/src/pipeline/atomize.rs`**

```rust
use crate::error::AppResult;
use crate::llm::{AtomizationResult, LlmProvider, RawClaim};
use crate::models::{Claim, ClaimKind};

pub const MAX_CLAIMS: usize = 25;

#[derive(Debug, Clone, PartialEq)]
pub struct AtomizationOutcome {
    pub claims: Vec<Claim>,
    pub truncated: bool,
}

pub async fn atomize_to_claims(
    provider: &dyn LlmProvider,
    input: &str,
) -> AppResult<AtomizationOutcome> {
    let AtomizationResult { claims: raw, truncated } = provider.atomize(input).await?;
    let resolved = resolve_offsets(input, raw);
    let truncated = truncated || resolved.len() > MAX_CLAIMS;
    let mut taken: Vec<Claim> = resolved.into_iter().take(MAX_CLAIMS).collect();
    for (idx, claim) in taken.iter_mut().enumerate() {
        claim.id = format!("c{}", idx + 1);
    }
    Ok(AtomizationOutcome { claims: taken, truncated })
}

/// Resolves each raw claim into a Claim with byte offsets. Drops misquoted claims.
fn resolve_offsets(input: &str, raw: Vec<RawClaim>) -> Vec<Claim> {
    let mut cursor = 0usize;
    let mut out = Vec::with_capacity(raw.len());
    for r in raw {
        let needle = r.text.trim();
        if needle.is_empty() {
            continue;
        }
        // Prefer matches at or after the running cursor (preserve document order).
        if let Some(rel) = input.get(cursor..).and_then(|s| s.find(needle)) {
            let start = cursor + rel;
            let end = start + needle.len();
            cursor = end;
            out.push(Claim {
                id: String::new(),
                text: needle.to_string(),
                start,
                end,
                kind: ClaimKind::from(r.kind),
                reason: r.reason,
                verification: None,
            });
            continue;
        }
        // Fall back to any earlier occurrence (LLM may have reported out of order).
        if let Some(start) = input.find(needle) {
            let end = start + needle.len();
            out.push(Claim {
                id: String::new(),
                text: needle.to_string(),
                start,
                end,
                kind: ClaimKind::from(r.kind),
                reason: r.reason,
                verification: None,
            });
        }
        // else: drop silently. Frontend never sees a claim with bad offsets.
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock::MockProvider;
    use crate::llm::{AtomizationResult, RawClaim, RawClaimKind};

    fn raw(text: &str, kind: RawClaimKind) -> RawClaim {
        RawClaim { text: text.into(), kind, reason: "r".into() }
    }

    #[tokio::test]
    async fn resolves_offsets_in_order() {
        let input = "Karel IV. se narodil v roce 1316 v Praze. Byl to skvělý král.";
        let m = MockProvider::new();
        m.push_atomize(AtomizationResult {
            claims: vec![
                raw("Karel IV. se narodil v roce 1316", RawClaimKind::Fact),
                raw("v Praze", RawClaimKind::Fact),
                raw("Byl to skvělý král", RawClaimKind::Opinion),
            ],
            truncated: false,
        });
        let out = atomize_to_claims(&m, input).await.unwrap();
        assert_eq!(out.claims.len(), 3);
        assert_eq!(out.claims[0].id, "c1");
        assert_eq!(out.claims[1].id, "c2");
        assert_eq!(out.claims[2].id, "c3");
        assert!(out.claims[0].start < out.claims[1].start);
        assert!(out.claims[1].start < out.claims[2].start);
        assert_eq!(&input[out.claims[0].start..out.claims[0].end], out.claims[0].text);
    }

    #[tokio::test]
    async fn drops_misquoted_claim() {
        let input = "Karel IV. se narodil v roce 1316.";
        let m = MockProvider::new();
        m.push_atomize(AtomizationResult {
            claims: vec![
                raw("Karel IV. se narodil v roce 1316", RawClaimKind::Fact),
                raw("Karel IV. se narodil v roce 1500", RawClaimKind::Fact),
            ],
            truncated: false,
        });
        let out = atomize_to_claims(&m, input).await.unwrap();
        assert_eq!(out.claims.len(), 1);
    }

    #[tokio::test]
    async fn caps_at_max_claims() {
        let input = "a ".repeat(40);
        let m = MockProvider::new();
        m.push_atomize(AtomizationResult {
            claims: (0..40).map(|_| raw("a", RawClaimKind::Fact)).collect(),
            truncated: false,
        });
        let out = atomize_to_claims(&m, &input).await.unwrap();
        assert_eq!(out.claims.len(), MAX_CLAIMS);
        assert!(out.truncated);
    }

    #[tokio::test]
    async fn preserves_truncated_flag_from_provider() {
        let m = MockProvider::new();
        m.push_atomize(AtomizationResult { claims: vec![], truncated: true });
        let out = atomize_to_claims(&m, "x").await.unwrap();
        assert!(out.truncated);
    }
}
```

- [ ] **Step 4: Re-export pipeline from `lib.rs`**

Full `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod hotkey;
pub mod llm;
pub mod models;
pub mod pipeline;
pub mod storage;
pub mod tray;

pub use error::{AppError, AppResult};
```

- [ ] **Step 5: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml pipeline::atomize
```

Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/pipeline src-tauri/src/lib.rs
git commit -m "feat(pipeline): atomize_to_claims with offset resolution and 25-claim cap"
```

---

## Task 8: Analysis Tauri command + events

**Files:**

- Modify: `src-tauri/src/commands/analysis.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write `src-tauri/src/commands/analysis.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::llm::anthropic::AnthropicProvider;
use crate::models::Analysis;
use crate::pipeline::atomize_to_claims;
use crate::storage::{keychain, settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY}};
use chrono::Utc;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

pub const ACCOUNT_ANTHROPIC: &str = "anthropic";

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

#[tauri::command]
pub async fn analyze_text<R: Runtime>(app: AppHandle<R>, text: String) -> AppResult<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("input is empty".into()));
    }

    let api_key = keychain::get_api_key(ACCOUNT_ANTHROPIC)?
        .ok_or_else(|| AppError::NotFound("anthropic key".into()))?;

    let settings = load_settings(&app);
    let provider = AnthropicProvider::new(api_key, settings.model.clone(), settings.locale.clone())?;

    let analysis_id = Uuid::now_v7().to_string();
    app.emit("analysis-started", AnalysisStartedEvent { analysis_id: analysis_id.clone() })
        .map_err(|e| AppError::Other(format!("emit: {e}")))?;

    let outcome = atomize_to_claims(&provider, trimmed).await?;
    let analysis = Analysis {
        id: analysis_id.clone(),
        created_at: Utc::now().timestamp_millis(),
        input: trimmed.to_string(),
        claims: outcome.claims,
        truncated: outcome.truncated,
    };

    app.emit(
        "analysis-claims",
        AnalysisClaimsEvent { analysis_id: analysis_id.clone(), analysis },
    )
    .map_err(|e| AppError::Other(format!("emit: {e}")))?;

    Ok(analysis_id)
}

fn load_model<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let store = app.store(SETTINGS_FILE).ok()?;
    let v = store.get(SETTINGS_KEY)?;
    let settings: Settings = serde_json::from_value(v).ok()?;
    Some(settings.model)
}
```

- [ ] **Step 2: Register `analyze_text` in `main.rs`**

In the `invoke_handler!` macro, add `analyze_text`. The full handler list:

```rust
.invoke_handler(tauri::generate_handler![
    get_settings,
    set_settings,
    set_api_key,
    clear_api_key,
    has_api_key,
    druhy_nazor_lib::commands::analysis::analyze_text,
])
```

- [ ] **Step 3: Verify compilation**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/analysis.rs src-tauri/src/main.rs
git commit -m "feat(commands): analyze_text emits analysis-started and analysis-claims"
```

---

## Task 9: Frontend analysis store and Tauri wrapper

**Files:**

- Modify: `src/lib/api.ts`
- Create: `src/lib/stores/analysis.svelte.ts`

- [ ] **Step 1: Extend `src/lib/api.ts`**

Append:

```ts
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Analysis } from './types';

export async function analyzeText(text: string): Promise<string> {
  return invoke<string>('analyze_text', { text });
}

export interface AnalysisStartedEvent {
  analysisId: string;
}

export interface AnalysisClaimsEvent {
  analysisId: string;
  analysis: Analysis;
}

export function onAnalysisStarted(handler: (e: AnalysisStartedEvent) => void): Promise<UnlistenFn> {
  return listen<AnalysisStartedEvent>('analysis-started', (msg) => handler(msg.payload));
}

export function onAnalysisClaims(handler: (e: AnalysisClaimsEvent) => void): Promise<UnlistenFn> {
  return listen<AnalysisClaimsEvent>('analysis-claims', (msg) => handler(msg.payload));
}
```

- [ ] **Step 2: Create `src/lib/stores/analysis.svelte.ts`**

```ts
import { analyzeText, onAnalysisClaims, onAnalysisStarted } from '$lib/api';
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
git commit -m "feat(web): analysis store wired to analysis-started/analysis-claims events"
```

---

## Task 10: ClaimText component

**Files:**

- Create: `src/lib/components/ClaimText.svelte`
- Test: `src/lib/components/ClaimText.test.ts`

This component renders the original input with each claim wrapped in a `<span>` carrying a kind-specific CSS class. Spans are clickable; clicking selects the claim in the analysis store.

- [ ] **Step 1: Write the failing test**

Create `src/lib/components/ClaimText.test.ts`:

```ts
import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import ClaimText from './ClaimText.svelte';
import type { Claim } from '$lib/types';

const input = 'Karel IV. se narodil v roce 1316. Byl skvělý.';
const claims: Claim[] = [
  {
    id: 'c1',
    text: 'Karel IV. se narodil v roce 1316',
    start: 0,
    end: 32,
    kind: 'fact',
    reason: 'Datum.',
    verification: null,
  },
  {
    id: 'c2',
    text: 'Byl skvělý',
    start: 34,
    end: 44,
    kind: 'opinion',
    reason: 'Hodnocení.',
    verification: null,
  },
];

describe('ClaimText', () => {
  it('renders two highlighted spans with kind classes', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: null });
    const spans = container.querySelectorAll('span.claim');
    expect(spans.length).toBe(2);
    expect(spans[0].classList.contains('kind-fact')).toBe(true);
    expect(spans[1].classList.contains('kind-opinion')).toBe(true);
  });

  it('marks the selected claim', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: 'c2' });
    const selected = container.querySelector('span.claim.selected');
    expect(selected?.getAttribute('data-id')).toBe('c2');
  });

  it('preserves text between and after claims', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: null });
    expect(container.textContent).toBe(input);
  });
});
```

- [ ] **Step 2: Run it to confirm failure**

```bash
pnpm test ClaimText
```

Expected: tests fail because the component does not exist yet.

- [ ] **Step 3: Create `src/lib/components/ClaimText.svelte`**

```svelte
<script lang="ts">
  import type { Claim } from '$lib/types';

  let {
    input,
    claims,
    selectedId,
    onSelect = () => {},
  }: {
    input: string;
    claims: Claim[];
    selectedId: string | null;
    onSelect?: (id: string) => void;
  } = $props();

  const segments = $derived(buildSegments(input, claims));

  type Seg = { kind: 'plain'; text: string } | { kind: 'claim'; claim: Claim };

  function buildSegments(text: string, list: Claim[]): Seg[] {
    if (list.length === 0) return [{ kind: 'plain', text }];
    const sorted = [...list].sort((a, b) => a.start - b.start);
    const out: Seg[] = [];
    let cursor = 0;
    for (const c of sorted) {
      if (c.start < cursor) continue; // overlap — skip
      if (c.start > cursor) out.push({ kind: 'plain', text: text.slice(cursor, c.start) });
      out.push({ kind: 'claim', claim: c });
      cursor = c.end;
    }
    if (cursor < text.length) out.push({ kind: 'plain', text: text.slice(cursor) });
    return out;
  }
</script>

<p class="ct">
  {#each segments as seg, i (i)}
    {#if seg.kind === 'plain'}
      <span class="plain">{seg.text}</span>
    {:else}
      <span
        class="claim kind-{seg.claim.kind}"
        class:selected={seg.claim.id === selectedId}
        data-id={seg.claim.id}
        role="button"
        tabindex="0"
        onclick={() => onSelect(seg.claim.id)}
        onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onSelect(seg.claim.id)}
        >{seg.claim.text}</span
      >
    {/if}
  {/each}
</p>

<style>
  .ct {
    line-height: 1.7;
    font-size: 15px;
    white-space: pre-wrap;
  }
  .claim {
    border-radius: 3px;
    padding: 1px 2px;
    cursor: pointer;
    transition: outline-color 120ms ease;
    outline: 2px solid transparent;
  }
  .claim.selected {
    outline-color: #111827;
  }
  .kind-fact {
    background: rgba(34, 197, 94, 0.18);
  }
  .kind-inference {
    background: rgba(234, 179, 8, 0.18);
  }
  .kind-opinion {
    background: rgba(249, 115, 22, 0.22);
  }
  .kind-contradiction {
    background: rgba(239, 68, 68, 0.22);
  }
</style>
```

- [ ] **Step 4: Run the test**

```bash
pnpm test ClaimText
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/ClaimText.svelte src/lib/components/ClaimText.test.ts
git commit -m "feat(web): ClaimText component with kind-colored spans and selection"
```

---

## Task 11: SidePanel component

**Files:**

- Create: `src/lib/components/SidePanel.svelte`

The side panel shows details for the selected claim. In M1 it renders kind badge, the reason from the LLM, and a placeholder source area. M2 fills the source area with real data.

- [ ] **Step 1: Add side-panel string keys**

Extend `src/lib/i18n/cs.json` — add to the root object:

```json
"sidepanel": {
  "empty": "Klikni na zvýrazněné tvrzení vlevo.",
  "kind_fact": "Ověřitelný fakt",
  "kind_inference": "Odvození",
  "kind_opinion": "Domněnka",
  "kind_contradiction": "Vnitřní rozpor",
  "why_label": "Proč tato barva",
  "sources_label": "Zdroje",
  "sources_pending": "Verifikace se připraví v dalším milníku."
}
```

And mirror to `src/lib/i18n/en.json`:

```json
"sidepanel": {
  "empty": "Click a highlighted claim on the left.",
  "kind_fact": "Verifiable fact",
  "kind_inference": "Inference",
  "kind_opinion": "Opinion",
  "kind_contradiction": "Internal contradiction",
  "why_label": "Why this color",
  "sources_label": "Sources",
  "sources_pending": "Verification arrives in the next milestone."
}
```

- [ ] **Step 2: Write `src/lib/components/SidePanel.svelte`**

```svelte
<script lang="ts">
  import type { Claim } from '$lib/types';
  import { t } from '$lib/stores/i18n.svelte';

  let { claim }: { claim: Claim | null } = $props();

  function kindLabel(k: Claim['kind']): string {
    return t(`sidepanel.kind_${k}`);
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
      <p class="pending">{t('sidepanel.sources_pending')}</p>
    </section>
  {/if}
</aside>

<style>
  .sp {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 16px;
    background: #fafafa;
    min-height: 320px;
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
    margin: 0 0 12px;
    font-size: 14px;
  }
  .pending {
    color: #9ca3af;
    font-style: italic;
  }
</style>
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/SidePanel.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(web): SidePanel component with kind badge and reason"
```

---

## Task 12: Wire main page to analysis flow

**Files:**

- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add a summary string key**

Extend `src/lib/i18n/cs.json` with:

```json
"summary": {
  "analyzing": "Analyzuji…",
  "claims_count": "{count} tvrzení",
  "truncated_warning": "Vstup byl zkrácen na prvních 25 tvrzení.",
  "error_prefix": "Chyba: {msg}",
  "missing_key": "Nastav nejdřív Anthropic API klíč v Nastavení."
}
```

And in `en.json`:

```json
"summary": {
  "analyzing": "Analyzing…",
  "claims_count": "{count} claims",
  "truncated_warning": "Input was truncated to the first 25 claims.",
  "error_prefix": "Error: {msg}",
  "missing_key": "Set the Anthropic API key in Settings first."
}
```

- [ ] **Step 2: Add a tiny formatter helper**

Append to `src/lib/stores/i18n.svelte.ts`:

```ts
export function tf(key: string, vars: Record<string, string | number>): string {
  let s = t(key);
  for (const [k, v] of Object.entries(vars)) {
    s = s.replaceAll(`{${k}}`, String(v));
  }
  return s;
}
```

- [ ] **Step 3: Replace `src/routes/+page.svelte`**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { goto } from '$app/navigation';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import ClaimText from '$lib/components/ClaimText.svelte';
  import SidePanel from '$lib/components/SidePanel.svelte';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let inputText = $state('');

  onMount(async () => {
    await analysisStore.init();
    const unlisten = listen('capture-trigger', async () => {
      const clipboard = await readText();
      if (clipboard) inputText = clipboard;
    });
    return () => {
      unlisten.then((u) => u());
    };
  });

  async function handleAnalyze(text: string) {
    if (!settings.anthropicPresent) {
      alert(t('summary.missing_key'));
      goto('/settings');
      return;
    }
    inputText = text;
    await analysisStore.run(text);
  }
</script>

<main class="page">
  <header>
    <h1>{t('app.title')}</h1>
    <nav>
      <button type="button" onclick={() => goto('/settings')}>{t('common.settings')}</button>
    </nav>
  </header>

  <p class="tagline">{t('app.tagline')}</p>

  <PasteInput bind:value={inputText} onAnalyze={handleAnalyze} />

  <section class="result">
    {#if analysisStore.status === 'running'}
      <p class="status">{t('summary.analyzing')}</p>
    {:else if analysisStore.status === 'error'}
      <p class="status error">{tf('summary.error_prefix', { msg: analysisStore.error ?? '?' })}</p>
    {:else if analysisStore.status === 'done' && analysisStore.current}
      <div class="grid">
        <div class="left">
          <p class="meta">
            {tf('summary.claims_count', { count: analysisStore.current.claims.length })}
          </p>
          {#if analysisStore.current.truncated}
            <p class="warning">{t('summary.truncated_warning')}</p>
          {/if}
          <ClaimText
            input={analysisStore.current.input}
            claims={analysisStore.current.claims}
            selectedId={analysisStore.selectedId}
            onSelect={(id) => analysisStore.select(id)}
          />
        </div>
        <div class="right">
          <SidePanel claim={analysisStore.selectedClaim} />
        </div>
      </div>
    {/if}
  </section>
</main>

<style>
  .page {
    max-width: 1080px;
    margin: 0 auto;
    padding: 24px;
    font-family: system-ui, sans-serif;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
  }
  .tagline {
    color: #6b7280;
    margin: 0 0 16px;
  }
  .result {
    margin-top: 16px;
  }
  .status {
    color: #6b7280;
    font-size: 14px;
  }
  .status.error {
    color: #b91c1c;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 320px;
    gap: 16px;
  }
  .meta {
    color: #6b7280;
    font-size: 13px;
    margin: 0 0 8px;
  }
  .warning {
    color: #92400e;
    background: #fef3c7;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 13px;
    margin: 0 0 8px;
  }
</style>
```

- [ ] **Step 4: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. With no API key set, clicking Analyze prompts to set the key and routes to Settings.
2. After setting a real Anthropic key, paste a paragraph of Czech AI output. Click Analyze. Within ~4 seconds, the text re-renders with colored highlights.
3. Click any highlighted claim. The side panel shows the kind badge, the verbatim quote, and the Czech reason.
4. Try a long input (>25 claims). The yellow truncation banner appears.

- [ ] **Step 5: Commit**

```bash
git add src/lib/i18n/cs.json src/lib/i18n/en.json src/lib/stores/i18n.svelte.ts src/routes/+page.svelte
git commit -m "feat(web): wire main page to live atomization with ClaimText + SidePanel"
```

---

## Task 13: LLM eval suite

**Files:**

- Create: `src-tauri/tests/eval.rs`
- Create: `src-tauri/tests/fixtures/01-karel-iv.json`
- Create: `src-tauri/tests/fixtures/02-mixed-opinions.json`
- Create: `src-tauri/tests/fixtures/03-contradiction.json`
- Create: `src-tauri/tests/fixtures/04-numerical-claims.json`
- Create: `src-tauri/tests/fixtures/05-no-claims.json`

The eval suite verifies the prompt still works after edits. It's gated by `RUN_LLM_EVAL=1` so it never runs in standard CI.

- [ ] **Step 1: Create fixture `01-karel-iv.json`**

```json
{
  "name": "karel-iv",
  "input": "Karel IV. se narodil v roce 1316 v Praze a založil pražskou univerzitu, která je nejstarší ve střední Evropě. Jeho otec Jan Lucemburský padl v bitvě u Kresčaku.",
  "expected_min_claims": 4,
  "must_classify_as_fact": [
    "Karel IV. se narodil v roce 1316",
    "založil pražskou univerzitu",
    "Jeho otec Jan Lucemburský padl v bitvě u Kresčaku"
  ],
  "must_classify_as_opinion": []
}
```

- [ ] **Step 2: Create fixture `02-mixed-opinions.json`**

```json
{
  "name": "mixed-opinions",
  "input": "Nejlepší programovací jazyk na začátek je Python, protože má jednoduchou syntaxi. Python vznikl v roce 1991. Mnoho vývojářů ho doporučuje pro začátečníky.",
  "expected_min_claims": 3,
  "must_classify_as_fact": ["Python vznikl v roce 1991"],
  "must_classify_as_opinion": ["Nejlepší programovací jazyk na začátek je Python"]
}
```

- [ ] **Step 3: Create fixture `03-contradiction.json`**

```json
{
  "name": "contradiction",
  "input": "Albert Einstein se narodil v roce 1879. Einstein zemřel v roce 1955 a žil přesně 65 let.",
  "expected_min_claims": 3,
  "must_classify_as_fact": [
    "Albert Einstein se narodil v roce 1879",
    "Einstein zemřel v roce 1955"
  ],
  "must_classify_as_opinion": []
}
```

- [ ] **Step 4: Create fixture `04-numerical-claims.json`**

```json
{
  "name": "numerical-claims",
  "input": "Česká republika má rozlohu 78 866 km² a 10,5 milionu obyvatel. Nejvyšší horou je Sněžka s nadmořskou výškou 1603 m.",
  "expected_min_claims": 4,
  "must_classify_as_fact": ["Česká republika má rozlohu 78 866 km²", "Nejvyšší horou je Sněžka"],
  "must_classify_as_opinion": []
}
```

- [ ] **Step 5: Create fixture `05-no-claims.json`**

```json
{
  "name": "no-claims",
  "input": "Ahoj! Jak se máš?",
  "expected_min_claims": 0,
  "must_classify_as_fact": [],
  "must_classify_as_opinion": []
}
```

- [ ] **Step 6: Create `src-tauri/tests/eval.rs`**

```rust
//! LLM prompt eval suite. Gated by RUN_LLM_EVAL=1 so it never runs in CI by default.

use druhy_nazor_lib::llm::anthropic::AnthropicProvider;
use druhy_nazor_lib::llm::LlmProvider;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Fixture {
    name: String,
    input: String,
    expected_min_claims: usize,
    must_classify_as_fact: Vec<String>,
    must_classify_as_opinion: Vec<String>,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixtures() -> Vec<Fixture> {
    let mut out = Vec::new();
    for entry in fs::read_dir(fixtures_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let txt = fs::read_to_string(&path).unwrap();
        out.push(serde_json::from_str(&txt).unwrap());
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn provider() -> AnthropicProvider {
    let key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY required for eval suite");
    let model = std::env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());
    AnthropicProvider::new(key, model).expect("provider construction")
}

#[tokio::test]
#[ignore]
async fn llm_eval_suite() {
    if std::env::var("RUN_LLM_EVAL").as_deref() != Ok("1") {
        eprintln!("RUN_LLM_EVAL=1 not set; skipping.");
        return;
    }
    let fixtures = load_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let p = provider();
    let mut failures: Vec<String> = Vec::new();

    for fx in &fixtures {
        let res = match p.atomize(&fx.input).await {
            Ok(r) => r,
            Err(e) => {
                failures.push(format!("{}: provider error: {e}", fx.name));
                continue;
            }
        };

        if res.claims.len() < fx.expected_min_claims {
            failures.push(format!(
                "{}: got {} claims, expected min {}",
                fx.name,
                res.claims.len(),
                fx.expected_min_claims
            ));
        }

        for needle in &fx.must_classify_as_fact {
            let hit = res.claims.iter().any(|c| {
                c.text.contains(needle)
                    && matches!(c.kind, druhy_nazor_lib::llm::RawClaimKind::Fact)
            });
            if !hit {
                failures.push(format!("{}: expected fact containing {needle:?}", fx.name));
            }
        }
        for needle in &fx.must_classify_as_opinion {
            let hit = res.claims.iter().any(|c| {
                c.text.contains(needle)
                    && matches!(c.kind, druhy_nazor_lib::llm::RawClaimKind::Opinion)
            });
            if !hit {
                failures.push(format!("{}: expected opinion containing {needle:?}", fx.name));
            }
        }
    }

    let pass = fixtures.len() * 1 - failures.len().min(fixtures.len());
    let total = fixtures.len();
    eprintln!("eval: {pass}/{total} fixtures clean");
    for f in &failures {
        eprintln!("  ✗ {f}");
    }
    // Threshold: at most 20% fixture-level failure tolerance.
    let max_failed = (fixtures.len() as f32 * 0.2).ceil() as usize;
    assert!(failures.len() <= max_failed, "eval threshold not met");
}
```

- [ ] **Step 7: Smoke-run locally with a real key**

```bash
export ANTHROPIC_API_KEY=sk-ant-yourkey
RUN_LLM_EVAL=1 cargo test --manifest-path src-tauri/Cargo.toml --test eval -- --ignored --nocapture
```

Expected: eval prints `eval: 5/5 fixtures clean` or, if any fail, ≤1 failure (within tolerance). If more fail, iterate on the prompt in `atomize_cs.txt` and rerun. Do not move on until passing.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/tests/
git commit -m "test(eval): CZ atomization fixtures + RUN_LLM_EVAL=1 gated runner"
```

---

## Task 14: M1 acceptance smoke + tag

- [ ] **Step 1: Run the M1 smoke checklist**

```bash
pnpm tauri dev
```

With a valid Anthropic key set:

1. Paste a short CZ paragraph mixing fact, opinion, and inference. Click Analyze. Within ~4s, colored highlights appear.
2. Click each colored span. Side panel updates with the verbatim quote and the Czech reason.
3. Empty input: Analyze button is disabled.
4. Paste >25 claims worth of text. Truncation banner appears.
5. Clear the Anthropic key in Settings. Click Analyze. The summary shows `Nastav nejdřív Anthropic API klíč…`.
6. Restore the key. Click Analyze again. Works.
7. Press the global hotkey from another app with a sentence in clipboard. The window focuses, text pre-fills, Analyze becomes active.

- [ ] **Step 2: Run lint+test pass**

```bash
pnpm check && pnpm lint && pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all green.

- [ ] **Step 3: Tag**

```bash
git tag m1-classification
```

- [ ] **Step 4: Move on to M2**

Open `2026-05-20-druhy-nazor-03-verification.md`.
