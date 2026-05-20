# Druhý názor — MVP Overview & Architecture

> **For agentic workers:** This is the master overview document. It defines architecture, file structure, conventions, and links to per-phase plan files. To implement, read this first, then follow the phase plans in order (`01-foundation.md` → `02-classification.md` → `03-verification.md`).

**Goal:** Ship a working desktop MVP (macOS + Windows) of _Druhý názor_ — a verification layer for AI responses. User pastes an AI response, the app atomizes it into claims, classifies each claim by epistemic type (verifiable fact / inference / opinion / contradiction), verifies factual claims against the open web, and presents the result with color-coded highlights and clickable sources in Czech.

**Architecture:** Native desktop app built with **Tauri 2.x**. Rust backend handles secure storage, hotkey/tray, HTTP, SQLite cache, and the verification pipeline. **Svelte 5 + TypeScript** frontend renders the analysis UI. The pipeline is: capture → LLM atomization+classification (one structured call) → parallel web search verification per fact-claim → cached, color-annotated result. The app is source-agnostic: any AI response can be analyzed via paste, drag&drop, or global hotkey + clipboard read.

**Tech Stack:**

- Tauri 2.x, Rust (tokio, reqwest, serde, rusqlite, keyring)
- Svelte 5 + TypeScript + Vite
- Anthropic Claude Haiku 4.5 (only LLM provider in MVP; BYO API key)
- Brave Search API (verification source discovery)
- `readability` crate (article body extraction)
- `tauri-plugin-global-shortcut` (hotkey)
- `tauri-plugin-store` (non-secret settings)
- `keyring` crate (OS-native secret storage)

---

## 1. MVP Scope

### In MVP Core (this set of plans):

- Tauri shell, macOS + Windows builds (developer signing only; production signing in a later plan).
- Three capture paths: paste into input, drag&drop, global hotkey that brings the app to focus and pre-fills from clipboard.
- LLM-driven atomization of an AI response into discrete claims.
- Classification of each claim into one of four buckets: `fact`, `inference`, `opinion`, `contradiction`.
- Parallel web search verification of fact-claims via Brave Search.
- Source body extraction and a hardcoded source-tier scoring heuristic.
- Verification verdict per claim: `supported`, `contradicted`, `no_consensus`, `not_found`.
- Color-coded UI with clickable per-claim side panel showing extracted sources.
- SQLite-backed cache keyed by claim hash (TTL 7 days).
- BYO API key flow: user enters Anthropic + Brave keys in settings; stored in OS keychain.
- Bilingual UI — Czech and English, both fully translated. The app auto-detects the OS locale at first launch (`cs-*` → Czech; anything else → English) and lets the user switch at any time in Settings. Adding more languages is a matter of dropping a new bundle in `src/lib/i18n/` and the matching prompt files in `src-tauri/src/llm/prompts/`.
- Per-locale LLM prompts (atomize + judge), per-locale Brave search hints, and per-locale verification verdict text. The Czech-speaking laic remains the primary target; English support widens the addressable audience and gives international contributors a working surface.
- Streaming UI: claims appear and color in as soon as classification arrives; sources fill in per-claim asynchronously.

### NOT in MVP Core (later plans):

- Onboarding flow polish, privacy disclosures, pre-send confirmation modal — covered in `04-privacy-polish.md` (to be written after Core lands).
- Production code signing, notarization, auto-updater, GitHub Actions release pipeline — covered in `05-distribution.md` (to be written after Polish lands).
- Public beta and 1.0 release process — covered in `06-release.md`.
- OpenAI / Mistral provider support.
- Ollama local mode.
- Companion browser extension.
- OCR / screenshot capture.
- History view with full-text search (basic history storage IS in Core; UI is in Polish).
- Citation back-export ("zkopírovat námitku").
- Mac App Store / Microsoft Store distribution.
- Telemetry of any kind.

---

## 2. Repository Layout

The whole project lives at the working directory root (`/Users/lukasoplt/Documents/Druhý názor/`). The Tauri convention is two top-level source trees: `src-tauri/` (Rust) and `src/` (frontend). Docs and plans live alongside.

