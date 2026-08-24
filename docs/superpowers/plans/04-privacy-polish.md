# Privacy & Polish (M3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the working MVP Core into a shippable, privacy-legible, accessible app: the user always knows what leaves their machine before it leaves, first-run explains the model, failures are actionable instead of raw strings, past analyses are browsable and deletable, the global hotkey is remappable from the UI, and the whole surface passes a VoiceOver/NVDA + contrast audit.

**Architecture:** No new subsystems. Four seams get widened. (1) `AppError` gains a machine-readable `code` so the frontend can localize a remedy instead of printing a Rust string. (2) The already-written-but-never-read `analysis_history` table gets `list/get/delete/clear/prune` in `storage/history.rs`, Tauri commands in `commands/history.rs`, typed wrappers in `src/lib/api.ts`, and a `/history` route — closing the dead path where the backend writes rows nobody can see. (3) Two new boolean/optional settings (`confirm_before_send`, `high_contrast`, `history_retention_days`) flow through the existing `Settings` struct, its serde defaults, and the Settings page. (4) Presentational work — modal, onboarding, error surface, hotkey recorder — lands as Svelte 5 components backed by *pure, unit-tested modules* in `src/lib/` so the logic is testable without a DOM.

**Tech Stack:** Tauri 2.x, Rust (rusqlite, serde, thiserror, tauri-plugin-global-shortcut), Svelte 5 runes + TypeScript, Vitest + @testing-library/svelte, `cargo test`.

## Global Constraints

- Rust edition 2021, MSRV 1.78. `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean.
- TypeScript strict; no `any` without a justifying comment. Svelte 5 runes only (`$state`, `$derived`, `$props`, `$bindable`).
- No hardcoded user-facing strings in components. Every string goes through `t()` / `tf()` and exists in **both** `src/lib/i18n/cs.json` and `src/lib/i18n/en.json`. Key parity is enforced by a test (Task 8).
- `prettier --check .` and `eslint --max-warnings=0` clean (`pnpm lint`).
- Zero telemetry. The only outbound hosts remain the three already in the CSP: `api.anthropic.com`, `api.search.brave.com`, `api.github.com`.
- Secrets stay in the OS keychain via `keyring`. Never log, never render, never persist a key to the settings store.
- Locales: exactly `cs` and `en`. Czech is the primary audience; English text must read as native English, not a translation.
- Accessibility target: **WCAG 2.1 AA** — body text contrast ≥ 4.5:1, large text (≥24px, or ≥18.66px bold) ≥ 3:1, every interactive element reachable and operable by keyboard, visible focus, no information conveyed by color alone.
- Conventional Commits. One commit per task, at the end of the task.
- Nothing in this plan performs code signing, notarization, or any release action. Task 9 produces *handover documentation only*.

---

## File Structure

**Rust — new responsibilities**

| File | Responsibility |
| --- | --- |
| `src-tauri/src/error.rs` (modify) | Adds `ErrorCode` enum, `AppError::Provider { code, detail }`, `AppError::code()`, and struct serialization `{code, message}`. |
| `src-tauri/src/llm/cli.rs` (modify) | Maps spawn/timeout/exit/parse failures onto `ErrorCode::CliNotFound / CliTimeout / CliFailed / CliBadOutput`. |
| `src-tauri/src/llm/anthropic.rs` (modify) | Maps transport + HTTP status onto `ErrorCode::Network / LlmAuth / LlmRateLimit / LlmHttp`. |
| `src-tauri/src/search/brave.rs` (modify) | Maps transport + HTTP status onto `ErrorCode::Network / SearchAuth / SearchRateLimit / SearchHttp`. |
| `src-tauri/src/storage/history.rs` (modify) | `HistoryEntry`, `list`, `get`, `delete`, `clear`, `prune` alongside the existing `insert`. |
| `src-tauri/src/commands/history.rs` (replace) | Tauri commands `list_history`, `get_analysis`, `delete_analysis`, `clear_history`. |
| `src-tauri/src/storage/settings_store.rs` (modify) | Three new fields: `confirm_before_send`, `high_contrast`, `history_retention_days` + validation + serde defaults. |
| `src-tauri/src/hotkey.rs` (modify) | `normalize()` for accelerator validation, `reinstall()` for runtime re-registration. |
| `src-tauri/src/commands/settings.rs` (modify) | Validates and re-registers the hotkey when it changes. |
| `src-tauri/src/lib.rs` (modify) | Registers the four history commands; prunes history on startup. |

**Frontend — new pure modules (logic, unit-tested without a DOM)**

| File | Responsibility |
| --- | --- |
| `src/lib/errors.ts` | `AppErrorPayload`, `toAppError()`, `isSettingsError()`, `errorKey()`. |
| `src/lib/sendSummary.ts` | `describeSend()` — what exactly leaves the machine for this analysis. |
| `src/lib/onboarding.ts` | Step list + guard predicates for the first-run flow. |
| `src/lib/hotkey.ts` | `acceleratorFromEvent()`, `formatAccelerator()`, `isModifierOnly()`. |
| `src/lib/history.ts` | `formatHistoryDate()`, `historyPreview()`. |
| `src/lib/contrast.ts` | `parseHex()`, `relativeLuminance()`, `contrastRatio()` — used by the contrast regression test. |

**Frontend — new components**

| File | Responsibility |
| --- | --- |
| `src/lib/components/ConfirmModal.svelte` | Generic accessible dialog: `role="dialog"`, `aria-modal`, focus trap, Escape, focus restore. |
| `src/lib/components/SendConfirm.svelte` | Pre-send disclosure body rendered inside `ConfirmModal`. |
| `src/lib/components/Onboarding.svelte` | Four-step first-run overlay. |
| `src/lib/components/ErrorState.svelte` | Localized, actionable error surface (replaces `alert()` and the raw-string banner). |
| `src/lib/components/HotkeyInput.svelte` | Key-capture recorder for the global hotkey. |
| `src/routes/history/+page.svelte` | History list, search, open, delete, clear-all. |

---

## Task 1: Structured error codes end-to-end

Today every backend failure reaches the UI as a Rust `Display` string (`"cli 'claude' exit 1: …"`), the analysis page renders it verbatim behind `Chyba: {msg}`, and preflight failures use `alert()`. This task gives every error a stable code so the UI can say *what to do* in the user's language, and keeps the raw detail as collapsible diagnostics.

**Files:**
- Modify: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/llm/cli.rs`
- Modify: `src-tauri/src/llm/anthropic.rs`
- Modify: `src-tauri/src/search/brave.rs`
- Create: `src/lib/errors.ts`
- Create: `src/lib/errors.test.ts`
- Create: `src/lib/components/ErrorState.svelte`
- Modify: `src/lib/stores/analysis.svelte.ts`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: nothing from later tasks.
- Produces:
  - Rust: `crate::error::ErrorCode` (serde `snake_case`), `AppError::provider(code, detail) -> AppError`, `AppError::code(&self) -> ErrorCode`. `AppError` now serializes as `{"code": "cli_not_found", "message": "…"}` instead of a bare string — **every** command's rejection value changes shape.
  - TS: `src/lib/errors.ts` exports `type ErrorCode`, `interface AppErrorPayload { code: ErrorCode; message: string }`, `toAppError(caught: unknown): AppErrorPayload`, `isSettingsError(code: ErrorCode): boolean`, `errorKey(code: ErrorCode): string`.
  - Svelte: `ErrorState.svelte` props `{ error: AppErrorPayload; onRetry?: () => void; onSettings?: () => void }`.
  - `analysisStore.error` changes type from `string | null` to `AppErrorPayload | null`.

- [ ] **Step 1: Write the failing Rust tests for error codes**

Append to `src-tauri/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_error_reports_its_code() {
        let error = AppError::provider(ErrorCode::CliNotFound, "claude not on PATH");
        assert_eq!(error.code(), ErrorCode::CliNotFound);
        assert_eq!(error.to_string(), "claude not on PATH");
    }

    #[test]
    fn plain_variants_map_to_stable_codes() {
        assert_eq!(AppError::Invalid("x".into()).code(), ErrorCode::Invalid);
        assert_eq!(AppError::NotFound("x".into()).code(), ErrorCode::NotFound);
        assert_eq!(AppError::Store("x".into()).code(), ErrorCode::Store);
        assert_eq!(AppError::Other("x".into()).code(), ErrorCode::Other);
    }

    #[test]
    fn serializes_as_code_and_message() {
        let error = AppError::provider(ErrorCode::LlmRateLimit, "429 slow down");
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "llm_rate_limit");
        assert_eq!(json["message"], "429 slow down");
    }

    #[test]
    fn every_code_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(ErrorCode::CliBadOutput).unwrap(),
            "cli_bad_output"
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::SearchRateLimit).unwrap(),
            "search_rate_limit"
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test error::tests`
Expected: FAIL — `cannot find type ErrorCode in this scope`.

- [ ] **Step 3: Implement `ErrorCode` and the new serialization**

Replace the body of `src-tauri/src/error.rs` above the test module with:

```rust
use serde::ser::SerializeStruct;
use serde::Serialize;
use thiserror::Error;

/// Stable, machine-readable classification of a failure. The frontend maps
/// each code to a localized sentence plus a remedy; the human-readable
/// `message` is kept only as collapsible diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The configured CLI program is not on PATH / not executable.
    CliNotFound,
    /// The CLI ran but exited non-zero.
    CliFailed,
    /// The CLI exceeded `CLI_TIMEOUT`.
    CliTimeout,
    /// The CLI produced output we could not read as a JSON object.
    CliBadOutput,
    /// The LLM API rejected our credentials (401/403).
    LlmAuth,
    /// The LLM API rate-limited us (429).
    LlmRateLimit,
    /// Any other non-success status from the LLM API.
    LlmHttp,
    /// The search API rejected our credentials (401/403).
    SearchAuth,
    /// The search API rate-limited us (429).
    SearchRateLimit,
    /// Any other non-success status from the search API.
    SearchHttp,
    /// Transport-level failure: DNS, TLS, connection refused, timeout.
    Network,
    Keychain,
    Store,
    Io,
    Serde,
    Tauri,
    Hotkey,
    NotFound,
    Invalid,
    Other,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("keychain error: {0}")]
    Keychain(#[from] keyring::Error),

    #[error("store error: {0}")]
    Store(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("global shortcut error: {0}")]
    GlobalShortcut(#[from] tauri_plugin_global_shortcut::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    /// A classified provider failure. `detail` is raw diagnostics — it may
    /// contain a CLI's stderr and is shown only behind a disclosure control.
    #[error("{detail}")]
    Provider { code: ErrorCode, detail: String },

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    #[must_use]
    pub fn provider(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self::Provider {
            code,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Provider { code, .. } => *code,
            Self::Keychain(_) => ErrorCode::Keychain,
            Self::Store(_) => ErrorCode::Store,
            Self::Io(_) => ErrorCode::Io,
            Self::Serde(_) => ErrorCode::Serde,
            Self::Tauri(_) => ErrorCode::Tauri,
            Self::GlobalShortcut(_) => ErrorCode::Hotkey,
            Self::NotFound(_) => ErrorCode::NotFound,
            Self::Invalid(_) => ErrorCode::Invalid,
            Self::Other(_) => ErrorCode::Other,
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test error::tests`
Expected: PASS (4 tests).

- [ ] **Step 5: Write failing tests for CLI error classification**

Add to the `tests` module in `src-tauri/src/llm/cli.rs`:

```rust
    #[tokio::test]
    async fn missing_program_reports_cli_not_found() {
        let provider =
            CliProvider::new("prove-definitely-not-a-real-binary", "en".into()).unwrap();
        let error = provider.atomize("x").await.unwrap_err();
        assert_eq!(error.code(), crate::error::ErrorCode::CliNotFound);
    }

    #[tokio::test]
    async fn nonzero_exit_reports_cli_failed() {
        let provider = CliProvider::new("sh -c 'exit 7'", "en".into()).unwrap();
        let error = provider.atomize("x").await.unwrap_err();
        assert_eq!(error.code(), crate::error::ErrorCode::CliFailed);
    }

    #[tokio::test]
    async fn unparseable_output_reports_cli_bad_output() {
        let cmd = "sh -c 'cat >/dev/null; printf %s \"no json here\"'";
        let provider = CliProvider::new(cmd, "en".into()).unwrap();
        let error = provider.atomize("x").await.unwrap_err();
        assert_eq!(error.code(), crate::error::ErrorCode::CliBadOutput);
    }
```

- [ ] **Step 6: Run them to verify they fail**

Run: `cd src-tauri && cargo test llm::cli::tests`
Expected: FAIL — the three new tests report `ErrorCode::Other`.

- [ ] **Step 7: Classify CLI failures**

In `src-tauri/src/llm/cli.rs`, add `use crate::error::ErrorCode;` to the imports and replace the error construction sites inside `run()`:

```rust
            let mut child = cmd.spawn().map_err(|e| {
                let code = if e.kind() == std::io::ErrorKind::NotFound {
                    ErrorCode::CliNotFound
                } else {
                    ErrorCode::CliFailed
                };
                AppError::provider(code, format!("cli spawn '{}': {e}", self.command[0]))
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(prompt.as_bytes()).await.map_err(|e| {
                    AppError::provider(ErrorCode::CliFailed, format!("cli stdin write: {e}"))
                })?;
                stdin.shutdown().await.map_err(|e| {
                    AppError::provider(ErrorCode::CliFailed, format!("cli stdin close: {e}"))
                })?;
            }

            let output = timeout(CLI_TIMEOUT, child.wait_with_output())
                .await
                .map_err(|_| {
                    AppError::provider(
                        ErrorCode::CliTimeout,
                        format!("cli timeout after {}s", CLI_TIMEOUT.as_secs()),
                    )
                })?
                .map_err(|e| {
                    AppError::provider(ErrorCode::CliFailed, format!("cli wait: {e}"))
                })?;

            if !output.status.success() {
                return Err(AppError::provider(
                    ErrorCode::CliFailed,
                    format!(
                        "cli '{}' exit {}: {}",
                        self.command[0],
                        output.status,
                        String::from_utf8_lossy(&output.stderr).trim()
                    ),
                ));
            }
```

`resolve_program` falls back to `PathBuf::from(&self.command[0])` when nothing on PATH matches, so a missing binary still reaches `spawn()` and surfaces as `ErrorKind::NotFound`. Keep that fallback.

Then in `parse_cli_json_object`, swap all three `AppError::Other(...)` for `AppError::provider(ErrorCode::CliBadOutput, ...)`, and in `parse_kind` / `RawJudge::into_verdict` swap `AppError::Other(...)` for `AppError::provider(ErrorCode::CliBadOutput, ...)`.

- [ ] **Step 8: Run the CLI tests**

Run: `cd src-tauri && cargo test llm::cli::tests`
Expected: PASS (all pre-existing tests still green).

- [ ] **Step 9: Classify HTTP failures for Anthropic and Brave**

In `src-tauri/src/llm/anthropic.rs`, add `use crate::error::ErrorCode;` and rewrite `post()`'s error paths:

```rust
        let response = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|error| {
                AppError::provider(ErrorCode::Network, format!("anthropic http: {error}"))
            })?;

        let status = response.status();
        let text = response.text().await.map_err(|error| {
            AppError::provider(ErrorCode::Network, format!("anthropic body: {error}"))
        })?;

        if !status.is_success() {
            return Err(AppError::provider(
                http_error_code(status.as_u16(), ErrorCode::LlmAuth, ErrorCode::LlmRateLimit, ErrorCode::LlmHttp),
                format!("anthropic {status}: {text}"),
            ));
        }
```

Add the shared helper to `src-tauri/src/error.rs` (above the test module):

```rust
/// Maps an HTTP status onto the auth / rate-limit / generic triple a caller
/// supplies, so the LLM and search clients classify statuses identically.
#[must_use]
pub fn http_error_code(
    status: u16,
    auth: ErrorCode,
    rate_limit: ErrorCode,
    other: ErrorCode,
) -> ErrorCode {
    match status {
        401 | 403 => auth,
        429 => rate_limit,
        _ => other,
    }
}
```

Import it in both clients (`use crate::error::{http_error_code, AppError, AppResult, ErrorCode};`) and apply the same shape in `src-tauri/src/search/brave.rs`:

```rust
        if !status.is_success() {
            return Err(AppError::provider(
                http_error_code(
                    status.as_u16(),
                    ErrorCode::SearchAuth,
                    ErrorCode::SearchRateLimit,
                    ErrorCode::SearchHttp,
                ),
                format!("brave {status}: {body}"),
            ));
        }
```

with `ErrorCode::Network` for both `send()` and `text()` failures.

Add to `src-tauri/src/error.rs`'s test module:

```rust
    #[test]
    fn http_status_maps_to_auth_rate_limit_or_generic() {
        assert_eq!(
            http_error_code(401, ErrorCode::LlmAuth, ErrorCode::LlmRateLimit, ErrorCode::LlmHttp),
            ErrorCode::LlmAuth
        );
        assert_eq!(
            http_error_code(429, ErrorCode::SearchAuth, ErrorCode::SearchRateLimit, ErrorCode::SearchHttp),
            ErrorCode::SearchRateLimit
        );
        assert_eq!(
            http_error_code(500, ErrorCode::LlmAuth, ErrorCode::LlmRateLimit, ErrorCode::LlmHttp),
            ErrorCode::LlmHttp
        );
    }
```

- [ ] **Step 10: Run the full Rust suite**

Run: `cd src-tauri && cargo test`
Expected: PASS.

- [ ] **Step 11: Write the failing frontend error-mapping test**

Create `src/lib/errors.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { errorKey, isSettingsError, toAppError } from './errors';

describe('toAppError', () => {
  it('passes through a structured Tauri rejection', () => {
    expect(toAppError({ code: 'cli_not_found', message: "cli spawn 'claude'" })).toEqual({
      code: 'cli_not_found',
      message: "cli spawn 'claude'",
    });
  });

  it('falls back to `other` for an unknown code', () => {
    expect(toAppError({ code: 'wat', message: 'boom' })).toEqual({
      code: 'other',
      message: 'boom',
    });
  });

  it('wraps a thrown Error', () => {
    expect(toAppError(new Error('kaput'))).toEqual({ code: 'other', message: 'kaput' });
  });

  it('wraps a bare string', () => {
    expect(toAppError('kaput')).toEqual({ code: 'other', message: 'kaput' });
  });

  it('never returns an empty message', () => {
    expect(toAppError(null).message.length).toBeGreaterThan(0);
  });
});

describe('isSettingsError', () => {
  it('is true for codes a settings change fixes', () => {
    for (const code of ['cli_not_found', 'llm_auth', 'search_auth', 'invalid'] as const) {
      expect(isSettingsError(code)).toBe(true);
    }
  });

  it('is false for transient codes', () => {
    for (const code of ['network', 'llm_rate_limit', 'cli_timeout'] as const) {
      expect(isSettingsError(code)).toBe(false);
    }
  });
});

describe('errorKey', () => {
  it('namespaces codes under `error.`', () => {
    expect(errorKey('cli_timeout')).toBe('error.cli_timeout');
  });
});
```