```
druhy-nazor/                          # repo root
├── README.md
├── LICENSE                           # Apache-2.0
├── .gitignore
├── .editorconfig
├── package.json                      # frontend deps + scripts
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── .github/
│   └── workflows/
│       ├── ci.yml                    # lint + test on PR
│       └── build.yml                 # cross-platform dev builds on main
├── docs/
│   ├── ARCHITECTURE.md
│   ├── PRIVACY.md                    # written in Polish phase
│   └── superpowers/
│       └── plans/
│           ├── 2026-05-20-druhy-nazor-00-overview.md   ← this file
│           ├── 2026-05-20-druhy-nazor-01-foundation.md
│           ├── 2026-05-20-druhy-nazor-02-classification.md
│           └── 2026-05-20-druhy-nazor-03-verification.md
│
├── src-tauri/                        # Rust backend
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                        # placeholder icons in M0
│   ├── capabilities/
│   │   └── default.json              # Tauri 2 capability manifest
│   ├── migrations/
│   │   └── 001_init.sql
│   └── src/
│       ├── main.rs                   # entry: tauri::Builder
│       ├── lib.rs                    # re-exports for tests
│       ├── error.rs                  # AppError, AppResult
│       ├── models.rs                 # shared types: Claim, Analysis, ...
│       ├── commands/                 # Tauri IPC handlers
│       │   ├── mod.rs
│       │   ├── analysis.rs           # analyze_text, get_analysis
│       │   ├── settings.rs           # get/set settings, set/clear API key
│       │   ├── capture.rs            # read_clipboard, focus_window
│       │   └── history.rs            # list/get/delete history
│       ├── llm/
│       │   ├── mod.rs                # LlmProvider trait
│       │   ├── anthropic.rs          # Anthropic Messages API client
│       │   ├── mock.rs               # in-memory provider for tests
│       │   └── prompts/
│       │       ├── mod.rs
│       │       ├── atomize_cs.txt    # CZ atomization+classification prompt
│       │       └── judge_cs.txt      # CZ source-judging prompt
│       ├── pipeline/
│       │   ├── mod.rs                # AnalysisPipeline orchestrator
│       │   ├── atomize.rs            # atomization service
│       │   ├── verify.rs             # per-claim verification
│       │   └── source_tier.rs        # A/B/C/D tier scoring
│       ├── search/
│       │   ├── mod.rs                # SearchProvider trait
│       │   ├── brave.rs              # Brave Search API client
│       │   └── extract.rs            # readability extraction
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── db.rs                 # SQLite connection pool
│       │   ├── cache.rs              # verification cache
│       │   ├── history.rs            # analysis history
│       │   ├── settings_store.rs     # non-secret kv store
│       │   └── keychain.rs           # OS keychain wrapper
│       ├── hotkey.rs                 # global shortcut registration
│       ├── tray.rs                   # system tray menu
│       └── tests/
│           ├── mod.rs
│           └── eval.rs               # CZ AI response eval suite
│
└── src/                              # Svelte 5 frontend
    ├── app.html
    ├── app.css                       # global tokens, dark mode vars
    ├── routes/
    │   ├── +layout.svelte            # shell, locale, tray-driven nav
    │   ├── +page.svelte              # main analysis screen
    │   ├── settings/
    │   │   └── +page.svelte
    │   └── history/
    │       └── +page.svelte
    └── lib/
        ├── api.ts                    # typed Tauri invoke wrappers
        ├── types.ts                  # mirrors Rust models
        ├── stores/
        │   ├── analysis.svelte.ts    # Svelte 5 runes
        │   ├── settings.svelte.ts
        │   └── i18n.svelte.ts
        ├── i18n/
        │   ├── cs.json
        │   └── en.json
        └── components/
            ├── PasteInput.svelte
            ├── ClaimText.svelte
            ├── SidePanel.svelte
            ├── SourceCard.svelte
            ├── TierBadge.svelte
            ├── ProgressDots.svelte
            └── ConfirmModal.svelte
```

---

## 3. Conventions

### Rust

- Edition 2021. MSRV 1.78.
- Error handling: `anyhow::Result` at command boundaries, `thiserror`-derived `AppError` for domain errors.
- Async runtime: tokio (provided by Tauri).
- Logging: `tracing` + `tracing-subscriber` (default `INFO`, opt-in `DEBUG` via env).
- Crate naming: snake_case module names matching file names.
- Lints: `#![warn(clippy::pedantic)]` at workspace level, allow common pedantic noise per-file as needed.