- [ ] **Step 12: Run it to verify it fails**

Run: `pnpm test -- src/lib/errors.test.ts`
Expected: FAIL — `Cannot find module './errors'`.

- [ ] **Step 13: Implement `src/lib/errors.ts`**

```ts
/** Mirrors `ErrorCode` in `src-tauri/src/error.rs`. Keep the two in sync. */
export const ERROR_CODES = [
  'cli_not_found',
  'cli_failed',
  'cli_timeout',
  'cli_bad_output',
  'llm_auth',
  'llm_rate_limit',
  'llm_http',
  'search_auth',
  'search_rate_limit',
  'search_http',
  'network',
  'keychain',
  'store',
  'io',
  'serde',
  'tauri',
  'hotkey',
  'not_found',
  'invalid',
  'other',
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export interface AppErrorPayload {
  code: ErrorCode;
  message: string;
}

/** Codes whose remedy lives in Settings, so the UI offers a jump there. */
const SETTINGS_CODES: ReadonlySet<ErrorCode> = new Set<ErrorCode>([
  'cli_not_found',
  'cli_bad_output',
  'llm_auth',
  'search_auth',
  'keychain',
  'invalid',
]);

function isErrorCode(value: unknown): value is ErrorCode {
  return typeof value === 'string' && (ERROR_CODES as readonly string[]).includes(value);
}

/**
 * Normalizes anything a rejected `invoke()` (or a thrown JS error) can produce
 * into `{ code, message }`. Tauri rejects with the serialized `AppError`, but a
 * frontend bug, a plugin, or the browser-preview path can throw other shapes.
 */
export function toAppError(caught: unknown): AppErrorPayload {
  if (typeof caught === 'object' && caught !== null && 'message' in caught) {
    const record = caught as { code?: unknown; message?: unknown };
    const message = String(record.message ?? '').trim();
    return {
      code: isErrorCode(record.code) ? record.code : 'other',
      message: message.length > 0 ? message : 'unknown error',
    };
  }

  const message = String(caught ?? '').trim();
  return { code: 'other', message: message.length > 0 ? message : 'unknown error' };
}

export function isSettingsError(code: ErrorCode): boolean {
  return SETTINGS_CODES.has(code);
}

export function errorKey(code: ErrorCode): string {
  return `error.${code}`;
}
```

- [ ] **Step 14: Run the test to verify it passes**

Run: `pnpm test -- src/lib/errors.test.ts`
Expected: PASS (7 tests).

- [ ] **Step 15: Add the localized error copy**

Add an `error` block to `src/lib/i18n/cs.json` (sibling of the existing `errors` block — keep both; `errors.*` is the settings-form copy, `error.*` is the code table):

```json
  "error": {
    "title": "Analýza se nepovedla",
    "details": "Technické detaily",
    "retry": "Zkusit znovu",
    "open_settings": "Otevřít nastavení",
    "dismiss": "Zavřít",
    "cli_not_found": "Nenašel jsem program z nastaveného CLI příkazu. Zkontroluj, že je nainstalovaný a dostupný v PATH.",
    "cli_failed": "CLI příkaz skončil chybou. Zkus ho spustit ručně v terminálu a podívej se na výpis.",
    "cli_timeout": "CLI příkaz neodpověděl včas. Zkus kratší vstup nebo rychlejší model.",
    "cli_bad_output": "CLI příkaz nevrátil platný JSON. Zkontroluj, že příkaz tiskne jen odpověď modelu.",
    "llm_auth": "Anthropic odmítl API klíč. Zkontroluj ho v Nastavení.",
    "llm_rate_limit": "Anthropic tě dočasně omezil. Zkus to za chvíli znovu.",
    "llm_http": "Anthropic vrátil chybu. Zkus to za chvíli znovu.",
    "search_auth": "Brave Search odmítl API klíč. Zkontroluj ho v Nastavení.",
    "search_rate_limit": "Brave Search tě dočasně omezil. Zkus to za chvíli znovu.",
    "search_http": "Brave Search vrátil chybu. Zkus to za chvíli znovu.",
    "network": "Nepodařilo se připojit k síti. Zkontroluj připojení.",
    "keychain": "Nepodařilo se sáhnout do systémové klíčenky.",
    "store": "Nepodařilo se načíst nebo uložit nastavení.",
    "io": "Nepodařilo se přečíst nebo zapsat soubor.",
    "serde": "Data mají neočekávaný tvar.",
    "tauri": "Chyba aplikačního jádra.",
    "hotkey": "Klávesovou zkratku se nepodařilo zaregistrovat. Nejspíš ji používá jiná aplikace.",
    "not_found": "Požadovaná položka neexistuje.",
    "invalid": "Neplatné nastavení nebo vstup.",
    "other": "Něco se pokazilo."
  },
```

And the English counterpart in `src/lib/i18n/en.json`:

```json
  "error": {
    "title": "Analysis failed",
    "details": "Technical details",
    "retry": "Try again",
    "open_settings": "Open settings",
    "dismiss": "Dismiss",
    "cli_not_found": "The program from your CLI command was not found. Check that it is installed and on your PATH.",
    "cli_failed": "The CLI command exited with an error. Try running it manually in a terminal and read the output.",
    "cli_timeout": "The CLI command did not answer in time. Try a shorter input or a faster model.",
    "cli_bad_output": "The CLI command did not return valid JSON. Make sure it prints only the model's answer.",
    "llm_auth": "Anthropic rejected the API key. Check it in Settings.",
    "llm_rate_limit": "Anthropic is rate-limiting you. Try again in a moment.",
    "llm_http": "Anthropic returned an error. Try again in a moment.",
    "search_auth": "Brave Search rejected the API key. Check it in Settings.",
    "search_rate_limit": "Brave Search is rate-limiting you. Try again in a moment.",
    "search_http": "Brave Search returned an error. Try again in a moment.",
    "network": "Could not reach the network. Check your connection.",
    "keychain": "Could not reach the system keychain.",
    "store": "Could not read or write settings.",
    "io": "Could not read or write a file.",
    "serde": "The data had an unexpected shape.",
    "tauri": "Application core error.",
    "hotkey": "The hotkey could not be registered. Another app is probably using it.",
    "not_found": "The requested item does not exist.",
    "invalid": "Invalid settings or input.",
    "other": "Something went wrong."
  },
```

- [ ] **Step 16: Create `src/lib/components/ErrorState.svelte`**

```svelte
<script lang="ts">
  import { errorKey, isSettingsError, type AppErrorPayload } from '$lib/errors';
  import { t } from '$lib/stores/i18n.svelte';

  let {
    error,
    onRetry,
    onSettings,
    onDismiss,
  }: {
    error: AppErrorPayload;
    onRetry?: () => void;
    onSettings?: () => void;
    onDismiss?: () => void;
  } = $props();

  const headline = $derived(t(errorKey(error.code)));
  const showSettings = $derived(Boolean(onSettings) && isSettingsError(error.code));
</script>

<div class="err glass" role="alert">
  <div class="body">
    <strong class="title">{t('error.title')}</strong>
    <p class="msg">{headline}</p>
    <details>
      <summary>{t('error.details')}</summary>
      <pre>{error.message}</pre>
    </details>
  </div>
  <div class="actions">
    {#if onRetry}
      <button type="button" class="primary" onclick={onRetry}>{t('error.retry')}</button>
    {/if}
    {#if showSettings}
      <button type="button" onclick={onSettings}>{t('error.open_settings')}</button>
    {/if}
    {#if onDismiss}
      <button type="button" onclick={onDismiss}>{t('error.dismiss')}</button>
    {/if}
  </div>
</div>

<style>
  .err {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border-color: var(--bad);
  }
  .body {
    min-width: 0;
    flex: 1 1 320px;
  }
  .title {
    display: block;
    color: var(--bad);
    font-size: 14px;
  }
  .msg {
    margin: var(--space-1) 0 var(--space-2);
    color: var(--text);
    font-size: 14px;
    line-height: 1.45;
  }
  summary {
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
  }
  pre {
    max-height: 160px;
    margin: var(--space-2) 0 0;
    padding: var(--space-2);
    overflow: auto;
    border-radius: var(--radius-sm);
    background: var(--neutral-soft);
    color: var(--text-muted);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
```

- [ ] **Step 17: Move the analysis store onto `AppErrorPayload`**

In `src/lib/stores/analysis.svelte.ts`:

- add `import { toAppError, type AppErrorPayload } from '$lib/errors';`
- change `let error = $state<string | null>(null);` to `let error = $state<AppErrorPayload | null>(null);`
- change the getter's return type to `AppErrorPayload | null`
- in `run()`, replace `error = String(caught);` with `error = toAppError(caught);`
- add a `lastInput` field so the error surface can offer a real retry:

```ts
let lastInput = $state<string | AnalyzeInput | null>(null);
```

and in `run()`, set `lastInput = input;` before the `try`. Expose:

```ts
  get lastInput() {
    return lastInput;
  },

  async retry(): Promise<void> {
    if (lastInput === null) return;
    await this.run(lastInput);
  },
```

Also clear `error` in `reset()` (already done) and set `error = null` in the `analysis-started` handler (already done).

- [ ] **Step 18: Replace `alert()` and the raw error banner in `src/routes/+page.svelte`**

- Import `ErrorState` and `toAppError`.
- Add `let preflight = $state<AppErrorPayload | null>(null);`
- Rewrite `handleAnalyze`:

```ts
  async function handleAnalyze(input: AnalyzeInput) {
    const message = preflightError();
    if (message) {
      preflight = { code: 'invalid', message };
      return;
    }

    preflight = null;
    questionText = input.question ?? '';
    answerText = input.answer;
    await analysisStore.run(input);
  }
```

- Render, directly under `<UpdateBanner />`:

```svelte
  {#if preflight}
    <ErrorState
      error={preflight}
      onSettings={() => goto(resolve('/settings'))}
      onDismiss={() => (preflight = null)}
    />
  {/if}
```

- Replace the `{:else if analysisStore.status === 'error'}` branch with:

```svelte
    {:else if analysisStore.status === 'error' && analysisStore.error}
      <ErrorState
        error={analysisStore.error}
        onRetry={() => analysisStore.retry()}
        onSettings={() => goto(resolve('/settings'))}
      />
```

- Delete the now-unused `.status.error` style rule and the `summary.error_prefix` usage. Keep the `summary.error_prefix` i18n key — it is still referenced by nothing, so remove it from both bundles too.

- [ ] **Step 19: Run the whole frontend suite and lint**

Run: `pnpm test && pnpm check && pnpm lint`
Expected: PASS, no svelte-check errors, no lint warnings.

- [ ] **Step 20: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/llm/cli.rs src-tauri/src/llm/anthropic.rs src-tauri/src/search/brave.rs src/lib/errors.ts src/lib/errors.test.ts src/lib/components/ErrorState.svelte src/lib/stores/analysis.svelte.ts src/routes/+page.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(errors): classify failures with stable codes and actionable UI"
```

---

## Task 2: Pre-send confirmation modal

Before any text leaves the user's machine, show exactly what will be sent and where. Default ON; dismissible with "don't ask again", which flips a persisted setting.

**Files:**
- Modify: `src-tauri/src/storage/settings_store.rs`
- Create: `src/lib/sendSummary.ts`
- Create: `src/lib/sendSummary.test.ts`
- Create: `src/lib/components/ConfirmModal.svelte`
- Create: `src/lib/components/ConfirmModal.test.ts`
- Create: `src/lib/components/SendConfirm.svelte`
- Modify: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/stores/settings.svelte.ts`
- Modify: `src/routes/+page.svelte`, `src/routes/settings/+page.svelte`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `AppErrorPayload` / `ErrorState` from Task 1 (only for the settings page's save failure path — optional).
- Produces:
  - Rust `Settings.confirm_before_send: bool` (serde default `true`).
  - TS `Settings.confirm_before_send: boolean`.
  - `src/lib/sendSummary.ts` exports `interface SendDestination { key: string; vars: Record<string, string | number> }` and `describeSend(input: SendSummaryInput): SendDestination[]`.
  - `ConfirmModal.svelte` props `{ open: boolean; titleId?: string; title: string; confirmLabel: string; cancelLabel: string; onConfirm: () => void; onCancel: () => void; children: Snippet }`.

- [ ] **Step 1: Write the failing Rust settings tests**

Add to the `tests` module in `src-tauri/src/storage/settings_store.rs`:

```rust
    #[test]
    fn confirm_before_send_defaults_to_true() {
        assert!(Settings::default().confirm_before_send);
    }

    #[test]
    fn legacy_settings_without_confirm_flag_default_to_true() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert!(parsed.confirm_before_send);
    }

    #[test]
    fn confirm_before_send_roundtrips_json() {
        let settings = Settings {
            confirm_before_send: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert!(!back.confirm_before_send);
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd src-tauri && cargo test storage::settings_store`
Expected: FAIL — `struct Settings has no field named confirm_before_send`.

- [ ] **Step 3: Add the field**

In `src-tauri/src/storage/settings_store.rs`, add to `struct Settings`:

```rust
    /// When true, the app asks for explicit confirmation before the first byte
    /// of the user's text leaves the process. Default on — the disclosure is
    /// the point of the app's privacy promise, so opting out must be deliberate.
    #[serde(default = "default_true")]
    pub confirm_before_send: bool,
```

Add the serde default helper next to the others:

```rust
const fn default_true() -> bool {
    true
}
```

And in `impl Default for Settings`, add `confirm_before_send: true,`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test storage::settings_store`
Expected: PASS.

- [ ] **Step 5: Write the failing send-summary test**

Create `src/lib/sendSummary.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { describeSend } from './sendSummary';
import type { Settings } from './types';

const base: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  cache_ttl_days: 7,
  onboarded: true,
  provider: 'cli',
  anthropic_model: 'claude-haiku-4-5-20251001',
  cli_command: 'claude -p',
  check_updates_on_launch: false,
  theme: 'auto',
  verified_claims_limit: 8,
  confirm_before_send: true,
};

describe('describeSend', () => {
  it('reports a local CLI run and no web search', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: false,
      question: '',
      answer: 'Karel IV. se narodil v roce 1316.',
    });

    expect(lines.map((line) => line.key)).toEqual([
      'send.dest_cli',
      'send.web_off',
      'send.payload',
    ]);
    expect(lines[0].vars.command).toBe('claude -p');
  });

  it('reports the Anthropic endpoint with the model', () => {
    const lines = describeSend({
      settings: { ...base, provider: 'anthropic' },
      bravePresent: false,
      question: '',
      answer: 'x',
    });

    expect(lines[0].key).toBe('send.dest_anthropic');
    expect(lines[0].vars.model).toBe('claude-haiku-4-5-20251001');
  });

  it('reports web verification with the configured limit', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: true,
      question: '',
      answer: 'x',
    });

    const web = lines.find((line) => line.key === 'send.web_on');
    expect(web?.vars.limit).toBe(8);
  });

  it('says "all" when the verification limit is null', () => {
    const lines = describeSend({
      settings: { ...base, verified_claims_limit: null },
      bravePresent: true,
      question: '',
      answer: 'x',
    });

    expect(lines.some((line) => line.key === 'send.web_on_all')).toBe(true);
  });

  it('counts question and answer characters separately', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: false,
      question: 'abc',
      answer: 'abcde',
    });

    const payload = lines.find((line) => line.key === 'send.payload');
    expect(payload?.vars).toEqual({ questionChars: 3, answerChars: 5 });
  });
});
```

- [ ] **Step 6: Run it to verify it fails**

Run: `pnpm test -- src/lib/sendSummary.test.ts`
Expected: FAIL — `Cannot find module './sendSummary'`.

- [ ] **Step 7: Implement `src/lib/sendSummary.ts`**

```ts
import type { Settings } from './types';

export interface SendDestination {
  /** i18n key rendered with `tf()`. */
  key: string;
  vars: Record<string, string | number>;
}

export interface SendSummaryInput {
  settings: Settings;
  bravePresent: boolean;
  question: string;
  answer: string;
}

/**
 * Describes, line by line, where this analysis's text is about to go. Pure so
 * the disclosure can be asserted in tests — the modal must never drift from
 * what the pipeline actually does.
 */
export function describeSend(input: SendSummaryInput): SendDestination[] {
  const { settings, bravePresent, question, answer } = input;
  const lines: SendDestination[] = [];

  if (settings.provider === 'anthropic') {
    lines.push({ key: 'send.dest_anthropic', vars: { model: settings.anthropic_model } });
  } else {
    lines.push({ key: 'send.dest_cli', vars: { command: settings.cli_command } });
  }

  if (!bravePresent) {
    lines.push({ key: 'send.web_off', vars: {} });
  } else if (settings.verified_claims_limit === null) {
    lines.push({ key: 'send.web_on_all', vars: {} });
  } else {
    lines.push({ key: 'send.web_on', vars: { limit: settings.verified_claims_limit } });
  }

  lines.push({
    key: 'send.payload',
    vars: { questionChars: question.trim().length, answerChars: answer.trim().length },
  });

  return lines;
}
```

- [ ] **Step 8: Run the test to verify it passes**

Run: `pnpm test -- src/lib/sendSummary.test.ts`
Expected: PASS (5 tests).

- [ ] **Step 9: Write the failing modal accessibility test**

Create `src/lib/components/ConfirmModal.test.ts`:

```ts
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ConfirmModal from './ConfirmModal.svelte';

afterEach(cleanup);

function open(overrides: Record<string, unknown> = {}) {
  return render(ConfirmModal, {
    props: {
      open: true,
      title: 'Send this?',
      confirmLabel: 'Send',
      cancelLabel: 'Cancel',
      onConfirm: vi.fn(),
      onCancel: vi.fn(),
      ...overrides,
    },
  });
}