### TypeScript / Svelte

- Strict TS (`"strict": true`, no `any` unless justified by comment).
- Svelte 5 runes (`$state`, `$derived`, `$effect`).
- No global stores beyond what's in `src/lib/stores/`. Components consume stores via `$derived` to remain testable.
- File naming: PascalCase for components, camelCase for stores/utils.
- All UI text is pulled from `i18n/cs.json` via a `t()` helper; no hardcoded Czech strings inside components.

### Git workflow

- Trunk-based: `main` is always green.
- Conventional Commits prefixes: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`.
- Each task in a phase plan ends with a commit. Tasks land squashed when reviewed.

### Code style

- Frontend: `prettier` + `eslint --max-warnings=0`.
- Rust: `cargo fmt` + `cargo clippy --all-targets -- -D warnings`.

---

## 4. Cross-Cutting Decisions

These decisions are referenced by individual tasks across the phase plans. Locked in here so they aren't re-debated mid-implementation.

| Topic                    | Decision                                                                                                                                                              | Rationale                                                                                                                     |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| LLM provider in MVP      | Anthropic Claude Haiku 4.5 only                                                                                                                                       | Best CZ quality at lowest latency/cost. Other providers behind a trait, added later.                                          |
| Default analysis model   | `claude-haiku-4-5-20251001`                                                                                                                                           | Cheap, fast, CZ-fluent.                                                                                                       |
| LLM output format        | JSON via Anthropic tool-use schema                                                                                                                                    | Strict schema, validated server-side, no parser hacks.                                                                        |
| Verification judge model | Same as analysis (Haiku 4.5)                                                                                                                                          | One model, one prompt-cache key prefix.                                                                                       |
| Search provider          | Brave Search API only in MVP                                                                                                                                          | One paid dep is enough. DDG fallback is a later plan.                                                                         |
| Max claims per analysis  | 25                                                                                                                                                                    | Hard cap. Anything longer is truncated with a UI warning. Avoids runaway cost.                                                |
| Verification cap         | 8 fact-claims per analysis                                                                                                                                            | First 8 in document order. Rest get `not_verified` state.                                                                     |
| Cache TTL                | 7 days                                                                                                                                                                | Balances cost with content freshness.                                                                                         |
| Selection capture        | Clipboard-read on hotkey (no Accessibility API)                                                                                                                       | Real accessibility is a permissions/UX rabbit hole. Clipboard hack works everywhere.                                          |
| Hotkey                   | `CommandOrControl+Shift+D`                                                                                                                                            | Documented and remappable in Settings (remap UI in Polish phase).                                                             |
| Storage path             | Tauri app data dir                                                                                                                                                    | OS-default location. Single SQLite file `cache.db`.                                                                           |
| Encryption at rest       | Plain SQLite in MVP                                                                                                                                                   | OS already protects user home dir. sqlcipher is a Polish-phase upgrade.                                                       |
| API keys at rest         | OS keychain via `keyring` crate                                                                                                                                       | macOS Keychain, Windows Credential Manager.                                                                                   |
| Telemetry                | None in MVP                                                                                                                                                           | Privacy-first. Crash reporting in a later plan.                                                                               |
| Localization             | Bilingual UI (cs, en) with OS-locale default; LLM prompts per locale; Brave search hints per locale; new languages added by dropping a JSON bundle plus prompt files. | Czech-speaking laic is the primary user; English support is needed for international AI responses and contributor onboarding. |
| Locale detection         | `sys-locale` crate at first launch maps `cs-*` → cs, else en. User can override in Settings; preference persists in `tauri-plugin-store`.                             | Pure-Rust, cross-platform, no plugin churn.                                                                                   |
| License                  | Apache-2.0                                                                                                                                                            | Patent grant matters for an open-source verification tool.                                                                    |

---

## 5. Shared Data Model

These types are defined once in `src-tauri/src/models.rs` and re-emitted to TypeScript via `src/lib/types.ts`. Phase 1 (foundation) creates the empty structs; Phase 2 fills atomization fields; Phase 3 fills verification fields.

```rust
// src-tauri/src/models.rs

use serde::{Deserialize, Serialize};