describe('ConfirmModal', () => {
  it('exposes a labelled modal dialog', () => {
    open();
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAccessibleName('Send this?');
  });

  it('renders nothing when closed', () => {
    render(ConfirmModal, {
      props: {
        open: false,
        title: 'Send this?',
        confirmLabel: 'Send',
        cancelLabel: 'Cancel',
        onConfirm: vi.fn(),
        onCancel: vi.fn(),
      },
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('cancels on Escape', async () => {
    const onCancel = vi.fn();
    open({ onCancel });
    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('confirms via the confirm button', async () => {
    const onConfirm = vi.fn();
    open({ onConfirm });
    await fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('keeps Tab inside the dialog', async () => {
    open();
    const confirm = screen.getByRole('button', { name: 'Send' });
    const cancel = screen.getByRole('button', { name: 'Cancel' });

    confirm.focus();
    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab' });
    expect(document.activeElement).toBe(cancel);

    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(confirm);
  });
});
```

`@testing-library/jest-dom` matchers (`toHaveAttribute`, `toHaveAccessibleName`) need a setup file. Create `src/test-setup.ts`:

```ts
import '@testing-library/jest-dom/vitest';
```

and register it in `vite.config.ts`:

```ts
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
  },
```

- [ ] **Step 10: Run it to verify it fails**

Run: `pnpm test -- src/lib/components/ConfirmModal.test.ts`
Expected: FAIL — `Failed to resolve import "./ConfirmModal.svelte"`.

- [ ] **Step 11: Implement `src/lib/components/ConfirmModal.svelte`**

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = false,
    title,
    confirmLabel,
    cancelLabel,
    confirmDisabled = false,
    onConfirm,
    onCancel,
    children,
  }: {
    open?: boolean;
    title: string;
    confirmLabel: string;
    cancelLabel: string;
    confirmDisabled?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
    children?: Snippet;
  } = $props();

  const titleId = `confirm-title-${Math.random().toString(36).slice(2, 9)}`;

  let dialog = $state<HTMLDivElement | null>(null);
  let restoreFocus: HTMLElement | null = null;

  const FOCUSABLE =
    'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (element) => element.offsetParent !== null || element === document.activeElement,
    );
  }

  // Focus the first control when the dialog opens; restore the previously
  // focused element when it closes, so keyboard and screen-reader users land
  // back where they were.
  $effect(() => {
    if (!open) return;
    restoreFocus = document.activeElement as HTMLElement | null;
    queueMicrotask(() => focusable()[0]?.focus());

    return () => {
      restoreFocus?.focus?.();
      restoreFocus = null;
    };
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
      return;
    }

    if (event.key !== 'Tab') return;

    const items = focusable();
    if (items.length === 0) return;

    event.preventDefault();
    const index = items.indexOf(document.activeElement as HTMLElement);
    const delta = event.shiftKey ? -1 : 1;
    const next = (index + delta + items.length) % items.length;
    items[next].focus();
  }
</script>

{#if open}
  <div class="backdrop">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="dialog glass"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      bind:this={dialog}
      onkeydown={onKeydown}
    >
      <h2 id={titleId}>{title}</h2>
      <div class="body">
        {@render children?.()}
      </div>
      <div class="actions">
        <button type="button" onclick={onCancel}>{cancelLabel}</button>
        <button type="button" class="primary" disabled={confirmDisabled} onclick={onConfirm}>
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: rgba(9, 9, 14, 0.45);
  }

  .dialog {
    width: min(520px, 100%);
    max-height: 85vh;
    overflow-y: auto;
    padding: var(--space-5);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: 18px;
    letter-spacing: -0.01em;
  }

  .body {
    margin-bottom: var(--space-4);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
```

- [ ] **Step 12: Run the modal test to verify it passes**

Run: `pnpm test -- src/lib/components/ConfirmModal.test.ts`
Expected: PASS (5 tests).

- [ ] **Step 13: Add the disclosure copy**

Add to `src/lib/i18n/cs.json`:

```json
  "send": {
    "title": "Odeslat k analýze?",
    "intro": "Než začne analýza, tohle opustí tvůj počítač:",
    "dest_cli": "Text jde do lokálního procesu „{command}“. Neopouští tvůj počítač — kam ho pošle sám nástroj, závisí na jeho nastavení.",
    "dest_anthropic": "Text jde na api.anthropic.com, model {model}.",
    "web_on": "Prvních {limit} faktických tvrzení se pošle jako vyhledávací dotazy na api.search.brave.com.",
    "web_on_all": "Každé faktické tvrzení se pošle jako vyhledávací dotaz na api.search.brave.com.",
    "web_off": "Na web se nic neposílá — Brave Search API klíč není uložený.",
    "payload": "Dotaz: {questionChars} znaků · Odpověď: {answerChars} znaků.",
    "dont_ask": "Příště se neptat",
    "confirm": "Odeslat a analyzovat",
    "cancel": "Zrušit"
  },
```

And `src/lib/i18n/en.json`:

```json
  "send": {
    "title": "Send for analysis?",
    "intro": "Before the analysis starts, this leaves your computer:",
    "dest_cli": "The text goes to the local process \"{command}\". It does not leave your computer — where that tool sends it is up to its own configuration.",
    "dest_anthropic": "The text goes to api.anthropic.com, model {model}.",
    "web_on": "The first {limit} factual claims are sent as search queries to api.search.brave.com.",
    "web_on_all": "Every factual claim is sent as a search query to api.search.brave.com.",
    "web_off": "Nothing goes to the web — no Brave Search API key is stored.",
    "payload": "Question: {questionChars} characters · Answer: {answerChars} characters.",
    "dont_ask": "Don't ask again",
    "confirm": "Send and analyze",
    "cancel": "Cancel"
  },
```

- [ ] **Step 14: Implement `src/lib/components/SendConfirm.svelte`**

```svelte
<script lang="ts">
  import { describeSend } from '$lib/sendSummary';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let {
    question,
    answer,
    dontAsk = $bindable(false),
  }: {
    question: string;
    answer: string;
    dontAsk?: boolean;
  } = $props();

  const lines = $derived(
    describeSend({
      settings: settings.current,
      bravePresent: settings.bravePresent,
      question,
      answer,
    }),
  );
</script>

<p class="intro">{t('send.intro')}</p>
<ul>
  {#each lines as line (line.key)}
    <li>{tf(line.key, line.vars)}</li>
  {/each}
</ul>
<label class="dont-ask">
  <input type="checkbox" bind:checked={dontAsk} />
  <span>{t('send.dont_ask')}</span>
</label>

<style>
  .intro {
    margin: 0 0 var(--space-2);
    color: var(--text-muted);
    font-size: 14px;
  }
  ul {
    margin: 0 0 var(--space-3);
    padding-left: var(--space-5);
    color: var(--text);
    font-size: 14px;
    line-height: 1.55;
  }
  li + li {
    margin-top: var(--space-1);
  }
  .dont-ask {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-muted);
    font-size: 13px;
  }
  .dont-ask input {
    width: auto;
    min-height: 0;
  }
</style>
```

- [ ] **Step 15: Thread the setting through the frontend types and store**

- `src/lib/types.ts` — add to `interface Settings`:

```ts
  /** Ask for explicit confirmation before any text leaves the machine. */
  confirm_before_send: boolean;
```

- `src/lib/api.ts` — add `confirm_before_send: true,` to `browserDefaultSettings()`.
- `src/lib/stores/settings.svelte.ts` — add `confirm_before_send: true,` to `defaults`.
- `src/lib/preflight.test.ts` — add `confirm_before_send: true,` to the `cliSettings` fixture.

- [ ] **Step 16: Gate the analysis on the modal in `src/routes/+page.svelte`**

Add imports for `ConfirmModal` and `SendConfirm`, then:

```ts
  let pendingInput = $state<AnalyzeInput | null>(null);
  let dontAskAgain = $state(false);

  async function handleAnalyze(input: AnalyzeInput) {
    const message = preflightError();
    if (message) {
      preflight = { code: 'invalid', message };
      return;
    }

    preflight = null;
    if (settings.current.confirm_before_send) {
      pendingInput = input;
      return;
    }

    await start(input);
  }

  async function start(input: AnalyzeInput) {
    questionText = input.question ?? '';
    answerText = input.answer;
    await analysisStore.run(input);
  }

  async function confirmSend() {
    const input = pendingInput;
    pendingInput = null;
    if (!input) return;

    if (dontAskAgain) {
      await settings.save({ ...settings.current, confirm_before_send: false });
      dontAskAgain = false;
    }

    await start(input);
  }
```

Render at the end of `<main>`:

```svelte
  <ConfirmModal
    open={pendingInput !== null}
    title={t('send.title')}
    confirmLabel={t('send.confirm')}
    cancelLabel={t('send.cancel')}
    onConfirm={confirmSend}
    onCancel={() => {
      pendingInput = null;
      dontAskAgain = false;
    }}
  >
    <SendConfirm
      question={pendingInput?.question ?? ''}
      answer={pendingInput?.answer ?? ''}
      bind:dontAsk={dontAskAgain}
    />
  </ConfirmModal>
```

- [ ] **Step 17: Expose the toggle in Settings**

In `src/routes/settings/+page.svelte`, add a section above the updates section:

```svelte
  <section class="settings-grid glass">
    <h2>{t('settings.privacy_section')}</h2>
    <label class="check">
      <input type="checkbox" bind:checked={local.confirm_before_send} />
      <span>{t('settings.confirm_before_send_label')}</span>
    </label>
    <small class="hint">{t('settings.confirm_before_send_hint')}</small>
  </section>
```

Add the keys to both bundles inside `settings`:

```json
    "privacy_section": "Soukromí",
    "confirm_before_send_label": "Před odesláním se zeptat",
    "confirm_before_send_hint": "Před každou analýzou ukáže, co přesně a kam se odesílá. Vypnutím se analýza spustí hned.",
```

```json
    "privacy_section": "Privacy",
    "confirm_before_send_label": "Confirm before sending",
    "confirm_before_send_hint": "Shows exactly what is sent and where before every analysis. Turning this off starts the analysis immediately.",
```

- [ ] **Step 18: Run everything**

Run: `pnpm test && pnpm check && pnpm lint && (cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings)`
Expected: PASS.

- [ ] **Step 19: Commit**

```bash
git add src-tauri/src/storage/settings_store.rs src/lib/sendSummary.ts src/lib/sendSummary.test.ts src/lib/components/ConfirmModal.svelte src/lib/components/ConfirmModal.test.ts src/lib/components/SendConfirm.svelte src/lib/types.ts src/lib/api.ts src/lib/stores/settings.svelte.ts src/lib/preflight.test.ts src/routes/+page.svelte src/routes/settings/+page.svelte src/lib/i18n/cs.json src/lib/i18n/en.json src/test-setup.ts vite.config.ts
git commit -m "feat(privacy): pre-send confirmation modal with explicit destination disclosure"
```

---

## Task 3: First-run onboarding

`Settings.onboarded` has existed since M0 and nothing ever reads it. This task makes it mean something: a four-step overlay that explains the tool, discloses the privacy model, gets a working provider configured, and shows the hotkey. It sets `onboarded: true` only when the user finishes or explicitly skips.

**Files:**
- Create: `src/lib/onboarding.ts`
- Create: `src/lib/onboarding.test.ts`
- Create: `src/lib/components/Onboarding.svelte`
- Modify: `src/routes/+layout.svelte`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `settings` store, `CLI_PRESETS` / `presetCommand` from `$lib/cliPresets`, `setApiKey` from `$lib/api`.
- Produces: `ONBOARDING_STEPS: readonly OnboardingStep[]` where `type OnboardingStep = 'welcome' | 'privacy' | 'provider' | 'ready'`; `nextStep(step)`, `prevStep(step)`, `canAdvance(step, draft, anthropicKeyEntered)`.

- [ ] **Step 1: Write the failing step-machine test**

Create `src/lib/onboarding.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { canAdvance, nextStep, ONBOARDING_STEPS, prevStep } from './onboarding';
import type { Settings } from './types';

const draft: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  cache_ttl_days: 7,
  onboarded: false,
  provider: 'cli',
  anthropic_model: 'claude-haiku-4-5-20251001',
  cli_command: 'claude -p',
  check_updates_on_launch: false,
  theme: 'auto',
  verified_claims_limit: 8,
  confirm_before_send: true,
};

describe('onboarding steps', () => {
  it('runs welcome → privacy → provider → ready', () => {
    expect(ONBOARDING_STEPS).toEqual(['welcome', 'privacy', 'provider', 'ready']);
  });

  it('clamps at both ends', () => {
    expect(prevStep('welcome')).toBe('welcome');
    expect(nextStep('ready')).toBe('ready');
    expect(nextStep('welcome')).toBe('privacy');
    expect(prevStep('provider')).toBe('privacy');
  });
});

describe('canAdvance', () => {
  it('always allows leaving the informational steps', () => {
    expect(canAdvance('welcome', draft, false)).toBe(true);
    expect(canAdvance('privacy', draft, false)).toBe(true);
  });

  it('requires a non-empty CLI command', () => {
    expect(canAdvance('provider', draft, false)).toBe(true);
    expect(canAdvance('provider', { ...draft, cli_command: '  ' }, false)).toBe(false);
  });

  it('requires a model and a key for the Anthropic provider', () => {
    const anthropic: Settings = { ...draft, provider: 'anthropic' };
    expect(canAdvance('provider', anthropic, false)).toBe(false);
    expect(canAdvance('provider', anthropic, true)).toBe(true);
    expect(canAdvance('provider', { ...anthropic, anthropic_model: '' }, true)).toBe(false);
  });

  it('always allows finishing from the last step', () => {
    expect(canAdvance('ready', draft, false)).toBe(true);
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm test -- src/lib/onboarding.test.ts`
Expected: FAIL — `Cannot find module './onboarding'`.

- [ ] **Step 3: Implement `src/lib/onboarding.ts`**

```ts
import type { Settings } from './types';

export const ONBOARDING_STEPS = ['welcome', 'privacy', 'provider', 'ready'] as const;

export type OnboardingStep = (typeof ONBOARDING_STEPS)[number];

function shift(step: OnboardingStep, delta: number): OnboardingStep {
  const index = ONBOARDING_STEPS.indexOf(step);
  const next = Math.min(Math.max(index + delta, 0), ONBOARDING_STEPS.length - 1);
  return ONBOARDING_STEPS[next];
}

export function nextStep(step: OnboardingStep): OnboardingStep {
  return shift(step, 1);
}

export function prevStep(step: OnboardingStep): OnboardingStep {
  return shift(step, -1);
}

/**
 * Whether the user may leave `step`. Only the provider step gates: finishing
 * onboarding with no working provider would drop the user straight into a
 * failed analysis.
 *
 * `anthropicKeyEntered` is true when a key is already in the keychain or the
 * user typed one into the onboarding field.
 */
export function canAdvance(
  step: OnboardingStep,
  draft: Settings,
  anthropicKeyEntered: boolean,
): boolean {
  if (step !== 'provider') return true;

  if (draft.provider === 'anthropic') {
    return anthropicKeyEntered && draft.anthropic_model.trim().length > 0;
  }

  return draft.cli_command.trim().length > 0;
}
```

- [ ] **Step 4: Run it to verify it passes**

Run: `pnpm test -- src/lib/onboarding.test.ts`
Expected: PASS (6 tests).

- [ ] **Step 5: Add the onboarding copy**

Add to `src/lib/i18n/cs.json`:

```json
  "onboarding": {
    "step_of": "Krok {current} ze {total}",
    "welcome_title": "Vítej v PROVE",
    "welcome_body": "Vlož odpověď od AI. PROVE ji rozloží na jednotlivá tvrzení, u každého řekne, jestli jde o ověřitelný fakt, odvození, domněnku nebo vnitřní rozpor, a faktická tvrzení zkusí ověřit proti webu.",
    "welcome_note": "PROVE není arbitr pravdy. Ukazuje ti, co se dá ověřit a čím — rozhodnutí zůstává na tobě.",
    "privacy_title": "Co odchází z počítače",
    "privacy_local": "Analýzy, nastavení i cache zůstávají v datové složce aplikace na tvém disku. Žádná telemetrie, žádné odesílání dat nám.",
    "privacy_llm": "Text jde tomu poskytovateli LLM, kterého si vybereš v dalším kroku — buď lokálnímu CLI nástroji, nebo do Anthropic API.",
    "privacy_web": "Pokud uložíš Brave Search API klíč, posílají se jednotlivá faktická tvrzení jako vyhledávací dotazy. Bez klíče se na web neposílá nic.",
    "privacy_keys": "API klíče se ukládají do systémové klíčenky, ne do souboru s nastavením.",
    "privacy_confirm": "Před každou analýzou uvidíš přesný souhrn a musíš ho potvrdit. Tohle jde vypnout v Nastavení.",
    "provider_title": "Vyber, jak volat LLM",
    "provider_body": "Doporučená volba je lokální CLI nástroj, který už na počítači máš — pak neplatíš nic navíc a text neposílá aplikace nikam sama.",
    "brave_optional": "Brave Search API klíč (volitelný, pro ověřování proti webu)",
    "ready_title": "Hotovo",
    "ready_hotkey": "Kdekoli v systému stiskni {hotkey} — PROVE se otevře a předvyplní text ze schránky.",
    "ready_body": "Zkratku, jazyk i rozsah ověřování změníš kdykoli v Nastavení.",
    "back": "Zpět",
    "next": "Pokračovat",
    "finish": "Začít používat",
    "skip": "Přeskočit"
  },
```

And `src/lib/i18n/en.json`:

```json
  "onboarding": {
    "step_of": "Step {current} of {total}",
    "welcome_title": "Welcome to PROVE",
    "welcome_body": "Paste an AI answer. PROVE breaks it into individual claims, labels each one as a verifiable fact, an inference, an opinion, or an internal contradiction, and tries to check the factual ones against the web.",
    "welcome_note": "PROVE is not an arbiter of truth. It shows you what can be checked and with what — the judgment stays yours.",
    "privacy_title": "What leaves your computer",
    "privacy_local": "Analyses, settings, and the cache stay in the app's data folder on your disk. No telemetry, nothing sent to us.",
    "privacy_llm": "The text goes to whichever LLM provider you pick in the next step — either a local CLI tool or the Anthropic API.",
    "privacy_web": "If you store a Brave Search API key, individual factual claims are sent as search queries. Without a key, nothing goes to the web.",
    "privacy_keys": "API keys are stored in the system keychain, not in the settings file.",
    "privacy_confirm": "Before every analysis you see an exact summary and have to confirm it. This can be turned off in Settings.",
    "provider_title": "Choose how to call the LLM",
    "provider_body": "The recommended option is a local CLI tool you already have — nothing extra to pay for, and the app itself sends the text nowhere.",
    "brave_optional": "Brave Search API key (optional, for web verification)",
    "ready_title": "You're set",
    "ready_hotkey": "Press {hotkey} anywhere in the system — PROVE opens and pre-fills from your clipboard.",
    "ready_body": "The hotkey, language, and verification depth are all changeable in Settings.",
    "back": "Back",
    "next": "Continue",
    "finish": "Start using PROVE",
    "skip": "Skip"
  },
```

- [ ] **Step 6: Implement `src/lib/components/Onboarding.svelte`**

```svelte
<script lang="ts">
  import { setApiKey } from '$lib/api';
  import { CLI_PRESETS, commandToCliPreset, presetCommand, type CliPresetId } from '$lib/cliPresets';
  import { toAppError, type AppErrorPayload } from '$lib/errors';
  import {
    canAdvance,
    nextStep,
    ONBOARDING_STEPS,
    prevStep,
    type OnboardingStep,
  } from '$lib/onboarding';
  import { formatAccelerator, platformKind } from '$lib/hotkey';
  import { setLocale, t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type Settings } from '$lib/types';
  import ErrorState from './ErrorState.svelte';

  let { onDone }: { onDone: () => void } = $props();

  let step = $state<OnboardingStep>('welcome');
  let draft = $state<Settings>({ ...settings.current });
  let cliPreset = $state<CliPresetId>(commandToCliPreset(settings.current.cli_command));
  let anthropicInput = $state('');
  let braveInput = $state('');
  let busy = $state(false);
  let failure = $state<AppErrorPayload | null>(null);

  const index = $derived(ONBOARDING_STEPS.indexOf(step));
  const isLast = $derived(step === 'ready');
  const advanceAllowed = $derived(
    canAdvance(step, draft, settings.anthropicPresent || anthropicInput.trim().length > 0),
  );
  const hotkeyLabel = $derived(formatAccelerator(draft.hotkey, platformKind()));

  function applyCliPreset() {
    const command = presetCommand(cliPreset);
    if (command) draft.cli_command = command;
  }

  async function persistKeys() {
    if (anthropicInput.trim()) await setApiKey(ACCOUNT_ANTHROPIC, anthropicInput.trim());
    if (braveInput.trim()) await setApiKey(ACCOUNT_BRAVE, braveInput.trim());
    if (anthropicInput.trim() || braveInput.trim()) await settings.refreshKeyState();
    anthropicInput = '';
    braveInput = '';
  }

  async function advance() {
    failure = null;
    if (step === 'provider') {
      busy = true;
      try {
        await persistKeys();
      } catch (caught) {
        failure = toAppError(caught);
        return;
      } finally {
        busy = false;
      }
    }

    step = nextStep(step);
  }

  async function finish() {
    busy = true;
    failure = null;
    try {
      await settings.save({ ...draft, onboarded: true });
      setLocale(draft.locale);
      onDone();
    } catch (caught) {
      failure = toAppError(caught);
    } finally {
      busy = false;
    }
  }
</script>

<div class="backdrop">
  <section
    class="card glass"
    role="dialog"
    aria-modal="true"
    aria-labelledby="onboarding-title"
    tabindex="-1"
  >
    <p class="progress">
      {tf('onboarding.step_of', { current: index + 1, total: ONBOARDING_STEPS.length })}
    </p>

    {#if step === 'welcome'}
      <h2 id="onboarding-title">{t('onboarding.welcome_title')}</h2>
      <p>{t('onboarding.welcome_body')}</p>
      <p class="note">{t('onboarding.welcome_note')}</p>
      <label class="inline">
        <span>{t('settings.locale_label')}</span>
        <select bind:value={draft.locale} onchange={() => setLocale(draft.locale)}>
          <option value="cs">Čeština</option>
          <option value="en">English</option>
        </select>
      </label>
    {:else if step === 'privacy'}
      <h2 id="onboarding-title">{t('onboarding.privacy_title')}</h2>
      <ul>
        <li>{t('onboarding.privacy_local')}</li>
        <li>{t('onboarding.privacy_llm')}</li>
        <li>{t('onboarding.privacy_web')}</li>
        <li>{t('onboarding.privacy_keys')}</li>
        <li>{t('onboarding.privacy_confirm')}</li>
      </ul>
    {:else if step === 'provider'}
      <h2 id="onboarding-title">{t('onboarding.provider_title')}</h2>
      <p>{t('onboarding.provider_body')}</p>
      <label>
        <span>{t('settings.provider_label')}</span>
        <select bind:value={draft.provider}>
          <option value="cli">{t('settings.provider_cli')}</option>
          <option value="anthropic">{t('settings.provider_anthropic')}</option>
        </select>
      </label>
      {#if draft.provider === 'cli'}
        <label>
          <span>{t('settings.cli_preset_label')}</span>
          <select bind:value={cliPreset} onchange={applyCliPreset}>
            {#each CLI_PRESETS as preset (preset.id)}
              <option value={preset.id}>{t(`settings.cli_preset_${preset.id}`)}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>{t('settings.cli_command_label')}</span>
          <input
            type="text"
            bind:value={draft.cli_command}
            oninput={() => (cliPreset = commandToCliPreset(draft.cli_command))}
            placeholder={t('settings.cli_command_placeholder')}
            autocomplete="off"
            spellcheck="false"
          />
        </label>
      {:else}
        <label>
          <span>{t('settings.anthropic_model_label')}</span>
          <input type="text" bind:value={draft.anthropic_model} autocomplete="off" />
        </label>
        <label>
          <span>{t('settings.anthropic_key_label')}</span>
          <input
            type="password"
            bind:value={anthropicInput}
            placeholder={t('settings.anthropic_key_placeholder')}
            autocomplete="off"
          />
        </label>
      {/if}
      <label>
        <span>{t('onboarding.brave_optional')}</span>
        <input
          type="password"
          bind:value={braveInput}
          placeholder={t('settings.brave_key_placeholder')}
          autocomplete="off"
        />
      </label>
    {:else}
      <h2 id="onboarding-title">{t('onboarding.ready_title')}</h2>
      <p>{tf('onboarding.ready_hotkey', { hotkey: hotkeyLabel })}</p>
      <p class="note">{t('onboarding.ready_body')}</p>
    {/if}

    {#if failure}
      <ErrorState error={failure} onDismiss={() => (failure = null)} />
    {/if}

    <div class="actions">
      <button type="button" onclick={finish} disabled={busy}>{t('onboarding.skip')}</button>
      <div class="spacer"></div>
      <button type="button" onclick={() => (step = prevStep(step))} disabled={index === 0 || busy}>
        {t('onboarding.back')}
      </button>
      {#if isLast}
        <button type="button" class="primary" onclick={finish} disabled={busy}>
          {t('onboarding.finish')}
        </button>
      {:else}
        <button type="button" class="primary" onclick={advance} disabled={!advanceAllowed || busy}>
          {t('onboarding.next')}
        </button>
      {/if}
    </div>
  </section>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: rgba(9, 9, 14, 0.5);
  }
  .card {
    display: grid;
    gap: var(--space-3);
    width: min(560px, 100%);
    max-height: 88vh;
    overflow-y: auto;
    padding: var(--space-6);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }
  .progress {
    margin: 0;
    color: var(--text-subtle);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  h2 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0;
    color: var(--text);
    font-size: 14px;
    line-height: 1.55;
  }
  .note {
    color: var(--text-muted);
    font-size: 13px;
  }
  ul {
    margin: 0;
    padding-left: var(--space-5);
    font-size: 14px;
    line-height: 1.6;
  }
  li + li {
    margin-top: var(--space-2);
  }
  label {
    display: grid;
    gap: var(--space-2);
  }
  label.inline {
    grid-template-columns: auto minmax(0, 200px);
    align-items: center;
  }
  label span {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
  .spacer {
    flex: 1;
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
```

> `formatAccelerator` and `platformKind` come from Task 6. Implement Task 6 before this step, or stub the label as `draft.hotkey` and replace it when Task 6 lands. The plan orders Task 6 after this one for review-size reasons; if you are executing strictly in order, do Task 6's `src/lib/hotkey.ts` module first — it has no dependencies.

- [ ] **Step 7: Mount it from the layout**

In `src/routes/+layout.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css';
  import Onboarding from '$lib/components/Onboarding.svelte';
  import { setLocale, t } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { theme } from '$lib/stores/theme.svelte';

  let { children } = $props();

  let bootLabel = $state('Loading… / Spouštím…');
  let onboardingDone = $state(false);

  const showOnboarding = $derived(
    settings.loaded && !settings.current.onboarded && !onboardingDone,
  );

  onMount(async () => {
    const nav = typeof navigator !== 'undefined' ? navigator.language : '';
    bootLabel = nav?.toLowerCase().startsWith('cs') ? 'Spouštím…' : 'Loading…';
    await settings.load();
    setLocale(settings.current.locale);
    theme.init(settings.current.theme, settings.current.high_contrast);
  });
</script>

{#if settings.loaded}
  <div class="app-mesh" aria-hidden="true"></div>
  <a class="skip-link" href="#main">{t('a11y.skip_to_content')}</a>
  {@render children()}
  {#if showOnboarding}
    <Onboarding onDone={() => (onboardingDone = true)} />
  {/if}
{:else}
  <div class="boot">{bootLabel}</div>
{/if}
```

> `theme.init(pref, highContrast)` and `a11y.skip_to_content` land in Task 7. Until then keep `theme.init(settings.current.theme)` and omit the skip link.

- [ ] **Step 8: Run everything**

Run: `pnpm test && pnpm check && pnpm lint`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src/lib/onboarding.ts src/lib/onboarding.test.ts src/lib/components/Onboarding.svelte src/routes/+layout.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(onboarding): first-run flow with privacy disclosure and provider setup"
```

---

## Task 4: History backend — close the dead write path

`analysis_history` rows have been written since M2 (`commands/analysis.rs` calls `storage::history::insert` twice per analysis), but `storage/history.rs` only has `insert`, `commands/history.rs` is a one-line comment, and `src/lib/api.ts` has no history functions at all. Nothing can read those rows. This task adds read/delete/prune to storage and exposes them as commands.

**Files:**
- Modify: `src-tauri/src/storage/history.rs`
- Replace: `src-tauri/src/commands/history.rs`
- Modify: `src-tauri/src/storage/settings_store.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `Db` (`storage::db`), `Analysis` (`models`), `AppError`/`ErrorCode` (Task 1).
- Produces:
  - `storage::history::HistoryEntry { id: String, created_at: i64, preview: String, claim_count: usize }` (serde snake_case, matching `Analysis`).
  - `storage::history::{list, get, delete, clear, prune}`.
  - Commands `list_history(query: Option<String>, limit: Option<usize>) -> Vec<HistoryEntry>`, `get_analysis(id: String) -> Analysis`, `delete_analysis(id: String) -> ()`, `clear_history() -> usize`.
  - `Settings.history_retention_days: Option<u32>` (serde default `Some(90)`; `None` = keep forever).

- [ ] **Step 1: Write the failing storage tests**

Replace the `tests` module in `src-tauri/src/storage/history.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Claim, ClaimKind};

    fn analysis_with(id: &str, created_at: i64, input: &str, claims: usize) -> Analysis {
        Analysis {
            id: id.into(),
            created_at,
            input: input.into(),
            claims: (0..claims)
                .map(|index| Claim {
                    id: format!("c{}", index + 1),
                    text: format!("claim {index}"),
                    start: 0,
                    end: 0,
                    kind: ClaimKind::Fact,
                    reason: String::new(),
                    verification: None,
                })
                .collect(),
            truncated: false,
        }
    }

    fn empty_analysis() -> Analysis {
        analysis_with("01900000-0000-0000-0000-000000000001", 1_700_000_000_000, "hi", 0)
    }

    #[test]
    fn insert_replaces_on_id_conflict() {
        let db = Db::open_in_memory().unwrap();
        let mut analysis = empty_analysis();
        insert(&db, &analysis).unwrap();
        analysis.input = "again".into();
        insert(&db, &analysis).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "again");
    }

    #[test]
    fn list_returns_newest_first_with_claim_counts() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1_000, "older", 2)).unwrap();
        insert(&db, &analysis_with("b", 2_000, "newer", 5)).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[0].claim_count, 5);
        assert_eq!(entries[1].id, "a");
        assert_eq!(entries[1].claim_count, 2);
    }

    #[test]
    fn list_respects_the_limit() {
        let db = Db::open_in_memory().unwrap();
        for index in 0..5 {
            insert(&db, &analysis_with(&format!("id{index}"), i64::from(index), "x", 0)).unwrap();
        }
        assert_eq!(list(&db, None, 3).unwrap().len(), 3);
    }

    #[test]
    fn list_filters_case_insensitively_on_input() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "Karel IV. se narodil", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "Praha je hlavní město", 0)).unwrap();

        let entries = list(&db, Some("karel"), 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "a");
    }

    #[test]
    fn list_treats_like_wildcards_as_literals() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "100% jistota", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "necо jineho", 0)).unwrap();

        assert_eq!(list(&db, Some("100%"), 10).unwrap().len(), 1);
        // A bare `%` must not match everything.
        assert_eq!(list(&db, Some("%"), 10).unwrap().len(), 1);
    }

    #[test]
    fn preview_is_truncated_to_the_cap() {
        let db = Db::open_in_memory().unwrap();
        let long = "á".repeat(PREVIEW_CHARS + 50);
        insert(&db, &analysis_with("a", 1, &long, 0)).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries[0].preview.chars().count(), PREVIEW_CHARS);
    }

    #[test]
    fn get_roundtrips_the_full_analysis() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "text", 3)).unwrap();

        let loaded = get(&db, "a").unwrap().unwrap();
        assert_eq!(loaded.claims.len(), 3);
        assert_eq!(loaded.input, "text");
        assert!(get(&db, "missing").unwrap().is_none());
    }

    #[test]
    fn delete_removes_one_row_and_reports_whether_it_existed() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "x", 0)).unwrap();

        assert!(delete(&db, "a").unwrap());
        assert!(!delete(&db, "a").unwrap());
        assert!(list(&db, None, 10).unwrap().is_empty());
    }

    #[test]
    fn clear_removes_everything_and_reports_the_count() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "x", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "y", 0)).unwrap();

        assert_eq!(clear(&db).unwrap(), 2);
        assert!(list(&db, None, 10).unwrap().is_empty());
    }

    #[test]
    fn prune_drops_only_rows_older_than_the_cutoff() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("old", 1_000, "x", 0)).unwrap();
        insert(&db, &analysis_with("new", 5_000, "y", 0)).unwrap();

        assert_eq!(prune(&db, 3_000).unwrap(), 1);
        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "new");
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd src-tauri && cargo test storage::history`
Expected: FAIL — `cannot find function list in this scope`.

- [ ] **Step 3: Implement the storage functions**

Replace the non-test part of `src-tauri/src/storage/history.rs` with:

```rust
use crate::error::AppResult;
use crate::models::Analysis;
use crate::storage::db::Db;
use serde::{Deserialize, Serialize};

/// How much of the analysed input a list row carries. Enough to recognise an
/// entry, small enough that listing 50 rows stays cheap.
pub const PREVIEW_CHARS: usize = 160;

/// Default page size for `list_history`.
pub const DEFAULT_LIST_LIMIT: usize = 50;

/// A row in the history list. Deliberately not the whole `Analysis` — the full
/// record is fetched only when the user opens one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: i64,
    pub preview: String,
    pub claim_count: usize,
}

pub fn insert(db: &Db, analysis: &Analysis) -> AppResult<()> {
    let json = serde_json::to_string(analysis)?;
    db.with(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO analysis_history (id, created_at_ms, input, analysis_json) VALUES (?,?,?,?)",
            rusqlite::params![analysis.id, analysis.created_at, analysis.input, json],
        )?;
        Ok(())
    })
}

/// Newest first. `query` does a case-insensitive substring match on the
/// analysed input; `%`, `_`, and `\` in it are treated as literals.
pub fn list(db: &Db, query: Option<&str>, limit: usize) -> AppResult<Vec<HistoryEntry>> {
    let limit = i64::try_from(limit.max(1)).unwrap_or(i64::from(u32::MAX));
    let pattern = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", escape_like(&value.to_lowercase())));

    let rows: Vec<(String, i64, String, String)> = db.with(|conn| {
        let mut collected = Vec::new();
        match &pattern {
            Some(pattern) => {
                let mut stmt = conn.prepare(
                    "SELECT id, created_at_ms, input, analysis_json FROM analysis_history \
                     WHERE lower(input) LIKE ?1 ESCAPE '\\' \
                     ORDER BY created_at_ms DESC LIMIT ?2",
                )?;
                let mapped = stmt.query_map(rusqlite::params![pattern, limit], row_tuple)?;
                for row in mapped {
                    collected.push(row?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, created_at_ms, input, analysis_json FROM analysis_history \
                     ORDER BY created_at_ms DESC LIMIT ?1",
                )?;
                let mapped = stmt.query_map(rusqlite::params![limit], row_tuple)?;
                for row in mapped {
                    collected.push(row?);
                }
            }
        }
        Ok(collected)
    })?;

    Ok(rows
        .into_iter()
        .map(|(id, created_at, input, json)| HistoryEntry {
            id,
            created_at,
            preview: preview_of(&input),
            claim_count: serde_json::from_str::<Analysis>(&json)
                .map(|analysis| analysis.claims.len())
                .unwrap_or_default(),
        })
        .collect())
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<Analysis>> {
    let json: Option<String> = db.with(|conn| {
        conn.query_row(
            "SELECT analysis_json FROM analysis_history WHERE id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .optional()
    })?;

    match json {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Returns `true` when a row was actually removed.
pub fn delete(db: &Db, id: &str) -> AppResult<bool> {
    let affected = db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_history WHERE id = ?",
            rusqlite::params![id],
        )
    })?;
    Ok(affected > 0)
}

/// Removes every row. Returns how many were removed.
pub fn clear(db: &Db) -> AppResult<usize> {
    db.with(|conn| conn.execute("DELETE FROM analysis_history", []))
}

/// Removes rows created strictly before `cutoff_ms`. Returns how many.
pub fn prune(db: &Db, cutoff_ms: i64) -> AppResult<usize> {
    db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_history WHERE created_at_ms < ?",
            rusqlite::params![cutoff_ms],
        )
    })
}

fn row_tuple(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, i64, String, String)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
}

fn preview_of(input: &str) -> String {
    let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized.chars().take(PREVIEW_CHARS).collect()
}

/// SQLite `LIKE` treats `%` and `_` as wildcards. A user searching for "100%"
/// means the literal characters, so escape them and pair with `ESCAPE '\'`.
fn escape_like(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
```

Add `use rusqlite::OptionalExtension;` at the top so `.optional()` resolves.

- [ ] **Step 4: Run the storage tests**

Run: `cd src-tauri && cargo test storage::history`
Expected: PASS (10 tests).

- [ ] **Step 5: Add the retention setting with failing tests first**

Add to the `tests` module in `src-tauri/src/storage/settings_store.rs`:

```rust
    #[test]
    fn history_retention_defaults_to_ninety_days() {
        assert_eq!(
            Settings::default().history_retention_days,
            Some(DEFAULT_HISTORY_RETENTION_DAYS)
        );
    }

    #[test]
    fn history_retention_forever_validates() {
        let settings = Settings {
            history_retention_days: None,
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn history_retention_zero_rejected() {
        let settings = Settings {
            history_retention_days: Some(0),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn history_retention_over_ten_years_rejected() {
        let settings = Settings {
            history_retention_days: Some(3651),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }
```

Run `cd src-tauri && cargo test storage::settings_store` — FAIL.

Then add to `settings_store.rs`:

```rust
/// Default history retention. `None` in `Settings` means "keep forever".
pub const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 90;
pub const MAX_HISTORY_RETENTION_DAYS: u32 = 3650;
```

field:

```rust
    /// How long analyses stay in the local history, in days. `None` keeps them
    /// forever. Pruned once at startup.
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: Option<u32>,
```

default fn + `Default` entry:

```rust
#[allow(clippy::unnecessary_wraps)] // serde default for an `Option<u32>` field
fn default_history_retention_days() -> Option<u32> {
    Some(DEFAULT_HISTORY_RETENTION_DAYS)
}
```

validation inside `validate()`:

```rust
        if let Some(days) = self.history_retention_days {
            if days == 0 || days > MAX_HISTORY_RETENTION_DAYS {
                return Err(AppError::Invalid(format!(
                    "history_retention_days out of range (1..={MAX_HISTORY_RETENTION_DAYS} or null to keep forever), got {days}"
                )));
            }
        }
```

Run the tests again — PASS.

- [ ] **Step 6: Implement the commands**

Replace `src-tauri/src/commands/history.rs` entirely:

```rust
use crate::error::{AppError, AppResult};
use crate::models::Analysis;
use crate::storage::db::Db;
use crate::storage::history::{self, HistoryEntry, DEFAULT_LIST_LIMIT};
use tauri::State;

/// Hard cap so a malformed frontend call cannot ask for the whole table.
const MAX_LIST_LIMIT: usize = 500;

#[tauri::command]
pub async fn list_history(
    db: State<'_, Db>,
    query: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<HistoryEntry>> {
    let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT).clamp(1, MAX_LIST_LIMIT);
    history::list(&db, query.as_deref(), limit)
}

#[tauri::command]
pub async fn get_analysis(db: State<'_, Db>, id: String) -> AppResult<Analysis> {
    history::get(&db, &id)?.ok_or_else(|| AppError::NotFound(format!("analysis {id}")))
}

#[tauri::command]
pub async fn delete_analysis(db: State<'_, Db>, id: String) -> AppResult<()> {
    history::delete(&db, &id)?;
    Ok(())
}

#[tauri::command]
pub async fn clear_history(db: State<'_, Db>) -> AppResult<usize> {
    history::clear(&db)
}
```

- [ ] **Step 7: Register the commands and prune on startup**

In `src-tauri/src/lib.rs`:

```rust
use commands::history::{clear_history, delete_analysis, get_analysis, list_history};
```

add to `tauri::generate_handler![...]`:

```rust
            list_history,
            get_analysis,
            delete_analysis,
            clear_history,
```

and in `setup()`, after `app.manage(Db::open(...)?)`:

```rust
            let data_dir = app.path().app_data_dir()?;
            let db = Db::open(data_dir.join("cache.db"))?;
            if let Some(days) = settings.history_retention_days {
                let cutoff = chrono::Utc::now().timestamp_millis()
                    - i64::from(days) * 24 * 3_600 * 1_000;
                match storage::history::prune(&db, cutoff) {
                    Ok(0) => {}
                    Ok(removed) => tracing::info!("pruned {removed} history entries"),
                    Err(error) => tracing::warn!("history prune failed: {error}"),
                }
            }
            app.manage(db);
```

(`chrono` is already a dependency — `commands/analysis.rs` uses `chrono::Utc`.)

- [ ] **Step 8: Run the full Rust suite**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/storage/history.rs src-tauri/src/commands/history.rs src-tauri/src/storage/settings_store.rs src-tauri/src/lib.rs
git commit -m "feat(history): expose stored analyses via list/get/delete/clear commands"
```

---

## Task 5: History UI

**Files:**
- Modify: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/stores/settings.svelte.ts`, `src/lib/stores/analysis.svelte.ts`
- Create: `src/lib/history.ts`, `src/lib/history.test.ts`
- Create: `src/routes/history/+page.svelte`
- Modify: `src/routes/+page.svelte`, `src/routes/settings/+page.svelte`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: Task 4's commands; `ConfirmModal` and `ErrorState` from Tasks 1–2.
- Produces:
  - TS `interface HistoryEntry { id: string; created_at: number; preview: string; claim_count: number }`, `Settings.history_retention_days: number | null`, `HISTORY_RETENTION_OPTIONS`.
  - `api.ts`: `listHistory(query?, limit?)`, `getAnalysis(id)`, `deleteAnalysis(id)`, `clearHistory()`.
  - `history.ts`: `formatHistoryDate(ms: number, locale: 'cs' | 'en'): string`.
  - `analysisStore.show(analysis: Analysis): void`.

- [ ] **Step 1: Write the failing date-formatting test**

Create `src/lib/history.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { formatHistoryDate } from './history';

// 2026-05-30T14:05:00Z
const STAMP = Date.UTC(2026, 4, 30, 14, 5, 0);

describe('formatHistoryDate', () => {
  it('formats Czech with day-first order', () => {
    const formatted = formatHistoryDate(STAMP, 'cs');
    expect(formatted).toMatch(/30/);
    expect(formatted).toMatch(/2026/);
  });

  it('formats English', () => {
    const formatted = formatHistoryDate(STAMP, 'en');
    expect(formatted).toMatch(/2026/);
  });

  it('returns an empty string for a non-finite stamp', () => {
    expect(formatHistoryDate(Number.NaN, 'cs')).toBe('');
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm test -- src/lib/history.test.ts`
Expected: FAIL — `Cannot find module './history'`.

- [ ] **Step 3: Implement `src/lib/history.ts`**

```ts
import type { Locale } from './stores/i18n.svelte';

const LOCALE_TAGS: Record<Locale, string> = { cs: 'cs-CZ', en: 'en-US' };

/** Absolute, locale-aware timestamp for a history row. */
export function formatHistoryDate(ms: number, locale: Locale): string {
  if (!Number.isFinite(ms)) return '';

  return new Intl.DateTimeFormat(LOCALE_TAGS[locale], {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(ms));
}
```

- [ ] **Step 4: Run it to verify it passes**

Run: `pnpm test -- src/lib/history.test.ts`
Expected: PASS (3 tests).

- [ ] **Step 5: Add the types**

In `src/lib/types.ts`:

```ts
export interface HistoryEntry {
  id: string;
  created_at: number;
  preview: string;
  claim_count: number;
}
```

add to `interface Settings`:

```ts
  /** Days to keep local analysis history. `null` keeps it forever. */
  history_retention_days: number | null;
```

and:

```ts
export const DEFAULT_HISTORY_RETENTION_DAYS = 90;

/** Selectable retention windows for the settings UI. `null` = keep forever. */
export const HISTORY_RETENTION_OPTIONS: Array<number | null> = [7, 30, 90, 365, null];
```

Then add `history_retention_days: DEFAULT_HISTORY_RETENTION_DAYS,` to `browserDefaultSettings()` in `api.ts`, to `defaults` in `stores/settings.svelte.ts`, to the `cliSettings` fixture in `preflight.test.ts`, to the `draft` fixture in `onboarding.test.ts`, and to the `base` fixture in `sendSummary.test.ts`.

- [ ] **Step 6: Add the API wrappers**

In `src/lib/api.ts`, add near the other invoke wrappers:

```ts
/** In-memory stand-in for the SQLite history when running in browser preview. */
const browserHistory: Analysis[] = [];

export async function listHistory(query?: string, limit = 50): Promise<HistoryEntry[]> {
  if (isTauriRuntime()) return invoke<HistoryEntry[]>('list_history', { query, limit });

  const needle = query?.trim().toLowerCase() ?? '';
  return browserHistory
    .filter((analysis) => !needle || analysis.input.toLowerCase().includes(needle))
    .slice()
    .sort((a, b) => b.created_at - a.created_at)
    .slice(0, limit)
    .map((analysis) => ({
      id: analysis.id,
      created_at: analysis.created_at,
      preview: analysis.input.split(/\s+/).join(' ').slice(0, 160),
      claim_count: analysis.claims.length,
    }));
}

export async function getAnalysis(id: string): Promise<Analysis> {
  if (isTauriRuntime()) return invoke<Analysis>('get_analysis', { id });

  const found = browserHistory.find((analysis) => analysis.id === id);
  if (!found) throw { code: 'not_found', message: `analysis ${id}` };
  return found;
}

export async function deleteAnalysis(id: string): Promise<void> {
  if (isTauriRuntime()) {
    await invoke('delete_analysis', { id });
    return;
  }

  const index = browserHistory.findIndex((analysis) => analysis.id === id);
  if (index >= 0) browserHistory.splice(index, 1);
}

export async function clearHistory(): Promise<number> {
  if (isTauriRuntime()) return invoke<number>('clear_history');

  const removed = browserHistory.length;
  browserHistory.length = 0;
  return removed;
}
```

Import `HistoryEntry` in the type import list, and in the browser branch of `analyzeText`, after `const analysis = buildBrowserAnalysis(...)`, add `browserHistory.push(analysis);` so the preview build has something to list.

- [ ] **Step 7: Let the analysis store display a stored analysis**

In `src/lib/stores/analysis.svelte.ts`, add to the exported object:

```ts
  /** Load an already-completed analysis (e.g. from history) into the view. */
  show(analysis: Analysis): void {
    current = analysis;
    status = 'done';
    selectedId = analysis.claims[0]?.id ?? null;
    error = null;
  },
```

- [ ] **Step 8: Add the history copy**

`src/lib/i18n/cs.json`:

```json
  "history": {
    "title": "Historie",
    "search_label": "Hledat v historii",
    "search_placeholder": "Hledat v uložených analýzách…",
    "empty": "Zatím tu nic není. Analýzy se sem ukládají automaticky.",
    "empty_search": "Nic neodpovídá hledání.",
    "claims": "{count} tvrzení",
    "open": "Otevřít",
    "delete": "Smazat",
    "delete_confirm_title": "Smazat tuhle analýzu?",
    "delete_confirm_body": "Záznam se z počítače smaže natrvalo.",
    "clear_all": "Smazat celou historii",
    "clear_confirm_title": "Smazat celou historii?",
    "clear_confirm_body": "Všechny uložené analýzy se z počítače smažou natrvalo. Tohle nejde vrátit.",
    "cleared": "Smazáno záznamů: {count}",
    "retention_label": "Jak dlouho uchovávat historii",
    "retention_forever": "Navždy",
    "retention_days": "{days} dní",
    "retention_hint": "Starší analýzy se při startu aplikace smažou. „Navždy“ nemaže nic."
  },
```

`src/lib/i18n/en.json`:

```json
  "history": {
    "title": "History",
    "search_label": "Search history",
    "search_placeholder": "Search saved analyses…",
    "empty": "Nothing here yet. Analyses are saved automatically.",
    "empty_search": "Nothing matches that search.",
    "claims": "{count} claims",
    "open": "Open",
    "delete": "Delete",
    "delete_confirm_title": "Delete this analysis?",
    "delete_confirm_body": "The record is permanently removed from your computer.",
    "clear_all": "Delete all history",
    "clear_confirm_title": "Delete all history?",
    "clear_confirm_body": "Every saved analysis is permanently removed from your computer. This cannot be undone.",
    "cleared": "Deleted {count} records",
    "retention_label": "How long to keep history",
    "retention_forever": "Forever",
    "retention_days": "{days} days",
    "retention_hint": "Older analyses are deleted when the app starts. 'Forever' deletes nothing."
  },
```

- [ ] **Step 9: Create `src/routes/history/+page.svelte`**

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { clearHistory, deleteAnalysis, getAnalysis, listHistory } from '$lib/api';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import ErrorState from '$lib/components/ErrorState.svelte';
  import { toAppError, type AppErrorPayload } from '$lib/errors';
  import { formatHistoryDate } from '$lib/history';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { getLocale, t, tf } from '$lib/stores/i18n.svelte';
  import type { HistoryEntry } from '$lib/types';

  let entries = $state<HistoryEntry[]>([]);
  let query = $state('');
  let loading = $state(true);
  let failure = $state<AppErrorPayload | null>(null);
  let pendingDelete = $state<HistoryEntry | null>(null);
  let confirmClear = $state(false);
  let notice = $state<string | null>(null);
  let debounce: ReturnType<typeof setTimeout> | null = null;

  async function refresh() {
    loading = true;
    failure = null;
    try {
      entries = await listHistory(query);
    } catch (caught) {
      failure = toAppError(caught);
    } finally {
      loading = false;
    }
  }

  function onSearch() {
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(() => void refresh(), 200);
  }

  async function open(entry: HistoryEntry) {
    failure = null;
    try {
      analysisStore.show(await getAnalysis(entry.id));
      await goto(resolve('/'));
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  async function confirmDelete() {
    const entry = pendingDelete;
    pendingDelete = null;
    if (!entry) return;

    try {
      await deleteAnalysis(entry.id);
      await refresh();
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  async function confirmClearAll() {
    confirmClear = false;
    try {
      const removed = await clearHistory();
      notice = tf('history.cleared', { count: removed });
      await refresh();
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  onMount(refresh);
</script>

<main id="main" class="page">
  <header class="topbar glass">
    <button type="button" onclick={() => goto(resolve('/'))}>{t('settings.back')}</button>
    <h1>{t('history.title')}</h1>
    <div class="spacer"></div>
    <button type="button" onclick={() => (confirmClear = true)} disabled={entries.length === 0}>
      {t('history.clear_all')}
    </button>
  </header>

  <label class="search">
    <span class="sr-only">{t('history.search_label')}</span>
    <input
      type="search"
      bind:value={query}
      oninput={onSearch}
      placeholder={t('history.search_placeholder')}
    />
  </label>

  {#if failure}
    <ErrorState error={failure} onRetry={refresh} onDismiss={() => (failure = null)} />
  {/if}

  {#if notice}
    <p class="notice" role="status">{notice}</p>
  {/if}

  <section class="list" aria-busy={loading}>
    {#if loading}
      <p class="muted">{t('summary.analyzing')}</p>
    {:else if entries.length === 0}
      <p class="muted">{query.trim() ? t('history.empty_search') : t('history.empty')}</p>
    {:else}
      <ul>
        {#each entries as entry (entry.id)}
          <li class="row glass">
            <div class="meta">
              <time datetime={new Date(entry.created_at).toISOString()}>
                {formatHistoryDate(entry.created_at, getLocale())}
              </time>
              <span class="count">{tf('history.claims', { count: entry.claim_count })}</span>
            </div>
            <p class="preview">{entry.preview}</p>
            <div class="row-actions">
              <button type="button" class="primary" onclick={() => open(entry)}>
                {t('history.open')}
              </button>
              <button type="button" onclick={() => (pendingDelete = entry)}>
                {t('history.delete')}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <ConfirmModal
    open={pendingDelete !== null}
    title={t('history.delete_confirm_title')}
    confirmLabel={t('history.delete')}
    cancelLabel={t('common.cancel')}
    onConfirm={confirmDelete}
    onCancel={() => (pendingDelete = null)}
  >
    <p>{t('history.delete_confirm_body')}</p>
  </ConfirmModal>

  <ConfirmModal
    open={confirmClear}
    title={t('history.clear_confirm_title')}
    confirmLabel={t('history.clear_all')}
    cancelLabel={t('common.cancel')}
    onConfirm={confirmClearAll}
    onCancel={() => (confirmClear = false)}
  >
    <p>{t('history.clear_confirm_body')}</p>
  </ConfirmModal>
</main>

<style>
  .page {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    height: 100vh;
    max-width: 980px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-4);
    gap: var(--space-3);
    overflow: hidden;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    flex: 0 0 auto;
  }
  h1 {
    margin: 0;
    font-size: 22px;
    letter-spacing: -0.01em;
  }
  .spacer {
    flex: 1;
  }
  .search input {
    width: 100%;
  }
  .list {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
  }
  ul {
    display: grid;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .row {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }
  .meta {
    display: flex;
    gap: var(--space-3);
    color: var(--text-muted);
    font-size: 12px;
  }
  .preview {
    margin: 0;
    color: var(--text);
    font-size: 14px;
    line-height: 1.5;
  }
  .row-actions {
    display: flex;
    gap: var(--space-2);
  }
  .muted {
    color: var(--text-muted);
    font-size: 14px;
  }
  .notice {
    margin: 0;
    color: var(--ok);
    font-size: 13px;
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
```

`.sr-only` is added globally in Task 7; until then add a local copy of the rule.

- [ ] **Step 10: Add navigation and the retention control**

In `src/routes/+page.svelte`'s `<nav>`, before the settings button:

```svelte
      <button type="button" onclick={() => goto(resolve('/history'))}>
        {t('common.history')}
      </button>
```

In `src/routes/settings/+page.svelte`, inside the privacy section from Task 2:

```svelte
    <label>
      <span>{t('history.retention_label')}</span>
      <select bind:value={local.history_retention_days}>
        {#each HISTORY_RETENTION_OPTIONS as option (option ?? 'forever')}
          <option value={option}>
            {option === null ? t('history.retention_forever') : tf('history.retention_days', { days: option })}
          </option>
        {/each}
      </select>
    </label>
    <small class="hint">{t('history.retention_hint')}</small>
```

Import `HISTORY_RETENTION_OPTIONS` from `$lib/types` and `tf` from `$lib/stores/i18n.svelte`.

- [ ] **Step 11: Run everything**

Run: `pnpm test && pnpm check && pnpm lint`
Expected: PASS.

- [ ] **Step 12: Commit**

```bash
git add src/lib/history.ts src/lib/history.test.ts src/lib/api.ts src/lib/types.ts src/lib/stores/analysis.svelte.ts src/lib/stores/settings.svelte.ts src/routes/history/+page.svelte src/routes/+page.svelte src/routes/settings/+page.svelte src/lib/i18n/cs.json src/lib/i18n/en.json src/lib/preflight.test.ts src/lib/onboarding.test.ts src/lib/sendSummary.test.ts
git commit -m "feat(history): browsable, searchable, deletable analysis history"
```

---

## Task 6: Hotkey remapping

Today the hotkey is a free-text field: the user can type anything, `set_settings` accepts it, and the change only takes effect after a restart (`hotkey::install` runs once in `setup`). This task makes it a key-capture control, validates the accelerator before persisting, and re-registers it live.

**Files:**
- Create: `src/lib/hotkey.ts`, `src/lib/hotkey.test.ts`
- Create: `src/lib/components/HotkeyInput.svelte`
- Modify: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src/routes/settings/+page.svelte`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Produces:
  - Rust: `hotkey::normalize(accelerator: &str) -> AppResult<String>`, `hotkey::reinstall<R>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()>`.
  - TS: `DEFAULT_HOTKEY`, `type PlatformKind = 'mac' | 'other'`, `platformKind()`, `acceleratorFromEvent(event: KeyboardEvent): string | null`, `formatAccelerator(accelerator: string, platform: PlatformKind): string`, `isModifierOnly(event: KeyboardEvent): boolean`.
  - `HotkeyInput.svelte` props `{ value: string (bindable) }`.

- [ ] **Step 1: Write the failing accelerator tests**

Create `src/lib/hotkey.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { acceleratorFromEvent, formatAccelerator, isModifierOnly } from './hotkey';

function key(init: Partial<KeyboardEvent> & { code: string; key: string }): KeyboardEvent {
  return new KeyboardEvent('keydown', init);
}

describe('acceleratorFromEvent', () => {
  it('builds a portable accelerator from a letter plus modifiers', () => {
    expect(
      acceleratorFromEvent(key({ code: 'KeyD', key: 'd', metaKey: true, shiftKey: true })),
    ).toBe('CommandOrControl+Shift+D');
  });

  it('treats Ctrl and Meta as the same portable modifier', () => {
    expect(acceleratorFromEvent(key({ code: 'KeyD', key: 'd', ctrlKey: true }))).toBe(
      'CommandOrControl+D',
    );
  });

  it('orders modifiers deterministically', () => {
    expect(
      acceleratorFromEvent(
        key({ code: 'KeyK', key: 'k', altKey: true, shiftKey: true, ctrlKey: true }),
      ),
    ).toBe('CommandOrControl+Alt+Shift+K');
  });

  it('supports digits, function keys, and named keys', () => {
    expect(acceleratorFromEvent(key({ code: 'Digit1', key: '1', ctrlKey: true }))).toBe(
      'CommandOrControl+1',
    );
    expect(acceleratorFromEvent(key({ code: 'F5', key: 'F5' }))).toBe('F5');
    expect(acceleratorFromEvent(key({ code: 'Space', key: ' ', ctrlKey: true }))).toBe(
      'CommandOrControl+Space',
    );
    expect(acceleratorFromEvent(key({ code: 'ArrowUp', key: 'ArrowUp', altKey: true }))).toBe(
      'Alt+Up',
    );
  });

  it('rejects a bare letter with no modifier', () => {
    expect(acceleratorFromEvent(key({ code: 'KeyD', key: 'd' }))).toBeNull();
  });

  it('rejects a modifier-only press', () => {
    expect(acceleratorFromEvent(key({ code: 'ShiftLeft', key: 'Shift', shiftKey: true }))).toBeNull();
  });

  it('rejects an unsupported key', () => {
    expect(
      acceleratorFromEvent(key({ code: 'IntlBackslash', key: '<', ctrlKey: true })),
    ).toBeNull();
  });
});

describe('isModifierOnly', () => {
  it('detects modifier keys', () => {
    expect(isModifierOnly(key({ code: 'ControlLeft', key: 'Control' }))).toBe(true);
    expect(isModifierOnly(key({ code: 'KeyD', key: 'd' }))).toBe(false);
  });
});

describe('formatAccelerator', () => {
  it('renders mac glyphs', () => {
    expect(formatAccelerator('CommandOrControl+Shift+D', 'mac')).toBe('⌘⇧D');
    expect(formatAccelerator('Alt+Up', 'mac')).toBe('⌥Up');
  });

  it('renders Windows/Linux names', () => {
    expect(formatAccelerator('CommandOrControl+Shift+D', 'other')).toBe('Ctrl+Shift+D');
  });

  it('passes unknown tokens through', () => {
    expect(formatAccelerator('', 'other')).toBe('');
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm test -- src/lib/hotkey.test.ts`
Expected: FAIL — `Cannot find module './hotkey'`.

- [ ] **Step 3: Implement `src/lib/hotkey.ts`**

```ts
export const DEFAULT_HOTKEY = 'CommandOrControl+Shift+D';

export type PlatformKind = 'mac' | 'other';

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'AltGraph', 'OS']);

/**
 * `event.code` → accelerator token, for keys whose `code` name is not already
 * the token Tauri expects. Everything else is derived structurally.
 */
const NAMED_CODES: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Insert: 'Insert',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Minus: 'Minus',
  Equal: 'Equal',
  Comma: 'Comma',
  Period: 'Period',
  Slash: 'Slash',
  Backslash: 'Backslash',
  Semicolon: 'Semicolon',
  Quote: 'Quote',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  Backquote: 'Backquote',
};

export function isModifierOnly(event: KeyboardEvent): boolean {
  return MODIFIER_KEYS.has(event.key);
}

function mainKey(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  if (/^Numpad[0-9]$/.test(code)) return code;
  return NAMED_CODES[code] ?? null;
}

/**
 * Converts a keydown into a Tauri global-shortcut accelerator, or `null` when
 * the combination is not usable as a global hotkey. Requires at least one
 * modifier except for function keys, which are standalone-usable.
 */
export function acceleratorFromEvent(event: KeyboardEvent): string | null {
  if (isModifierOnly(event)) return null;

  const key = mainKey(event.code);
  if (!key) return null;

  const parts: string[] = [];
  // macOS reports Command as metaKey, Windows/Linux report Control as ctrlKey.
  // `CommandOrControl` is the portable token Tauri resolves per platform.
  if (event.metaKey || event.ctrlKey) parts.push('CommandOrControl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');

  const isFunctionKey = /^F([1-9]|1[0-9]|2[0-4])$/.test(key);
  if (parts.length === 0 && !isFunctionKey) return null;

  parts.push(key);
  return parts.join('+');
}

const MAC_GLYPHS: Record<string, string> = {
  CommandOrControl: '⌘',
  Command: '⌘',
  Control: '⌃',
  Alt: '⌥',
  Option: '⌥',
  Shift: '⇧',
};

const OTHER_NAMES: Record<string, string> = {
  CommandOrControl: 'Ctrl',
  Command: 'Ctrl',
  Control: 'Ctrl',
  Alt: 'Alt',
  Option: 'Alt',
  Shift: 'Shift',
};

/** Human-readable rendering of an accelerator for the current platform. */
export function formatAccelerator(accelerator: string, platform: PlatformKind): string {
  const tokens = accelerator.split('+').filter(Boolean);
  if (tokens.length === 0) return '';

  if (platform === 'mac') {
    return tokens.map((token) => MAC_GLYPHS[token] ?? token).join('');
  }

  return tokens.map((token) => OTHER_NAMES[token] ?? token).join('+');
}

export function platformKind(): PlatformKind {
  if (typeof navigator === 'undefined') return 'other';
  const source = navigator.userAgent ?? '';
  return /Mac|iPhone|iPad/i.test(source) ? 'mac' : 'other';
}
```

- [ ] **Step 4: Run it to verify it passes**

Run: `pnpm test -- src/lib/hotkey.test.ts`
Expected: PASS (11 tests).

- [ ] **Step 5: Write the failing Rust normalization tests**

Add to `src-tauri/src/hotkey.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_accepts_the_default_accelerator() {
        assert_eq!(
            normalize("CommandOrControl+Shift+D").unwrap(),
            "CommandOrControl+Shift+D"
        );
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(normalize("  Alt+F5  ").unwrap(), "Alt+F5");
    }

    #[test]
    fn normalize_rejects_empty_input() {
        assert!(normalize("   ").is_err());
    }

    #[test]
    fn normalize_rejects_garbage() {
        assert!(normalize("NotAKey+++").is_err());
    }
}
```

- [ ] **Step 6: Run them to verify they fail**

Run: `cd src-tauri && cargo test hotkey::tests`
Expected: FAIL — `cannot find function normalize`.

- [ ] **Step 7: Implement `normalize` and `reinstall`**

Add to `src-tauri/src/hotkey.rs`:

```rust
use crate::error::AppError;

pub const DEFAULT_ACCELERATOR: &str = "CommandOrControl+Shift+D";

/// Validates an accelerator string by parsing it the same way registration
/// will. Returns the trimmed form so callers persist a canonical value.
pub fn normalize(accelerator: &str) -> AppResult<String> {
    let trimmed = accelerator.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("hotkey cannot be empty".into()));
    }

    trimmed
        .parse::<Shortcut>()
        .map_err(|error| AppError::Invalid(format!("invalid hotkey '{trimmed}': {error}")))?;

    Ok(trimmed.to_string())
}

/// Drops every registered shortcut and registers `accelerator`. Used when the
/// user remaps the hotkey at runtime, so the change takes effect immediately
/// instead of at the next launch.
pub fn reinstall<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let accelerator = normalize(accelerator)?;
    app.global_shortcut().unregister_all()?;
    install(app, &accelerator)
}
```

Also replace the hardcoded fallback inside `install` with `DEFAULT_ACCELERATOR` so there is one source of truth:

```rust
        Err(error) => {
            warn!("hotkey {accelerator} invalid ({error}); falling back to default");
            DEFAULT_ACCELERATOR.parse().expect("default accelerator parses")
        }
```

- [ ] **Step 8: Run the Rust tests**

Run: `cd src-tauri && cargo test hotkey::tests`
Expected: PASS (4 tests).

- [ ] **Step 9: Validate and re-register on save**

In `src-tauri/src/commands/settings.rs`, rewrite `set_settings`:

```rust
#[tauri::command]
pub async fn set_settings<R: Runtime>(app: AppHandle<R>, settings: Settings) -> AppResult<()> {
    let mut settings = settings;
    settings.validate()?;
    settings.hotkey = crate::hotkey::normalize(&settings.hotkey)?;

    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AppError::Store(error.to_string()))?;

    let previous_hotkey = store
        .get(SETTINGS_KEY)
        .and_then(|value| serde_json::from_value::<Settings>(value).ok())
        .map(|previous| previous.hotkey);

    if previous_hotkey.as_deref() != Some(settings.hotkey.as_str()) {
        crate::hotkey::reinstall(&app, &settings.hotkey)?;
    }

    store.set(SETTINGS_KEY, json!(settings));
    store
        .save()
        .map_err(|error| AppError::Store(error.to_string()))?;
    Ok(())
}
```

Re-registering **before** persisting means a rejected accelerator (already taken by another app) surfaces as `ErrorCode::Hotkey` and the stored settings keep the working value.

- [ ] **Step 10: Implement `src/lib/components/HotkeyInput.svelte`**

```svelte
<script lang="ts">
  import {
    acceleratorFromEvent,
    DEFAULT_HOTKEY,
    formatAccelerator,
    isModifierOnly,
    platformKind,
  } from '$lib/hotkey';
  import { t } from '$lib/stores/i18n.svelte';

  let { value = $bindable(DEFAULT_HOTKEY) }: { value?: string } = $props();

  let recording = $state(false);
  let rejected = $state(false);

  const platform = platformKind();
  const label = $derived(formatAccelerator(value, platform));

  function onKeydown(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();
    event.stopPropagation();

    if (event.key === 'Escape') {
      recording = false;
      rejected = false;
      return;
    }

    if (isModifierOnly(event)) return;

    const accelerator = acceleratorFromEvent(event);
    if (!accelerator) {
      rejected = true;
      return;
    }

    value = accelerator;
    rejected = false;
    recording = false;
  }
</script>

<div class="hk">
  <button
    type="button"
    class="capture"
    class:recording
    aria-live="polite"
    onclick={() => {
      recording = !recording;
      rejected = false;
    }}
    onkeydown={onKeydown}
    onblur={() => (recording = false)}
  >
    {recording ? t('hotkey.recording') : label}
  </button>
  <button
    type="button"
    onclick={() => {
      value = DEFAULT_HOTKEY;
      rejected = false;
    }}
  >
    {t('hotkey.reset')}
  </button>
</div>
<small class="hint" role={rejected ? 'alert' : undefined}>
  {rejected ? t('hotkey.rejected') : t('hotkey.hint')}
</small>

<style>
  .hk {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .capture {
    min-width: 160px;
    font-variant-numeric: tabular-nums;
  }
  .capture.recording {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .hint {
    display: block;
    margin-top: var(--space-1);
    color: var(--text-subtle);
    font-size: 12px;
    line-height: 1.45;
  }
</style>
```

- [ ] **Step 11: Add the hotkey copy and swap the settings field**

`src/lib/i18n/cs.json`:

```json
  "hotkey": {
    "recording": "Stiskni kombinaci…",
    "reset": "Výchozí",
    "hint": "Klikni a stiskni kombinaci. Escape zruší. Musí obsahovat alespoň jeden modifikátor (nebo být funkční klávesa).",
    "rejected": "Tuhle kombinaci použít nejde. Zkus jinou."
  },
```

`src/lib/i18n/en.json`:

```json
  "hotkey": {
    "recording": "Press a combination…",
    "reset": "Default",
    "hint": "Click, then press a combination. Escape cancels. It needs at least one modifier (or be a function key).",
    "rejected": "That combination cannot be used. Try another."
  },
```

In `src/routes/settings/+page.svelte`, replace the hotkey text input:

```svelte
    <label>
      <span>{t('settings.hotkey_label')}</span>
      <HotkeyInput bind:value={local.hotkey} />
    </label>
```

and import `HotkeyInput from '$lib/components/HotkeyInput.svelte'`.

- [ ] **Step 12: Run everything**

Run: `pnpm test && pnpm check && pnpm lint && (cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings)`
Expected: PASS.

- [ ] **Step 13: Commit**

```bash
git add src/lib/hotkey.ts src/lib/hotkey.test.ts src/lib/components/HotkeyInput.svelte src-tauri/src/hotkey.rs src-tauri/src/commands/settings.rs src/routes/settings/+page.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(hotkey): key-capture remapping with live re-registration"
```

---

## Task 7: Accessibility and contrast

Two problems. **Screen readers:** claim spans are `role="button"` on `<span>` with no accessible description of what the colour means, streaming verification results never announce, and there is no skip link. **Contrast:** the glass surfaces are translucent over a coloured mesh, and `--text-subtle` (`#8b8b94` on `#eef0f4`) sits at ~2.9:1 — well under AA. This task fixes both and adds a regression test so the tokens cannot silently drift back.

**Files:**
- Create: `src/lib/contrast.ts`, `src/lib/contrast.test.ts`
- Modify: `src/lib/styles/tokens.css`, `src/app.css`
- Modify: `src/lib/components/ClaimText.svelte`, `src/lib/components/ClaimText.test.ts`
- Modify: `src/lib/components/SidePanel.svelte`, `src/lib/components/VerdictBanner.svelte`, `src/lib/components/PasteInput.svelte`, `src/lib/components/TierBadge.svelte`
- Modify: `src/routes/+page.svelte`, `src/routes/+layout.svelte`, `src/routes/settings/+page.svelte`
- Modify: `src/lib/stores/theme.svelte.ts`, `src/lib/theme.ts`, `src/lib/theme.test.ts`
- Modify: `src-tauri/src/storage/settings_store.rs`, `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/stores/settings.svelte.ts`
- Modify: `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Produces:
  - `src/lib/contrast.ts`: `parseHex(value: string): [number, number, number] | null`, `relativeLuminance(rgb): number`, `contrastRatio(a: string, b: string): number`.
  - `Settings.high_contrast: boolean` (Rust + TS, default `false`).
  - `theme.init(pref: ThemePref, highContrast: boolean)`, `theme.setContrast(next: boolean)`.
  - i18n namespace `a11y.*`.

- [ ] **Step 1: Write the failing contrast test**

Create `src/lib/contrast.test.ts`:

```ts
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { contrastRatio, parseHex, relativeLuminance } from './contrast';

const tokens = readFileSync(new URL('./styles/tokens.css', import.meta.url), 'utf8');

/** Reads a `--name: #rrggbb;` declaration from a block of tokens.css. */
function token(name: string, scope: 'light' | 'dark'): string {
  const block =
    scope === 'light'
      ? tokens.slice(tokens.indexOf(':root {'), tokens.indexOf("[data-theme='dark']"))
      : tokens.slice(tokens.indexOf("[data-theme='dark']"));
  const match = new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{3,8})`).exec(block);
  if (!match) throw new Error(`token --${name} not found in ${scope} scope`);
  return match[1];
}

describe('contrast helpers', () => {
  it('parses 6- and 3-digit hex', () => {
    expect(parseHex('#ffffff')).toEqual([255, 255, 255]);
    expect(parseHex('#fff')).toEqual([255, 255, 255]);
    expect(parseHex('nope')).toBeNull();
  });

  it('computes the reference luminances', () => {
    expect(relativeLuminance([255, 255, 255])).toBeCloseTo(1, 5);
    expect(relativeLuminance([0, 0, 0])).toBeCloseTo(0, 5);
  });

  it('computes the reference ratio', () => {
    expect(contrastRatio('#ffffff', '#000000')).toBeCloseTo(21, 2);
  });
});

describe('token contrast meets WCAG AA', () => {
  const pairs: Array<[string, string, string]> = [
    ['text', 'bg', 'light'],
    ['text-muted', 'bg', 'light'],
    ['text-subtle', 'bg', 'light'],
    ['ok', 'bg', 'light'],
    ['bad', 'bg', 'light'],
    ['warn', 'bg', 'light'],
    ['accent', 'bg', 'light'],
    ['tier-a-fg', 'tier-a-bg', 'light'],
    ['tier-b-fg', 'tier-b-bg', 'light'],
    ['tier-c-fg', 'tier-c-bg', 'light'],
    ['tier-d-fg', 'tier-d-bg', 'light'],
    ['text', 'bg', 'dark'],
    ['text-muted', 'bg', 'dark'],
    ['text-subtle', 'bg', 'dark'],
    ['ok', 'bg', 'dark'],
    ['bad', 'bg', 'dark'],
    ['warn', 'bg', 'dark'],
    ['accent', 'bg', 'dark'],
  ];

  for (const [fg, bg, scope] of pairs) {
    it(`--${fg} on --${bg} (${scope}) is at least 4.5:1`, () => {
      const ratio = contrastRatio(
        token(fg, scope as 'light' | 'dark'),
        token(bg, scope as 'light' | 'dark'),
      );
      expect(ratio).toBeGreaterThanOrEqual(4.5);
    });
  }
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm test -- src/lib/contrast.test.ts`
Expected: FAIL — module missing; once the module exists, several token pairs fail the 4.5 threshold.

- [ ] **Step 3: Implement `src/lib/contrast.ts`**

```ts
export type Rgb = [number, number, number];

/** Parses `#rgb` / `#rrggbb` / `#rrggbbaa` (alpha ignored). */
export function parseHex(value: string): Rgb | null {
  const match = /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(value.trim());
  if (!match) return null;

  const digits = match[1];
  const expanded =
    digits.length === 3
      ? digits
          .split('')
          .map((digit) => digit + digit)
          .join('')
      : digits.slice(0, 6);

  return [
    Number.parseInt(expanded.slice(0, 2), 16),
    Number.parseInt(expanded.slice(2, 4), 16),
    Number.parseInt(expanded.slice(4, 6), 16),
  ];
}

/** WCAG 2.1 relative luminance. */
export function relativeLuminance(rgb: Rgb): number {
  const [r, g, b] = rgb.map((channel) => {
    const normalized = channel / 255;
    return normalized <= 0.04045
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  });

  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG 2.1 contrast ratio between two hex colours. Returns 0 on bad input. */
export function contrastRatio(a: string, b: string): number {
  const first = parseHex(a);
  const second = parseHex(b);
  if (!first || !second) return 0;

  const lighter = Math.max(relativeLuminance(first), relativeLuminance(second));
  const darker = Math.min(relativeLuminance(first), relativeLuminance(second));
  return (lighter + 0.05) / (darker + 0.05);
}
```

- [ ] **Step 4: Fix the failing tokens**

In `src/lib/styles/tokens.css`, light `:root`:

```css
  --text-muted: #4b4b55;
  --text-subtle: #62626d;
  --accent: #4338ca;
  --accent-hover: #3730a3;
```

dark `[data-theme='dark']`:

```css
  --text-muted: #c3c6cf;
  --text-subtle: #a2a6b0;
```

Re-run the test after each edit and adjust until every pair passes. Do not lower the threshold — darken the foreground.

Then append the contrast modes to `tokens.css`:

```css
/* High contrast — opt-in via Settings (`data-contrast="more"`) or the OS.
   Two independent axes: `data-theme` picks light/dark, `data-contrast` picks
   the contrast level. Both are always set explicitly by the theme store. */
:root[data-contrast='more'] {
  --surface-glass: rgba(255, 255, 255, 0.96);
  --surface-glass-strong: rgba(255, 255, 255, 1);
  --surface-glass-border: rgba(17, 17, 26, 0.45);
  --glass-blur: 0px;
  --glass-sat: 100%;
  --text-muted: #2b2b33;
  --text-subtle: #3a3a44;
  --mesh-1: transparent;
  --mesh-2: transparent;
  --mesh-3: transparent;
}

[data-theme='dark'][data-contrast='more'] {
  --surface-glass: rgba(12, 13, 18, 0.96);
  --surface-glass-strong: rgba(12, 13, 18, 1);
  --surface-glass-border: rgba(255, 255, 255, 0.5);
  --text-muted: #e6e8ee;
  --text-subtle: #cfd2da;
  --mesh-1: transparent;
  --mesh-2: transparent;
  --mesh-3: transparent;
}

@media (prefers-contrast: more) {
  :root:not([data-contrast='normal']) {
    --surface-glass: rgba(255, 255, 255, 0.96);
    --surface-glass-strong: rgba(255, 255, 255, 1);
    --surface-glass-border: rgba(17, 17, 26, 0.45);
    --glass-blur: 0px;
    --glass-sat: 100%;
  }
}

/* Windows High Contrast / forced-colors: hand control to the OS palette. */
@media (forced-colors: active) {
  .glass {
    background: Canvas;
    border-color: CanvasText;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    box-shadow: none;
  }

  .app-mesh {
    background: Canvas;
  }
}
```

- [ ] **Step 5: Add `high_contrast` to settings (Rust first)**

Add to the `tests` module in `src-tauri/src/storage/settings_store.rs`:

```rust
    #[test]
    fn high_contrast_defaults_to_false() {
        assert!(!Settings::default().high_contrast);
    }

    #[test]
    fn legacy_settings_without_high_contrast_default_to_false() {
        let legacy = r#"{"locale":"cs","hotkey":"CommandOrControl+Shift+D","cache_ttl_days":7,"onboarded":false}"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert!(!parsed.high_contrast);
    }
```

Run `cd src-tauri && cargo test storage::settings_store` — FAIL. Then add the field:

```rust
    /// Forces the high-contrast palette regardless of the OS setting: opaque
    /// surfaces, no blur, no mesh background.
    #[serde(default)]
    pub high_contrast: bool,
```

with `high_contrast: false,` in `impl Default`. Re-run — PASS.

Mirror it on the frontend: `Settings.high_contrast: boolean` in `types.ts`, `high_contrast: false` in `browserDefaultSettings()`, in the store `defaults`, and in every test fixture (`preflight.test.ts`, `onboarding.test.ts`, `sendSummary.test.ts`).

- [ ] **Step 6: Apply the contrast attribute from the theme store**

`src/lib/theme.ts` — add:

```ts
export type ContrastAttr = 'more' | 'normal';

export function resolveContrast(highContrast: boolean): ContrastAttr {
  return highContrast ? 'more' : 'normal';
}
```

`src/lib/theme.test.ts` — add:

```ts
import { resolveContrast } from './theme';

describe('resolveContrast', () => {
  it('maps the preference onto the data-contrast attribute', () => {
    expect(resolveContrast(true)).toBe('more');
    expect(resolveContrast(false)).toBe('normal');
  });
});
```

`src/lib/stores/theme.svelte.ts` — track and apply it:

```ts
import { resolveContrast, resolveTheme, type ResolvedTheme } from '$lib/theme';

let pref = $state<ThemePref>('auto');
let resolved = $state<ResolvedTheme>('light');
let highContrast = $state(false);

function apply(): void {
  resolved = resolveTheme(pref, prefersDark());
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', resolved);
    document.documentElement.setAttribute('data-contrast', resolveContrast(highContrast));
  }
}
```

with `init(initial: ThemePref, initialContrast = false)` setting both, a `get highContrast()` accessor, and:

```ts
  setContrast(next: boolean): void {
    highContrast = next;
    apply();
  },
```

Setting `data-contrast="normal"` explicitly is what lets the `:root:not([data-contrast='normal'])` media rule stand down when the user has opted out of high contrast in-app.

- [ ] **Step 7: Add the toggle to Settings**

In the privacy/appearance area of `src/routes/settings/+page.svelte`:

```svelte
    <label class="check">
      <input
        type="checkbox"
        bind:checked={local.high_contrast}
        onchange={() => theme.setContrast(local.high_contrast)}
      />
      <span>{t('a11y.high_contrast_label')}</span>
    </label>
    <small class="hint">{t('a11y.high_contrast_hint')}</small>
```

Import `theme` from `$lib/stores/theme.svelte`.

- [ ] **Step 8: Write the failing ClaimText accessibility test**

Replace `src/lib/components/ClaimText.test.ts`'s content (keeping any existing segmentation assertions) with tests that pin the semantics:

```ts
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ClaimText from './ClaimText.svelte';
import { setLocale } from '$lib/stores/i18n.svelte';
import type { Claim } from '$lib/types';

afterEach(cleanup);

const claims: Claim[] = [
  {
    id: 'c1',
    text: 'Karel IV.',
    start: 0,
    end: 9,
    kind: 'fact',
    reason: 'r',
    verification: { status: 'supported', sources: [], summary: 's' },
  },
  {
    id: 'c2',
    text: 'nejlepší',
    start: 10,
    end: 18,
    kind: 'opinion',
    reason: 'r',
    verification: null,
  },
];

function setup(selectedId: string | null = null, onSelect = vi.fn()) {
  setLocale('en');
  render(ClaimText, {
    props: { input: 'Karel IV. nejlepší', claims, selectedId, onSelect },
  });
  return onSelect;
}

describe('ClaimText accessibility', () => {
  it('renders each claim as a real button', () => {
    setup();
    expect(screen.getAllByRole('button')).toHaveLength(2);
  });

  it('names each claim with its text and epistemic kind', () => {
    setup();
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAccessibleName(
      /Verifiable fact/,
    );
    expect(screen.getByRole('button', { name: /nejlepší/ })).toHaveAccessibleName(/Opinion/);
  });

  it('announces the verification status when there is one', () => {
    setup();
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAccessibleName(/Verified/);
  });

  it('marks the selected claim with aria-pressed', () => {
    setup('c1');
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('selects the next claim with ArrowRight', async () => {
    const onSelect = setup('c1');
    const first = screen.getByRole('button', { name: /Karel IV\./ });
    await fireEvent.keyDown(first, { key: 'ArrowRight' });
    expect(onSelect).toHaveBeenCalledWith('c2');
  });

  it('wraps around from the last claim to the first', async () => {
    const onSelect = setup('c2');
    const last = screen.getByRole('button', { name: /nejlepší/ });
    await fireEvent.keyDown(last, { key: 'ArrowRight' });
    expect(onSelect).toHaveBeenCalledWith('c1');
  });
});
```

Run: `pnpm test -- src/lib/components/ClaimText.test.ts` → FAIL.

- [ ] **Step 9: Rewrite `ClaimText.svelte` with real buttons**

```svelte
<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
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

  type Segment = { kind: 'plain'; text: string } | { kind: 'claim'; claim: Claim };

  const segments = $derived(buildSegments(input, claims));
  const order = $derived(
    segments.filter((s): s is { kind: 'claim'; claim: Claim } => s.kind === 'claim'),
  );

  function buildSegments(text: string, list: Claim[]): Segment[] {
    if (list.length === 0) return [{ kind: 'plain', text }];

    const sorted = [...list].sort((a, b) => a.start - b.start);
    const out: Segment[] = [];
    let cursor = 0;

    for (const claim of sorted) {
      if (claim.start < cursor) continue;
      if (claim.start > cursor) out.push({ kind: 'plain', text: text.slice(cursor, claim.start) });
      out.push({ kind: 'claim', claim });
      cursor = claim.end;
    }

    if (cursor < text.length) out.push({ kind: 'plain', text: text.slice(cursor) });
    return out;
  }

  /**
   * The colour alone carries the classification, which fails WCAG 1.4.1. The
   * label repeats it as text for screen readers, along with the verification
   * verdict once it arrives.
   */
  function claimLabel(claim: Claim): string {
    const parts = [t(`sidepanel.kind_${claim.kind}`)];
    if (claim.verification) parts.push(t(`status.${claim.verification.status}`));
    return parts.join(', ');
  }

  function onClaimKeydown(event: KeyboardEvent, id: string) {
    const index = order.findIndex((segment) => segment.claim.id === id);
    if (index < 0 || order.length === 0) return;

    const step =
      event.key === 'ArrowRight' || event.key === 'ArrowDown'
        ? 1
        : event.key === 'ArrowLeft' || event.key === 'ArrowUp'
          ? -1
          : 0;

    if (step === 0) return;

    event.preventDefault();
    const next = order[(index + step + order.length) % order.length];
    onSelect(next.claim.id);
  }
</script>

<p class="ct" role="group" aria-label={t('a11y.claims_group')}>
  {#each segments as segment, index (index)}
    {#if segment.kind === 'plain'}<span class="plain">{segment.text}</span>{:else}<button
        type="button"
        class="claim kind-{segment.claim.kind}"
        class:selected={segment.claim.id === selectedId}
        data-id={segment.claim.id}
        aria-pressed={segment.claim.id === selectedId}
        tabindex={segment.claim.id === selectedId || (selectedId === null && index === 0) ? 0 : -1}
        onclick={() => onSelect(segment.claim.id)}
        onkeydown={(event) => onClaimKeydown(event, segment.claim.id)}
        >{segment.claim.text}<span class="sr-only"> ({claimLabel(segment.claim)})</span></button
      >{/if}
  {/each}
</p>

<style>
  .ct {
    margin: 0;
    line-height: 1.75;
    font-size: 15px;
    white-space: pre-wrap;
    color: var(--text);
  }

  .claim {
    display: inline;
    margin: 0;
    padding: 1px 3px;
    border: 0;
    border-radius: var(--radius-sm);
    color: inherit;
    font: inherit;
    text-align: left;
    white-space: pre-wrap;
    cursor: pointer;
    outline: 2px solid transparent;
    transition:
      outline-color var(--dur-fast) var(--ease),
      background var(--dur-fast) var(--ease);
  }

  .claim:hover {
    background: var(--accent-soft);
  }

  .claim.selected {
    outline-color: var(--accent);
  }

  /* Redundant, non-colour cue so classification survives colour-blindness,
     greyscale printing, and forced-colors mode. */
  .kind-fact {
    background: var(--ok-soft);
    box-shadow: inset 0 -2px 0 var(--ok);
  }
  .kind-inference {
    background: var(--warn-soft);
    box-shadow: inset 0 -2px 0 var(--warn);
  }
  .kind-opinion {
    background: var(--neutral-soft);
    box-shadow: inset 0 -2px 0 var(--neutral);
  }
  .kind-contradiction {
    background: var(--bad-soft);
    box-shadow: inset 0 -2px 0 var(--bad);
  }
</style>
```

The `{#if}`/`{:else}` and the button's children are deliberately written without surrounding whitespace so `white-space: pre-wrap` does not gain stray spaces around every claim.

- [ ] **Step 10: Add the `a11y` copy and the global `.sr-only` utility**

`src/lib/i18n/cs.json`:

```json
  "a11y": {
    "skip_to_content": "Přeskočit na obsah",
    "claims_group": "Rozebraná tvrzení",
    "status_region": "Průběh analýzy",
    "sources_region": "Detail vybraného tvrzení",
    "high_contrast_label": "Vysoký kontrast",
    "high_contrast_hint": "Vypne průhlednost a rozostření pozadí. Zapíná se i automaticky podle nastavení systému."
  },
```

`src/lib/i18n/en.json`:

```json
  "a11y": {
    "skip_to_content": "Skip to content",
    "claims_group": "Extracted claims",
    "status_region": "Analysis progress",
    "sources_region": "Selected claim detail",
    "high_contrast_label": "High contrast",
    "high_contrast_hint": "Turns off translucency and background blur. Also switches on automatically based on your system setting."
  },
```

Add to `src/app.css`:

```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.skip-link {
  position: absolute;
  top: -100px;
  left: var(--space-3);
  z-index: 100;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--accent-contrast);
  text-decoration: none;
  transition: top var(--dur-fast) var(--ease);
}

.skip-link:focus {
  top: var(--space-3);
}
```

and replace `ThemeToggle.svelte`'s local `.sr` class with `sr-only` (Svelte scopes component styles, so the global class must come from `app.css`; keep the markup using `class="sr-only"` and delete the local rule).

- [ ] **Step 11: Wire the landmarks and live regions**

`src/routes/+layout.svelte` — add the skip link (shown in Task 3, Step 7) and `theme.init(settings.current.theme, settings.current.high_contrast)`.

`src/routes/+page.svelte`:
- `<main id="main" class="page">`
- wrap the result block:

```svelte
  <section
    class="result"
    aria-live="polite"
    aria-busy={analysisStore.status === 'running'}
    aria-label={t('a11y.status_region')}
  >
```

`src/lib/components/SidePanel.svelte` — make the panel a labelled complementary landmark and let verification updates announce:

```svelte
<aside class="sp glass" aria-label={t('a11y.sources_region')}>
```

and on the sources `<section>`:

```svelte
    <section aria-live="polite">
```

`src/lib/components/VerdictBanner.svelte` — the `✓ / ✕ / ~ / ?` glyph is `aria-hidden`, and the headline text already carries the meaning; add `role="status"` to the banner so the aggregate verdict is announced when it settles.

`src/lib/components/PasteInput.svelte` — give each textarea an explicit `id` and the wrapper `aria-describedby` pointing at a `.sr-only` hint explaining that drag & drop is supported, so the drop target is discoverable non-visually.

`src/lib/components/TierBadge.svelte` — verify the badge exposes its tier as text, not only as a colour; if it renders a bare letter, add `<span class="sr-only">{t('tier.' + tier)}</span>`.

- [ ] **Step 12: Run the whole suite**

Run: `pnpm test && pnpm check && pnpm lint && (cd src-tauri && cargo test)`
Expected: PASS, including all contrast pairs.

- [ ] **Step 13: Manual screen-reader pass (record the result in the commit body)**

macOS VoiceOver (`Cmd+F5`):
1. `VO+U` → Landmarks: expect `main`, `complementary` (side panel), and the skip link as the first tab stop.
2. Tab to a claim: expect "«claim text» (Verifiable fact, Verified), toggle button".
3. Arrow between claims: expect each new selection announced.
4. Run an analysis: expect the progress region to announce "Analyzing…" and then the verdict.

Windows NVDA (`Insert+Space` for focus mode):
1. `D` cycles landmarks — same three.
2. `Tab` reaches every button, select, checkbox, and the modal traps focus.
3. `Insert+F7` element list → Buttons: every claim appears with its kind.

Also check Windows High Contrast (`Left Alt+Left Shift+Print Screen`): the glass surfaces must become opaque `Canvas`, and all text must stay readable.

- [ ] **Step 14: Commit**

```bash
git add src/lib/contrast.ts src/lib/contrast.test.ts src/lib/styles/tokens.css src/app.css src/lib/components src/routes src/lib/theme.ts src/lib/theme.test.ts src/lib/stores/theme.svelte.ts src/lib/types.ts src/lib/api.ts src/lib/stores/settings.svelte.ts src-tauri/src/storage/settings_store.rs src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "feat(a11y): screen-reader semantics, AA contrast tokens, high-contrast mode"
```

---

## Task 8: CLI provider — retrofit spec coverage and tests

The local-CLI LLM provider (`src-tauri/src/llm/cli.rs`, `src/lib/cliPresets.ts`, `ProviderKind::Cli`, the `*_cli_*.txt` prompts) was built after M2 and never entered the plan documents. It is now the **default** provider — `Settings::default()` returns `ProviderKind::Cli` — which contradicts the overview's cross-cutting decision table ("Anthropic Claude Haiku 4.5 only"). This task closes both gaps: the docs and the test coverage.

**Files:**
- Modify: `docs/superpowers/plans/2026-05-20-druhy-nazor-00-overview.md`
- Modify: `src/lib/cliPresets.test.ts`
- Create: `src/lib/i18n/parity.test.ts`
- Modify: `src-tauri/src/llm/cli.rs`
- Modify: `src-tauri/src/llm/prompts/mod.rs` (tests only)

- [ ] **Step 1: Correct the overview's cross-cutting decisions**

In §4 of `docs/superpowers/plans/2026-05-20-druhy-nazor-00-overview.md`, replace the "LLM provider in MVP" row and add three rows:

```markdown
| LLM provider (default)   | Local CLI runner via `ProviderKind::Cli` — the user's own `claude` / `codex` / `ollama` binary, prompt on stdin, JSON on stdout | Zero marginal cost, no BYO cloud key needed to start, and the text never leaves the machine by the app's own doing. Added after M2; supersedes "Anthropic only". |
| LLM provider (cloud)     | Anthropic Claude Haiku 4.5 via `ProviderKind::Anthropic`, BYO key                                                             | Still supported and still the best CZ quality/latency when the user wants it.                                                                                   |
| CLI output contract      | First balanced JSON object on stdout; markdown fences tolerated; unescaped quotes inside strings repaired once before failing  | Local runners wrap answers in prose and fences. Being strict here would make the default provider unusable.                                                      |
| CLI prompt variants      | Separate `*_cli_cs.txt` / `*_cli_en.txt` prompts per stage                                                                    | CLI runners have no tool-use schema, so they need explicit "print only JSON" instructions the Anthropic prompts do not carry.                                    |
```

- [ ] **Step 2: Add the missing spec-coverage rows**

In §8 of the overview, append:

```markdown
| Local CLI LLM provider (default)          | `04-privacy-polish.md` Task 8 (retrofit); implemented in `src-tauri/src/llm/cli.rs`, `CliProvider::{new,atomize,judge}`                                                    |
| CLI provider presets in Settings          | `04-privacy-polish.md` Task 8 (retrofit); implemented in `src/lib/cliPresets.ts` + the provider section of `src/routes/settings/+page.svelte`                             |
| CLI binary discovery outside the GUI PATH | `04-privacy-polish.md` Task 8 (retrofit); `build_cli_path` / `resolve_program` in `src-tauri/src/llm/cli.rs`                                                              |
| Tolerant CLI JSON extraction and repair   | `04-privacy-polish.md` Task 8 (retrofit); `extract_json_object` / `repair_unescaped_string_quotes` in `src-tauri/src/llm/cli.rs`                                          |
| Provider-aware prompt selection           | `04-privacy-polish.md` Task 8 (retrofit); `atomize_prompt(locale, provider)` / `judge_prompt(locale, provider)` in `src-tauri/src/llm/prompts/mod.rs`                     |
| Pre-send disclosure of the destination    | `04-privacy-polish.md` Task 2 (`describeSend`, `SendConfirm.svelte`)                                                                                                      |
| First-run onboarding                      | `04-privacy-polish.md` Task 3                                                                                                                                            |
| Actionable error states                   | `04-privacy-polish.md` Task 1                                                                                                                                            |
| History view                              | `04-privacy-polish.md` Tasks 4–5                                                                                                                                         |
| Hotkey remapping in Settings              | `04-privacy-polish.md` Task 6                                                                                                                                            |
| Accessibility + high contrast             | `04-privacy-polish.md` Task 7                                                                                                                                            |
```

Also update §7's phase map so the `04-privacy-polish.md` bullet points at the real filename (it is `04-privacy-polish.md`, not date-prefixed) and mark it as written.

- [ ] **Step 3: Broaden the CLI preset tests**

Replace `src/lib/cliPresets.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import cs from './i18n/cs.json';
import en from './i18n/en.json';
import { CLI_PRESETS, commandToCliPreset, presetCommand } from './cliPresets';

describe('CLI presets', () => {
  it('offers multiple CLI-backed providers', () => {
    expect(CLI_PRESETS.map((preset) => preset.id)).toEqual(['claude', 'codex', 'ollama', 'custom']);
  });

  it('gives every non-custom preset a concrete command', () => {
    for (const preset of CLI_PRESETS) {
      if (preset.id === 'custom') {
        expect(preset.command).toBeNull();
      } else {
        expect(preset.command?.trim().length).toBeGreaterThan(0);
      }
    }
  });

  it('round-trips every preset command back to its id', () => {
    for (const preset of CLI_PRESETS) {
      if (preset.command === null) continue;
      expect(commandToCliPreset(preset.command)).toBe(preset.id);
      expect(presetCommand(preset.id)).toBe(preset.command);
    }
  });

  it('ignores surrounding whitespace when matching', () => {
    expect(commandToCliPreset('  claude -p  ')).toBe('claude');
  });

  it('falls back to custom for an unknown command', () => {
    expect(commandToCliPreset('my-llm --json')).toBe('custom');
    expect(commandToCliPreset('')).toBe('custom');
    expect(presetCommand('custom')).toBeNull();
  });

  it('has a label for every preset in both locales', () => {
    for (const preset of CLI_PRESETS) {
      expect(cs.settings).toHaveProperty(`cli_preset_${preset.id}`);
      expect(en.settings).toHaveProperty(`cli_preset_${preset.id}`);
    }
  });
});
```

Run: `pnpm test -- src/lib/cliPresets.test.ts` → PASS.

- [ ] **Step 4: Add the i18n parity test**

Create `src/lib/i18n/parity.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import cs from './cs.json';
import en from './en.json';

function flatten(node: unknown, prefix = ''): string[] {
  if (typeof node !== 'object' || node === null) return [prefix];

  return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
    flatten(value, prefix ? `${prefix}.${key}` : key),
  );
}

const csKeys = flatten(cs).sort();
const enKeys = flatten(en).sort();

describe('i18n bundles', () => {
  it('have identical key sets', () => {
    expect(csKeys.filter((key) => !enKeys.includes(key))).toEqual([]);
    expect(enKeys.filter((key) => !csKeys.includes(key))).toEqual([]);
  });

  it('have no empty strings', () => {
    for (const [locale, bundle] of [
      ['cs', cs],
      ['en', en],
    ] as const) {
      const empties = flatten(bundle).filter((key) => {
        const value = key
          .split('.')
          .reduce<unknown>(
            (node, part) => (node as Record<string, unknown> | undefined)?.[part],
            bundle,
          );
        return typeof value === 'string' && value.trim().length === 0;
      });
      expect(empties, `${locale} has empty values`).toEqual([]);
    }
  });

  it('use the same interpolation placeholders in both locales', () => {
    const placeholders = (bundle: unknown, key: string): string[] => {
      const value = key
        .split('.')
        .reduce<unknown>(
          (node, part) => (node as Record<string, unknown> | undefined)?.[part],
          bundle,
        );
      if (typeof value !== 'string') return [];
      return [...value.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
    };

    for (const key of csKeys) {
      expect(placeholders(cs, key), `mismatch for ${key}`).toEqual(placeholders(en, key));
    }
  });
});
```

Run: `pnpm test -- src/lib/i18n/parity.test.ts` → PASS (fix any key drift the earlier tasks introduced).

- [ ] **Step 5: Add the missing Rust CLI tests**

Append to the `tests` module in `src-tauri/src/llm/cli.rs`:

```rust
    #[test]
    fn build_cli_path_prefers_inherited_then_user_then_fallback_dirs() {
        let home = PathBuf::from("/home/tester");
        let path = build_cli_path(Some(&home), Some(OsStr::new("/inherited/bin")));
        let dirs: Vec<PathBuf> = env::split_paths(&path).collect();

        assert_eq!(dirs[0], PathBuf::from("/inherited/bin"));
        assert!(dirs.contains(&home.join(".local/bin")));
        assert!(dirs.contains(&PathBuf::from("/opt/homebrew/bin")));
    }

    #[test]
    fn build_cli_path_deduplicates_repeated_dirs() {
        let path = build_cli_path(None, Some(OsStr::new("/usr/bin:/usr/bin:/bin")));
        let dirs: Vec<PathBuf> = env::split_paths(&path).collect();
        let usr_bin = dirs.iter().filter(|d| *d == &PathBuf::from("/usr/bin")).count();

        assert_eq!(usr_bin, 1);
    }

    #[test]
    fn build_cli_path_skips_empty_entries() {
        let path = build_cli_path(None, Some(OsStr::new("")));
        let dirs: Vec<PathBuf> = env::split_paths(&path).collect();
        assert!(dirs.iter().all(|dir| !dir.as_os_str().is_empty()));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_program_accepts_an_absolute_path() {
        assert_eq!(
            resolve_program("/bin/sh", OsStr::new("/nowhere")).unwrap(),
            PathBuf::from("/bin/sh")
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolve_program_rejects_an_absolute_path_that_is_not_executable() {
        assert!(resolve_program("/etc/hosts", OsStr::new("/nowhere")).is_none());
    }

    #[test]
    fn repair_leaves_valid_json_untouched() {
        let json = r#"{"a":"b","c":[1,2]}"#;
        assert_eq!(repair_unescaped_string_quotes(json), json);
    }

    #[test]
    fn repair_escapes_only_quotes_that_do_not_close_a_string() {
        let broken = r#"{"text":"say "hi" now","kind":"fact"}"#;
        let repaired = repair_unescaped_string_quotes(broken);
        let parsed: serde_json::Value = serde_json::from_str(&repaired).unwrap();
        assert_eq!(parsed["text"], r#"say "hi" now"#);
        assert_eq!(parsed["kind"], "fact");
    }

    #[test]
    fn extract_jsonish_recovers_when_braces_are_unbalanced_in_strings() {
        // `extract_json_object` bails on a stray unescaped quote; the fallback
        // path is what keeps such output usable.
        let text = r#"prefix {"text":"a " b","kind":"fact"} suffix"#;
        assert!(extract_jsonish_object(text).is_some());
    }

    #[tokio::test]
    async fn cli_judge_rejects_an_unknown_stance() {
        let canned = r#"{"stance":"maybe","quote":"q"}"#;
        let cmd = format!(
            r#"sh -c 'cat >/dev/null; printf %s "{}"'"#,
            canned.replace('"', r#"\""#)
        );
        let provider = CliProvider::new(&cmd, "en".into()).unwrap();
        let error = provider.judge("claim", "src").await.unwrap_err();
        assert_eq!(error.code(), crate::error::ErrorCode::CliBadOutput);
    }
```

- [ ] **Step 6: Close the two gaps in the prompt-selection tests**

`src-tauri/src/llm/prompts/mod.rs` already tests locale fallback, the CLI "JSON only" instruction, and non-emptiness. Two things are untested: that the CLI and Anthropic variants are actually different strings (a copy-paste of the wrong `include_str!` would pass every current test), and that `judge_prompt` splits by provider the same way `atomize_prompt` does. Add to the existing `tests` module:

```rust
    #[test]
    fn cli_and_anthropic_prompts_differ_per_stage() {
        for locale in ["cs", "en"] {
            assert_ne!(
                atomize_prompt(locale, ProviderKind::Cli),
                atomize_prompt(locale, ProviderKind::Anthropic),
                "{locale} atomize prompts must differ per provider"
            );
            assert_ne!(
                judge_prompt(locale, ProviderKind::Cli),
                judge_prompt(locale, ProviderKind::Anthropic),
                "{locale} judge prompts must differ per provider"
            );
        }
    }

    #[test]
    fn judge_prompt_unknown_locale_falls_back_to_en() {
        for provider in [ProviderKind::Anthropic, ProviderKind::Cli] {
            assert_eq!(judge_prompt("de", provider), judge_prompt("en", provider));
        }
    }

    #[test]
    fn cs_and_en_variants_differ_per_provider() {
        for provider in [ProviderKind::Anthropic, ProviderKind::Cli] {
            assert_ne!(
                atomize_prompt("cs", provider),
                atomize_prompt("en", provider)
            );
        }
    }
```

- [ ] **Step 7: Run everything**

Run: `pnpm test && (cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings)`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add docs/superpowers/plans/2026-05-20-druhy-nazor-00-overview.md src/lib/cliPresets.test.ts src/lib/i18n/parity.test.ts src-tauri/src/llm/cli.rs src-tauri/src/llm/prompts/mod.rs
git commit -m "test(cli): retrofit CLI provider into spec coverage and widen its tests"
```

---

## Task 9: Signing, notarization, and SmartScreen — handover materials only

**Explicitly out of execution scope.** Nothing here runs `codesign`, `notarytool`, or `signtool`, buys a certificate, or touches CI secrets. This task produces the document the owner needs to do it themselves.

**Files:**
- Create: `docs/distribution/CODE-SIGNING.md`
- Modify: `README.md` (one link)

- [ ] **Step 1: Write `docs/distribution/CODE-SIGNING.md`**

The document must contain, with no placeholders:

1. **Current state** — `src-tauri/tauri.conf.json` sets `bundle.macOS.signingIdentity: "-"` (ad-hoc). Ad-hoc signatures satisfy Gatekeeper's "identified developer" check for nobody: on macOS the user gets "cannot be opened because the developer cannot be verified"; on Windows every `.exe`/`.msi` download triggers SmartScreen's "Windows protected your PC" until reputation accumulates.
2. **What the owner must obtain** (a checklist, because none of it can be automated from here):
   - Apple Developer Program membership (99 USD/yr), a **Developer ID Application** certificate, the Team ID, and either an app-specific password or an App Store Connect API key (Issuer ID + Key ID + `.p8`) for `notarytool`.
   - Windows: an OV or EV code-signing certificate, or an **Azure Trusted Signing** subscription. Note the tradeoff plainly: OV certificates build SmartScreen reputation slowly (weeks of downloads); EV certificates and Azure Trusted Signing get immediate reputation but cost more and EV requires a hardware token, which makes unattended CI signing awkward.
3. **macOS steps** — exact commands the owner runs, in order: export the cert to a `.p12`, add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` as GitHub Actions secrets, change `signingIdentity` from `"-"` to the identity string, confirm `Entitlements.plist` requests only what the app needs, then verify locally:

   ```bash
   codesign --verify --deep --strict --verbose=2 "src-tauri/target/release/bundle/macos/PROVE.app"
   ```

   ```bash
   xcrun stapler validate "src-tauri/target/release/bundle/dmg/PROVE_0.4.4_universal.dmg"
   ```

   ```bash
   spctl --assess --type execute --verbose "src-tauri/target/release/bundle/macos/PROVE.app"
   ```

4. **Windows steps** — the `bundle.windows` keys Tauri reads (`certificateThumbprint`, `digestAlgorithm: "sha256"`, `timestampUrl`), where the thumbprint comes from, and the verification command:

   ```bash
   signtool verify /pa /v "src-tauri\target\release\bundle\nsis\PROVE_0.4.4_x64-setup.exe"
   ```

5. **What changes in CI** — which steps in `.github/workflows/release.yml` gain the secrets, and the rule that the secrets are referenced by name only and never echoed.
6. **Explicit non-goals** — no Mac App Store, no Microsoft Store, no auto-updater signing key rotation (that belongs to `05-distribution.md`).
7. **Decision the owner still has to make**, stated as a question, not an assumption: OV + slow reputation, EV + hardware token, or Azure Trusted Signing.

- [ ] **Step 2: Link it from the README**

Add to the README's build/release section:

```markdown
Release builds are currently ad-hoc signed. See [docs/distribution/CODE-SIGNING.md](docs/distribution/CODE-SIGNING.md) for what production signing and notarization would require.
```

- [ ] **Step 3: Commit**

```bash
git add docs/distribution/CODE-SIGNING.md README.md
git commit -m "docs(distribution): signing and notarization handover checklist"
```

---

## Task 10: Privacy documentation and changelog

**Files:**
- Create: `docs/PRIVACY.md`
- Modify: `README.md`, `CHANGELOG.md`
- Modify: `src/routes/settings/+page.svelte` (link out)

- [ ] **Step 1: Write `docs/PRIVACY.md`**

Bilingual is unnecessary here (contributors read English), but it must be exact. Required sections:

1. **What PROVE processes** — the question and answer text the user pastes; nothing else. No account, no sign-up, no identifiers.
2. **What leaves the machine, per configuration** — a table with three rows: CLI provider (nothing leaves by the app's doing; the CLI tool's own behaviour is the user's to know), Anthropic provider (full question + answer text to `api.anthropic.com`), Brave Search enabled (each verified claim's text as a search query to `api.search.brave.com`, plus a plain GET of each result URL for body extraction, sent with the `prove/<version>` user agent).
3. **The opt-in update check** — one GET to `api.github.com/repos/.../releases/latest` on launch when enabled; default off; no data sent beyond the request itself.
4. **What is stored locally, and where** — `cache.db` (SQLite: `verification_cache`, `analysis_history`) and `settings.json` in the Tauri app data dir, with the concrete paths: `~/Library/Application Support/app.prove.desktop/` on macOS and `%APPDATA%\app.prove.desktop\` on Windows.
5. **API keys** — macOS Keychain / Windows Credential Manager via `keyring`; never written to `settings.json`; never logged.
6. **Retention and deletion** — the history retention setting (default 90 days, pruned at launch), "Delete all history" in the History view, cache TTL (`cache_ttl_days`, default 7), and how to remove everything: quit the app and delete the app data directory.
7. **Telemetry** — none. No analytics, no crash reporting, no phone-home. State it flatly.
8. **Confirmation before send** — on by default; describes exactly what the modal shows.

- [ ] **Step 2: Link it from the README and Settings**

README: a "Privacy" section linking `docs/PRIVACY.md` and summarising the three-row table in two sentences.

Settings privacy section: a link that opens the GitHub-hosted copy via `openInBrowser`, with `t('settings.privacy_link')` — add the key to both bundles (`"Jak PROVE nakládá s daty"` / `"How PROVE handles your data"`).

- [ ] **Step 3: Add the changelog entry**

Insert above `## [0.4.4]` in `CHANGELOG.md`:

```markdown
## [Unreleased]

### Added

- **Pre-send confirmation.** Before any analysis starts, PROVE shows exactly what leaves your computer — which provider gets the text, whether web verification will run, and how many characters are involved. On by default; dismissible per analysis or permanently.
- **First-run onboarding.** A four-step introduction covering what the app does, what leaves your machine, provider setup, and the global hotkey.
- **History view.** Analyses have been stored locally since 0.3.0 but were unreachable. There is now a searchable list with per-entry open and delete, a "delete all" action, and a configurable retention window (default 90 days, pruned at launch).
- **Hotkey remapping.** The global shortcut is now recorded by pressing the combination instead of typed as text, is validated before saving, and takes effect immediately instead of at next launch.
- **High-contrast mode.** Opt-in in Settings, and automatic under the OS "increase contrast" setting and Windows High Contrast.

### Changed

- **Errors say what to do.** Backend failures now carry a stable code, and the UI renders a localized explanation with a retry and a jump to the relevant setting. The raw diagnostics stay available behind a disclosure.
- **Accessibility.** Claims are real buttons with screen-reader labels that name the classification and the verification verdict, results announce as they stream in, there is a skip link and landmark structure, and every text/background token now meets WCAG AA — enforced by a test.

### Fixed

- Saving a hotkey that another application already owns no longer silently persists a dead shortcut; the failure surfaces and the previous working shortcut is kept.
```

- [ ] **Step 4: Run everything one last time**

Run: `pnpm test && pnpm check && pnpm lint && (cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check)`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add docs/PRIVACY.md README.md CHANGELOG.md src/routes/settings/+page.svelte src/lib/i18n/cs.json src/lib/i18n/en.json
git commit -m "docs(privacy): privacy policy, README section, and 0.5.0 changelog"
```

---

## Spec Coverage Check

| Overview §7 requirement for `04-privacy-polish.md` | Covered by |
| --- | --- |
| Pre-send confirmation modal | Task 2 |
| Onboarding screens | Task 3 |
| Privacy disclosures | Task 3 (privacy step), Task 10 (`docs/PRIVACY.md`) |
| Error states | Task 1 |
| History UI | Task 4 (backend), Task 5 (UI) |
| Hotkey remapping | Task 6 |
| Accessibility (VoiceOver/NVDA) | Task 7 (Steps 8–13) |
| High-contrast mode | Task 7 (Steps 4–7) |
| CLI provider retrofitted into spec coverage and tests | Task 8 |
| macOS signing / notarization, Windows SmartScreen | Task 9 — **materials only, not executed** |

## Out of Scope

- Executing any signing, notarization, or certificate purchase (Task 9 is documentation).
- Auto-updater install flow — the app still only *detects* newer releases and opens the download page.
- Full-text search over claim text (history search matches the analysed input only).
- SQLCipher encryption at rest.
- Any telemetry or crash reporting, opt-in or otherwise.