/// A single atomic claim extracted from an AI response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    /// Stable id within the analysis ("c1", "c2", ...).
    pub id: String,
    /// Verbatim text span as it appears in the source.
    pub text: String,
    /// 0-based char offsets in the original response text.
    pub start: usize,
    pub end: usize,
    /// Epistemic classification.
    pub kind: ClaimKind,
    /// LLM-provided short justification for the classification, in Czech.
    pub reason: String,
    /// Verification result. None until verification runs (or skipped).
    pub verification: Option<Verification>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Fact,
    Inference,
    Opinion,
    Contradiction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Verification {
    pub status: VerificationStatus,
    pub sources: Vec<SourceHit>,
    /// Short Czech explanation of the verdict.
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Supported,
    Contradicted,
    NoConsensus,
    NotFound,
    NotVerified, // claim was not eligible (e.g. capped)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceHit {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub tier: SourceTier,
    pub stance: SourceStance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceTier { A, B, C, D }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStance { Supports, Contradicts, Mentions }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: String,        // UUIDv7
    pub created_at: i64,   // unix millis
    pub input: String,     // original AI response text
    pub claims: Vec<Claim>,
    pub truncated: bool,   // true if input was over the cap
}
```

The TypeScript mirror lives in `src/lib/types.ts` and is regenerated by hand whenever `models.rs` changes (a 10-line discipline, not a code-gen tool, in MVP).

---

## 6. Testing Strategy

| Layer            | Tool                                  | Trigger          | Notes                                                                                  |
| ---------------- | ------------------------------------- | ---------------- | -------------------------------------------------------------------------------------- |
| Rust unit tests  | `cargo test`                          | every commit     | Per-module tests next to the code in `#[cfg(test)] mod tests`.                         |
| Rust integration | `cargo test --test integration`       | CI on PR         | Spawns Tauri-less services with mock LLM and mock search.                              |
| LLM eval         | `cargo test --test eval -- --ignored` | nightly + manual | Runs a curated CZ fixture set through the real Haiku model. Gated by `RUN_LLM_EVAL=1`. |
| Frontend unit    | `vitest`                              | every commit     | Component logic with `@testing-library/svelte`.                                        |
| End-to-end       | None in MVP                           | —                | Manual smoke checklist before each phase merge.                                        |

**LLM eval fixtures** live in `src-tauri/tests/eval/fixtures/*.json`. Each fixture is:

```json
{
  "name": "karel-iv-narozeni",
  "input": "Karel IV. se narodil v roce 1316 v Praze a založil pražskou univerzitu...",
  "expected_min_claims": 3,
  "expected_kinds": { "fact": ">=3" },
  "must_classify_as_fact": [
    "Karel IV. se narodil v roce 1316.",
    "Karel IV. založil pražskou univerzitu."
  ]
}
```

Eval scoring is precision/recall on `must_classify_as_fact` plus a sanity check that `expected_min_claims` is met. Threshold for passing: 80% of fixtures pass. Below threshold, prompt needs revision.

**Mock-first development:** Every external dependency (LLM, search, source fetch) has a `Mock*` implementation that returns canned responses. Integration tests always use mocks. Real APIs are only hit in the eval suite and manual smoke runs.

---

## 7. Phase Plan Map

The MVP Core is split into three sequential phase plans. Each phase ships an end-to-end-runnable state (even if the UI shows placeholder data in early phases).

| Phase                   | Plan file                                     | Output                                                                                                                                                                        |
| ----------------------- | --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0 — Foundation**     | `2026-05-20-druhy-nazor-01-foundation.md`     | A signed-during-dev Tauri app that opens, accepts pasted text, persists settings, registers a global hotkey, and shows the system tray. No analysis yet.                      |
| **M1 — Classification** | `2026-05-20-druhy-nazor-02-classification.md` | App calls Anthropic with the user's key, atomizes the input into claims, classifies each, streams results to the UI, and renders color-coded highlights. No verification yet. |
| **M2 — Verification**   | `2026-05-20-druhy-nazor-03-verification.md`   | Fact-claims trigger Brave searches in parallel. Results are body-extracted, tier-scored, and judged. Per-claim side panel shows sources and verdict. SQLite cache populated.  |

After M2 ships and is dogfooded for at least a week, the following phase plans will be written (one at a time, never speculatively):

- `04-privacy-polish.md` — pre-send confirmation modal, onboarding screens, error states, history UI, hotkey remapping, accessibility (VoiceOver/NVDA), high-contrast mode.
- `05-distribution.md` — Apple Developer ID signing + notarization, Windows EV code signing, Tauri auto-updater hosted on GitHub Releases, landing page.
- `06-release.md` — private beta, public 1.0 release process, crash reporting opt-in.

These are explicitly out of scope here and not stubbed in advance — the skill's "no placeholders" rule applies.

---

## 8. Spec Coverage Check

Mapping each MVP blueprint requirement to its task in the phase plans:

| Blueprint requirement                     | Covered in                                                                                                                                                                                                                          |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop app for macOS + Windows           | M0 Task 1 (Tauri init), M0 Task 4 (cross-platform CI matrix)                                                                                                                                                                        |
| Source-agnostic input (paste/drag/hotkey) | M0 Task 8 (paste flow), M0 Task 9 (global hotkey + clipboard read)                                                                                                                                                                  |
| Atomization of AI response into claims    | M1 Task 4 (atomization prompt), M1 Task 5 (parser+validator)                                                                                                                                                                        |
| Four-bucket classification                | M1 Task 4 (same prompt covers it)                                                                                                                                                                                                   |
| Color-coded UI annotation                 | M1 Task 8 (ClaimText component)                                                                                                                                                                                                     |
| Web search verification of fact-claims    | M2 Task 1 (Brave client), M2 Task 4 (verification orchestrator)                                                                                                                                                                     |
| Source body extraction                    | M2 Task 2 (readability extraction)                                                                                                                                                                                                  |
| Source tier scoring                       | M2 Task 3 (tier heuristic)                                                                                                                                                                                                          |
| Per-claim side panel with sources         | M2 Task 6 (SidePanel component)                                                                                                                                                                                                     |
| BYO API key flow                          | M0 Task 5 (keychain wrapper), M0 Task 7 (Settings page)                                                                                                                                                                             |
| Czech + English UI with OS-locale default | M0 Task 3 (sys-locale dep), Task 6 (`with_system_locale` constructor), Task 7 (`get_settings` returns OS-detected when store is empty), Task 9 (parity bundles `cs.json` + `en.json`); every component task pulls strings via `t()` |
| Locale-aware LLM prompts                  | M1 Task 3 (`atomize_cs.txt` + `atomize_en.txt` + `atomize_prompt(locale)`), M2 Task 1 (`judge_cs.txt` + `judge_en.txt` + `judge_prompt(locale)`), provider carries `locale` field                                                   |
| Locale-aware web search                   | M2 Task 2 (`BraveClient` derives `country` and `search_lang` from locale)                                                                                                                                                           |
| Locale-aware verdict summary              | M2 Task 7 (`summarize(locale, status, hits)` with cs/en branches)                                                                                                                                                                   |
| Streaming UI as results arrive            | M1 Task 7 (Tauri events), M2 Task 5 (per-claim verify events)                                                                                                                                                                       |
| SQLite cache with TTL                     | M2 Task 7 (cache schema), M2 Task 8 (cache lookup)                                                                                                                                                                                  |
| System tray                               | M0 Task 10                                                                                                                                                                                                                          |
| OS-native key storage                     | M0 Task 5                                                                                                                                                                                                                           |
| Hard cap on claims and verifications      | M1 Task 4 (claim cap in prompt), M2 Task 4 (verify cap in orchestrator)                                                                                                                                                             |
| Eval suite for prompt regression          | M1 Task 11 (eval fixtures + runner)                                                                                                                                                                                                 |

If any blueprint requirement is missing here, the phase plan needs a new task added.

---

## 9. Glossary

- **Claim** — one atomic factual or non-factual proposition extracted from the AI response.
- **Atomization** — the LLM-driven step that splits the response into Claims.
- **Classification** — labeling each Claim with one of `fact / inference / opinion / contradiction`.
- **Verification** — the search + judge step that decides whether a fact-claim is `supported / contradicted / no_consensus / not_found` based on retrieved sources.
- **Tier** — A/B/C/D quality label on a source, computed from a hardcoded domain heuristic plus title cues. A = primary/authoritative, D = social/spam.
- **No consensus** — the verifier found sources on both sides without a clear A-tier resolution. UI shows this distinct from "not found".
- **BYO key** — user provides their own Anthropic and Brave API keys in settings.
