# Druhý názor — Phase M0: Foundation

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Read `2026-05-20-druhy-nazor-00-overview.md` first for architecture and conventions.

**Goal:** A runnable, cross-platform Tauri 2.x desktop app shell with paste input, secure settings storage (Anthropic + Brave keys in OS keychain), global hotkey, system tray, Czech i18n scaffolding, and CI. No LLM calls yet — the analyze button just echoes the input.

**Architecture:** Tauri 2 project with `src-tauri/` (Rust) and `src/` (Svelte 5 + TS). pnpm as the JS package manager. Secrets in OS keychain via the `keyring` crate; non-secret prefs in `tauri-plugin-store`. i18n via a minimal `t()` helper reading `cs.json`.

**Tech Stack:** Tauri 2.x, Rust 1.78+, Svelte 5, TypeScript, Vite, pnpm, `tauri-plugin-global-shortcut`, `tauri-plugin-store`, `keyring` (v3).

**Prerequisites:**

- Rust toolchain installed (`rustup`).
- Node.js 20+ and pnpm 9+ installed.
- Tauri prerequisites for the host OS (see https://v2.tauri.app/start/prerequisites/).
- On macOS: Xcode Command Line Tools.
- On Windows: WebView2 Runtime (default on Win11) and Visual Studio Build Tools with C++ workload.

---

## Task 1: Initialize the Tauri project

**Files:**

- Create: `package.json`, `pnpm-lock.yaml`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `src/`, `src-tauri/`, `.gitignore`, `LICENSE`, `README.md` (minimal).

- [ ] **Step 1: Scaffold the project**

From the working directory `/Users/lukasoplt/Documents/Druhý názor/`:

```bash
pnpm create tauri-app@latest . \
  --manager pnpm \
  --template svelte-ts \
  --identifier cz.druhynazor.app \
  --tauri-version 2
```

When prompted for "Project name", accept the default (the current directory). When prompted to overwrite the empty directory, confirm yes.

- [ ] **Step 2: Install dependencies**

```bash
pnpm install
```

Expected: `node_modules/` created, `pnpm-lock.yaml` produced.

- [ ] **Step 3: Verify the scaffold runs**

```bash
pnpm tauri dev
```

Expected: a window opens showing the default Tauri+Svelte template. Close it.

- [ ] **Step 4: Initialize git and write `.gitignore`**

```bash
git init
git branch -m main
```

Write `.gitignore`:

```gitignore
# Node
node_modules/
.pnp.*
.yarn/

# Vite
dist/
.svelte-kit/
.vite/

# Tauri / Rust
src-tauri/target/
src-tauri/gen/

# Env
.env
.env.local

# OS
.DS_Store
Thumbs.db

# Editor
.idea/
.vscode/*
!.vscode/settings.json
!.vscode/extensions.json
```

- [ ] **Step 5: Write the LICENSE file**

Write `LICENSE` with the full Apache-2.0 license text (copy from https://www.apache.org/licenses/LICENSE-2.0.txt).

- [ ] **Step 6: Write a minimal README**

Write `README.md`:

````markdown
# Druhý názor

Verification layer for AI responses. Desktop app for macOS and Windows.

## Development

```bash
pnpm install
pnpm tauri dev
```
````

See `docs/superpowers/plans/` for the implementation plan.

## License

Apache-2.0.

````

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2 + Svelte 5 + TS project"
````

---

## Task 2: Lock in Tauri configuration

**Files:**

- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `package.json` (scripts)

- [ ] **Step 1: Replace `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Druhý názor",
  "version": "0.1.0",
  "identifier": "cz.druhynazor.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../build"
  },
  "app": {
    "windows": [
      {
        "title": "Druhý názor",
        "width": 1100,
        "height": 720,
        "minWidth": 720,
        "minHeight": 480,
        "resizable": true,
        "fullscreen": false,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.anthropic.com https://api.search.brave.com"
    },
    "trayIcon": {
      "id": "main",
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg", "msi", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "Productivity",
    "shortDescription": "Verification layer for AI responses",
    "longDescription": "Druhý názor analyzes AI responses, classifies each claim by epistemic type, and verifies factual claims against the open web."
  }
}
```

- [ ] **Step 2: Replace `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-show",
    "core:window:allow-set-focus",
    "core:app:default",
    "global-shortcut:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "store:default",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save",
    "store:allow-load",
    "clipboard-manager:default",
    "clipboard-manager:allow-read-text",
    "clipboard-manager:allow-write-text"
  ]
}
```

- [ ] **Step 3: Update `package.json` scripts**

In `package.json`, replace the `"scripts"` block with:

```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "preview": "vite preview",
  "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
  "lint": "prettier --check . && eslint .",
  "format": "prettier --write .",
  "test": "vitest run",
  "test:watch": "vitest",
  "tauri": "tauri",
  "tauri:dev": "tauri dev",
  "tauri:build": "tauri build"
}
```

- [ ] **Step 4: Verify the config still builds**

```bash
pnpm tauri dev
```

Expected: window now titled "Druhý názor", 1100×720. Close it.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json package.json
git commit -m "chore: lock in tauri config (window, csp, tray, capabilities)"
```

---

## Task 3: Add Rust dependencies and module skeleton

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Update `src-tauri/Cargo.toml`**

Replace the file contents:

```toml
[package]
name = "druhy-nazor"
version = "0.1.0"
description = "Verification layer for AI responses"
authors = ["Druhý názor contributors"]
edition = "2021"
rust-version = "1.78"

[lib]
name = "druhy_nazor_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-store = "2"
tauri-plugin-clipboard-manager = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
keyring = "3"
uuid = { version = "1", features = ["v7"] }
sys-locale = "0.3"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "test-util"] }

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

- [ ] **Step 2: Create `src-tauri/src/error.rs`**

```rust
use thiserror::Error;

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

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

// Serialize AppError to a plain string so Tauri commands can return it to JS.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

- [ ] **Step 3: Create `src-tauri/src/models.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub kind: ClaimKind,
    pub reason: String,
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
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Supported,
    Contradicted,
    NoConsensus,
    NotFound,
    NotVerified,
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
pub enum SourceTier {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStance {
    Supports,
    Contradicts,
    Mentions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: String,
    pub created_at: i64,
    pub input: String,
    pub claims: Vec<Claim>,
    pub truncated: bool,
}
```

- [ ] **Step 4: Create `src-tauri/src/commands/mod.rs`**

```rust
// Tauri IPC command handlers. Each submodule registers via `register_handlers!`
// below.

pub mod analysis;
pub mod capture;
pub mod history;
pub mod settings;
```

Empty submodule files are created in subsequent tasks; for now create them as empty stubs:

```bash
mkdir -p src-tauri/src/commands
```

Write `src-tauri/src/commands/analysis.rs`:

```rust
// Filled in M1 (classification phase).
```

Write `src-tauri/src/commands/capture.rs`:

```rust
// Filled in Task 9 (global hotkey + clipboard read).
```

Write `src-tauri/src/commands/history.rs`:

```rust
// Filled in M2 (verification phase) and Polish phase.
```

Write `src-tauri/src/commands/settings.rs`:

```rust
// Filled in Task 7 (settings commands).
```

- [ ] **Step 5: Create `src-tauri/src/storage/mod.rs`**

```bash
mkdir -p src-tauri/src/storage
```

Write `src-tauri/src/storage/mod.rs`:

```rust
pub mod keychain;
pub mod settings_store;
// db.rs, cache.rs, history.rs added in later phases.
```

Touch the submodules to be filled by later tasks:

Write `src-tauri/src/storage/keychain.rs`:

```rust
// Filled in Task 5.
```

Write `src-tauri/src/storage/settings_store.rs`:

```rust
// Filled in Task 6.
```

- [ ] **Step 6: Create `src-tauri/src/lib.rs`**

```rust
pub mod commands;
pub mod error;
pub mod models;
pub mod storage;

pub use error::{AppError, AppResult};
```

- [ ] **Step 7: Replace `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 8: Verify compilation**

```bash
cd src-tauri && cargo build && cd ..
```

Expected: builds cleanly. Warnings about unused modules are fine — they will be filled in later tasks.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/
git commit -m "feat(rust): add dep tree, error type, shared models, module skeleton"
```

---

## Task 4: CI workflow

**Files:**

- Create: `.github/workflows/ci.yml`
- Create: `.editorconfig`
- Create: `.prettierrc.json`
- Create: `.prettierignore`
- Create: `eslint.config.js`

- [ ] **Step 1: Create `.editorconfig`**

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true

[*.rs]
indent_size = 4

[*.md]
trim_trailing_whitespace = false
```

- [ ] **Step 2: Create `.prettierrc.json`**

```json
{
  "useTabs": false,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [{ "files": "*.svelte", "options": { "parser": "svelte" } }]
}
```

- [ ] **Step 3: Create `.prettierignore`**

```
node_modules/
build/
dist/
src-tauri/target/
src-tauri/gen/
pnpm-lock.yaml
```

- [ ] **Step 4: Install lint deps**

```bash
pnpm add -D prettier prettier-plugin-svelte eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint-plugin-svelte
```

- [ ] **Step 5: Create `eslint.config.js`**

```js
import tsParser from '@typescript-eslint/parser';
import tsPlugin from '@typescript-eslint/eslint-plugin';
import sveltePlugin from 'eslint-plugin-svelte';

export default [
  {
    files: ['**/*.{ts,svelte}'],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaVersion: 2022, sourceType: 'module' },
    },
    plugins: { '@typescript-eslint': tsPlugin, svelte: sveltePlugin },
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
    },
  },
  {
    ignores: ['node_modules/', 'build/', 'src-tauri/target/', 'src-tauri/gen/'],
  },
];
```

- [ ] **Step 6: Create `.github/workflows/ci.yml`**

```yaml
name: ci

on:
  pull_request:
  push:
    branches: [main]

jobs:
  frontend:
    name: frontend (lint + typecheck + test)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v3
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm check
      - run: pnpm lint
      - run: pnpm test

  rust:
    name: rust (fmt + clippy + test)
    strategy:
      fail-fast: false
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: src-tauri }
      - name: Install macOS deps
        if: matrix.os == 'macos-latest'
        run: brew install pkg-config
      - run: cargo fmt --manifest-path src-tauri/Cargo.toml --check
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
      - run: cargo test --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 7: Verify lint locally**

```bash
pnpm lint
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: no errors. Fix any reported issue before continuing.

- [ ] **Step 8: Commit**

```bash
git add .editorconfig .prettierrc.json .prettierignore eslint.config.js .github/ package.json pnpm-lock.yaml
git commit -m "ci: add lint, typecheck, test workflow (frontend + rust matrix)"
```

---

## Task 5: Keychain wrapper for API keys

**Files:**

- Modify: `src-tauri/src/storage/keychain.rs`
- Test: `src-tauri/src/storage/keychain.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write the failing test**

Replace `src-tauri/src/storage/keychain.rs` with the test block first:

```rust
use crate::error::AppResult;
use keyring::Entry;

const SERVICE: &str = "cz.druhynazor.app";

/// Stores an API key under the given account name in the OS keychain.
pub fn set_api_key(account: &str, secret: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, account)?;
    entry.set_password(secret)?;
    Ok(())
}

/// Retrieves an API key by account name. Returns `Ok(None)` if absent.
pub fn get_api_key(account: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, account)?;
    match entry.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Deletes an API key by account name. Idempotent.
pub fn clear_api_key(account: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, account)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_account() -> String {
        format!("test-{}", uuid::Uuid::now_v7())
    }

    #[test]
    fn set_then_get_roundtrip() {
        let account = unique_account();
        set_api_key(&account, "secret-value-123").unwrap();
        let got = get_api_key(&account).unwrap();
        assert_eq!(got.as_deref(), Some("secret-value-123"));
        clear_api_key(&account).unwrap();
    }

    #[test]
    fn get_missing_returns_none() {
        let account = unique_account();
        let got = get_api_key(&account).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn clear_is_idempotent() {
        let account = unique_account();
        clear_api_key(&account).unwrap();
        clear_api_key(&account).unwrap();
    }

    #[test]
    fn overwrite_replaces_value() {
        let account = unique_account();
        set_api_key(&account, "first").unwrap();
        set_api_key(&account, "second").unwrap();
        let got = get_api_key(&account).unwrap();
        assert_eq!(got.as_deref(), Some("second"));
        clear_api_key(&account).unwrap();
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml storage::keychain
```

Expected: 4 tests pass on macOS (the host that prompts for keychain access may show a dialog on first run; accept it). On Windows the credential manager is used silently.

Note: in headless CI environments the keychain may be unavailable. If CI fails here later, gate the test module behind `#[cfg(not(ci))]` or skip via env var — but in local dev all four pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/storage/keychain.rs
git commit -m "feat(storage): keychain wrapper with roundtrip + idempotent clear"
```

---

## Task 6: Non-secret settings store

**Files:**

- Modify: `src-tauri/src/storage/settings_store.rs`
- Test: `src-tauri/src/storage/settings_store.rs` (inline)

The non-secret settings (UI language, hotkey override, default model, cache TTL override) live in `tauri-plugin-store` under `settings.json`. This wrapper hides the plugin's stringly-typed API behind a typed `Settings` struct.

- [ ] **Step 1: Write `settings_store.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.json";
pub const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub locale: String,            // "cs" or "en"
    pub hotkey: String,            // accelerator string
    pub model: String,             // anthropic model id
    pub cache_ttl_days: u32,
    pub onboarded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            hotkey: "CommandOrControl+Shift+D".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
            cache_ttl_days: 7,
            onboarded: false,
        }
    }
}

impl Settings {
    /// Maps a raw OS locale string ("cs-CZ", "en_US", "de-DE", ...) to one of
    /// the locales the app actually supports. Unsupported locales fall back to
    /// English.
    pub fn map_locale(raw: &str) -> String {
        let two = raw.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
        match two.as_str() {
            "cs" => "cs".into(),
            "en" => "en".into(),
            _ => "en".into(),
        }
    }

    /// Builds Settings with the locale derived from the host OS. Falls back to
    /// `Self::default()` (English) when the OS locale cannot be determined.
    pub fn with_system_locale() -> Self {
        let detected = sys_locale::get_locale()
            .as_deref()
            .map(Self::map_locale)
            .unwrap_or_else(|| "en".into());
        Self { locale: detected, ..Self::default() }
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.locale != "cs" && self.locale != "en" {
            return Err(AppError::Invalid(format!("locale must be cs or en, got {}", self.locale)));
        }
        if self.cache_ttl_days == 0 || self.cache_ttl_days > 90 {
            return Err(AppError::Invalid(format!(
                "cache_ttl_days out of range (1..=90), got {}",
                self.cache_ttl_days
            )));
        }
        if self.hotkey.trim().is_empty() {
            return Err(AppError::Invalid("hotkey cannot be empty".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_validate() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn invalid_locale_rejected() {
        let s = Settings { locale: "de".into(), ..Settings::default() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn zero_ttl_rejected() {
        let s = Settings { cache_ttl_days: 0, ..Settings::default() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn ttl_over_max_rejected() {
        let s = Settings { cache_ttl_days: 91, ..Settings::default() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn empty_hotkey_rejected() {
        let s = Settings { hotkey: "  ".into(), ..Settings::default() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn settings_roundtrip_json() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn map_locale_cs_variants() {
        assert_eq!(Settings::map_locale("cs-CZ"), "cs");
        assert_eq!(Settings::map_locale("cs_CZ"), "cs");
        assert_eq!(Settings::map_locale("CS"), "cs");
    }

    #[test]
    fn map_locale_en_variants() {
        assert_eq!(Settings::map_locale("en-US"), "en");
        assert_eq!(Settings::map_locale("en_GB"), "en");
        assert_eq!(Settings::map_locale("EN"), "en");
    }

    #[test]
    fn map_locale_unsupported_falls_back_to_en() {
        assert_eq!(Settings::map_locale("de-DE"), "en");
        assert_eq!(Settings::map_locale("fr"), "en");
        assert_eq!(Settings::map_locale(""), "en");
    }

    #[test]
    fn with_system_locale_produces_supported_locale() {
        let s = Settings::with_system_locale();
        assert!(s.locale == "cs" || s.locale == "en");
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml storage::settings_store
```

Expected: 10 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/storage/settings_store.rs
git commit -m "feat(storage): typed settings struct with validation"
```

---

## Task 7: Settings Tauri commands

**Files:**

- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

The frontend talks to settings through three Tauri commands: `get_settings`, `set_settings`, and a pair `set_api_key` / `clear_api_key` / `has_api_key` (we never expose `get_api_key` to JS — the key never leaves Rust).

- [ ] **Step 1: Replace `src-tauri/src/commands/settings.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::storage::keychain;
use crate::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use serde_json::json;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub async fn get_settings<R: Runtime>(app: AppHandle<R>) -> AppResult<Settings> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|e| AppError::Store(e.to_string()))?;

    let value = store.get(SETTINGS_KEY);
    let settings: Settings = match value {
        Some(v) => serde_json::from_value(v).unwrap_or_else(|_| Settings::with_system_locale()),
        None => Settings::with_system_locale(),
    };
    Ok(settings)
}

#[tauri::command]
pub async fn set_settings<R: Runtime>(
    app: AppHandle<R>,
    settings: Settings,
) -> AppResult<()> {
    settings.validate()?;

    let store = app
        .store(SETTINGS_FILE)
        .map_err(|e| AppError::Store(e.to_string()))?;
    store.set(SETTINGS_KEY, json!(settings));
    store.save().map_err(|e| AppError::Store(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(account: String, secret: String) -> AppResult<()> {
    if secret.trim().is_empty() {
        return Err(AppError::Invalid("api key is empty".into()));
    }
    keychain::set_api_key(&account, secret.trim())
}

#[tauri::command]
pub async fn clear_api_key(account: String) -> AppResult<()> {
    keychain::clear_api_key(&account)
}

#[tauri::command]
pub async fn has_api_key(account: String) -> AppResult<bool> {
    Ok(keychain::get_api_key(&account)?.is_some())
}
```

- [ ] **Step 2: Register commands in `src-tauri/src/main.rs`**

Replace the body of `main` so the invoke handler lists every command we expose:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use druhy_nazor_lib::commands::settings::{
    clear_api_key, get_settings, has_api_key, set_api_key, set_settings,
};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            set_api_key,
            clear_api_key,
            has_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify the Rust build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: clean build with at most unused-warning noise from later-task placeholders.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/settings.rs src-tauri/src/main.rs
git commit -m "feat(commands): get/set settings + set/clear/has_api_key"
```

---

## Task 8: Frontend type bridge and Tauri invoke wrappers

**Files:**

- Create: `src/lib/types.ts`
- Create: `src/lib/api.ts`

These two files are the only place that knows about Tauri `invoke()` calls in the frontend. Every component imports typed wrappers, not `invoke` directly.

- [ ] **Step 1: Create `src/lib/types.ts`**

```ts
export type ClaimKind = 'fact' | 'inference' | 'opinion' | 'contradiction';

export type VerificationStatus =
  | 'supported'
  | 'contradicted'
  | 'no_consensus'
  | 'not_found'
  | 'not_verified';

export type SourceTier = 'a' | 'b' | 'c' | 'd';

export type SourceStance = 'supports' | 'contradicts' | 'mentions';

export interface SourceHit {
  url: string;
  title: string;
  snippet: string;
  tier: SourceTier;
  stance: SourceStance;
}

export interface Verification {
  status: VerificationStatus;
  sources: SourceHit[];
  summary: string;
}

export interface Claim {
  id: string;
  text: string;
  start: number;
  end: number;
  kind: ClaimKind;
  reason: string;
  verification: Verification | null;
}

export interface Analysis {
  id: string;
  created_at: number;
  input: string;
  claims: Claim[];
  truncated: boolean;
}

export interface Settings {
  locale: 'cs' | 'en';
  hotkey: string;
  model: string;
  cache_ttl_days: number;
  onboarded: boolean;
}

export const ACCOUNT_ANTHROPIC = 'anthropic';
export const ACCOUNT_BRAVE = 'brave';
export type ApiAccount = typeof ACCOUNT_ANTHROPIC | typeof ACCOUNT_BRAVE;
```

- [ ] **Step 2: Create `src/lib/api.ts`**

```ts
import { invoke } from '@tauri-apps/api/core';
import type { ApiAccount, Settings } from './types';

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function setSettings(settings: Settings): Promise<void> {
  await invoke('set_settings', { settings });
}

export async function setApiKey(account: ApiAccount, secret: string): Promise<void> {
  await invoke('set_api_key', { account, secret });
}

export async function clearApiKey(account: ApiAccount): Promise<void> {
  await invoke('clear_api_key', { account });
}

export async function hasApiKey(account: ApiAccount): Promise<boolean> {
  return invoke<boolean>('has_api_key', { account });
}
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts src/lib/api.ts
git commit -m "feat(web): typed Tauri invoke wrappers and shared types"
```

---

## Task 9: i18n scaffolding

**Files:**

- Create: `src/lib/i18n/cs.json`
- Create: `src/lib/i18n/en.json`
- Create: `src/lib/stores/i18n.svelte.ts`
- Test: `src/lib/stores/i18n.test.ts`

- [ ] **Step 1: Create `src/lib/i18n/cs.json`**

```json
{
  "app": {
    "title": "Druhý názor",
    "tagline": "Druhý názor na odpovědi AI"
  },
  "input": {
    "placeholder": "Vlož sem odpověď AI…",
    "analyze": "Analyzovat",
    "clear": "Vymazat",
    "paste_from_clipboard": "Vložit ze schránky"
  },
  "settings": {
    "title": "Nastavení",
    "anthropic_key_label": "Anthropic API klíč",
    "anthropic_key_placeholder": "sk-ant-…",
    "brave_key_label": "Brave Search API klíč",
    "brave_key_placeholder": "BSA…",
    "save_key": "Uložit klíč",
    "clear_key": "Smazat klíč",
    "key_present": "Klíč je uložen v systémové klíčence.",
    "key_missing": "Klíč zatím není uložen.",
    "hotkey_label": "Globální klávesová zkratka",
    "model_label": "Výchozí model",
    "cache_ttl_label": "Doba platnosti cache (dny)",
    "back": "Zpět"
  },
  "tray": {
    "show": "Otevřít Druhý názor",
    "quit": "Ukončit"
  },
  "errors": {
    "key_empty": "Klíč nemůže být prázdný.",
    "invalid_settings": "Neplatná nastavení.",
    "unknown": "Neznámá chyba."
  },
  "common": {
    "settings": "Nastavení",
    "history": "Historie",
    "ok": "OK",
    "cancel": "Zrušit"
  }
}
```

- [ ] **Step 2: Create `src/lib/i18n/en.json`**

```json
{
  "app": {
    "title": "Druhý názor",
    "tagline": "A second opinion on AI responses"
  },
  "input": {
    "placeholder": "Paste an AI response here…",
    "analyze": "Analyze",
    "clear": "Clear",
    "paste_from_clipboard": "Paste from clipboard"
  },
  "settings": {
    "title": "Settings",
    "anthropic_key_label": "Anthropic API key",
    "anthropic_key_placeholder": "sk-ant-…",
    "brave_key_label": "Brave Search API key",
    "brave_key_placeholder": "BSA…",
    "save_key": "Save key",
    "clear_key": "Clear key",
    "key_present": "Key is stored in the system keychain.",
    "key_missing": "Key is not stored yet.",
    "hotkey_label": "Global hotkey",
    "model_label": "Default model",
    "cache_ttl_label": "Cache TTL (days)",
    "back": "Back"
  },
  "tray": {
    "show": "Open Druhý názor",
    "quit": "Quit"
  },
  "errors": {
    "key_empty": "The key cannot be empty.",
    "invalid_settings": "Invalid settings.",
    "unknown": "Unknown error."
  },
  "common": {
    "settings": "Settings",
    "history": "History",
    "ok": "OK",
    "cancel": "Cancel"
  }
}
```

- [ ] **Step 3: Create `src/lib/stores/i18n.svelte.ts`**

```ts
import cs from '../i18n/cs.json';
import en from '../i18n/en.json';

export type Locale = 'cs' | 'en';

const bundles: Record<Locale, unknown> = { cs, en };

let currentLocale = $state<Locale>('cs');

export function setLocale(l: Locale): void {
  currentLocale = l;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function t(key: string): string {
  const parts = key.split('.');
  let node: unknown = bundles[currentLocale];
  for (const p of parts) {
    if (typeof node !== 'object' || node === null) return key;
    node = (node as Record<string, unknown>)[p];
    if (node === undefined) return key;
  }
  return typeof node === 'string' ? node : key;
}
```

- [ ] **Step 4: Install vitest**

```bash
pnpm add -D vitest @testing-library/svelte @testing-library/jest-dom jsdom
```

- [ ] **Step 5: Configure vitest in `vite.config.ts`**

Open `vite.config.ts` and add the `test` block. The full file should read:

```ts
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  test: {
    environment: 'jsdom',
    globals: true,
  },
});
```

- [ ] **Step 6: Write the failing test**

Create `src/lib/stores/i18n.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest';
import { setLocale, t } from './i18n.svelte';

describe('i18n', () => {
  beforeEach(() => setLocale('cs'));

  it('returns the cs string for a known key', () => {
    expect(t('input.placeholder')).toBe('Vlož sem odpověď AI…');
  });

  it('returns the en string after switching locale', () => {
    setLocale('en');
    expect(t('input.placeholder')).toBe('Paste an AI response here…');
  });

  it('falls back to the key when missing', () => {
    expect(t('does.not.exist')).toBe('does.not.exist');
  });

  it('falls back when the lookup descends into a non-string', () => {
    expect(t('input')).toBe('input');
  });
});
```

- [ ] **Step 7: Run the test**

```bash
pnpm test
```

Expected: 4 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/lib/i18n/ src/lib/stores/i18n.svelte.ts src/lib/stores/i18n.test.ts vite.config.ts package.json pnpm-lock.yaml
git commit -m "feat(web): i18n scaffolding with cs/en bundles and t() helper"
```

---

## Task 10: Settings store + Settings page

**Files:**

- Create: `src/lib/stores/settings.svelte.ts`
- Create: `src/routes/settings/+page.svelte`
- Modify: `src/routes/+layout.svelte`

- [ ] **Step 1: Create `src/lib/stores/settings.svelte.ts`**

```ts
import { getSettings, setSettings, hasApiKey } from '$lib/api';
import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type Settings } from '$lib/types';

const defaults: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  model: 'claude-haiku-4-5-20251001',
  cache_ttl_days: 7,
  onboarded: false,
};

let current = $state<Settings>(defaults);
let anthropicPresent = $state(false);
let bravePresent = $state(false);
let loaded = $state(false);

export const settings = {
  get current() {
    return current;
  },
  get anthropicPresent() {
    return anthropicPresent;
  },
  get bravePresent() {
    return bravePresent;
  },
  get loaded() {
    return loaded;
  },

  async load(): Promise<void> {
    current = await getSettings();
    anthropicPresent = await hasApiKey(ACCOUNT_ANTHROPIC);
    bravePresent = await hasApiKey(ACCOUNT_BRAVE);
    loaded = true;
  },

  async save(next: Settings): Promise<void> {
    await setSettings(next);
    current = next;
  },

  async refreshKeyState(): Promise<void> {
    anthropicPresent = await hasApiKey(ACCOUNT_ANTHROPIC);
    bravePresent = await hasApiKey(ACCOUNT_BRAVE);
  },
};
```

- [ ] **Step 2: Update `src/routes/+layout.svelte`**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { setLocale } from '$lib/stores/i18n.svelte';
  import '../app.css';

  let { children } = $props();

  onMount(async () => {
    await settings.load();
    setLocale(settings.current.locale);
  });
</script>

{#if settings.loaded}
  {@render children()}
{:else}
  <div class="boot">Spouštím…</div>
{/if}

<style>
  .boot {
    display: grid;
    place-items: center;
    height: 100vh;
    font-family: system-ui, sans-serif;
    color: #6b7280;
  }
</style>
```

- [ ] **Step 3: Create `src/routes/settings/+page.svelte`**

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { settings } from '$lib/stores/settings.svelte';
  import { t } from '$lib/stores/i18n.svelte';
  import { setApiKey, clearApiKey } from '$lib/api';
  import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type Settings } from '$lib/types';

  let local: Settings = $state({ ...settings.current });
  let anthropicInput = $state('');
  let braveInput = $state('');
  let saving = $state(false);
  let message = $state<string | null>(null);

  async function persistSettings() {
    saving = true;
    try {
      await settings.save(local);
      message = 'OK';
    } catch (e) {
      message = String(e);
    } finally {
      saving = false;
    }
  }

  async function saveAnthropic() {
    const v = anthropicInput.trim();
    if (!v) {
      message = t('errors.key_empty');
      return;
    }
    await setApiKey(ACCOUNT_ANTHROPIC, v);
    anthropicInput = '';
    await settings.refreshKeyState();
    message = t('settings.key_present');
  }

  async function removeAnthropic() {
    await clearApiKey(ACCOUNT_ANTHROPIC);
    await settings.refreshKeyState();
  }

  async function saveBrave() {
    const v = braveInput.trim();
    if (!v) {
      message = t('errors.key_empty');
      return;
    }
    await setApiKey(ACCOUNT_BRAVE, v);
    braveInput = '';
    await settings.refreshKeyState();
    message = t('settings.key_present');
  }

  async function removeBrave() {
    await clearApiKey(ACCOUNT_BRAVE);
    await settings.refreshKeyState();
  }
</script>

<main class="page">
  <header>
    <button type="button" onclick={() => goto('/')}>{t('settings.back')}</button>
    <h1>{t('settings.title')}</h1>
  </header>

  <section>
    <h2>{t('settings.anthropic_key_label')}</h2>
    <p class="status">
      {settings.anthropicPresent ? t('settings.key_present') : t('settings.key_missing')}
    </p>
    <div class="row">
      <input
        type="password"
        bind:value={anthropicInput}
        placeholder={t('settings.anthropic_key_placeholder')}
        autocomplete="off"
      />
      <button type="button" onclick={saveAnthropic}>{t('settings.save_key')}</button>
      <button type="button" onclick={removeAnthropic} disabled={!settings.anthropicPresent}>
        {t('settings.clear_key')}
      </button>
    </div>
  </section>

  <section>
    <h2>{t('settings.brave_key_label')}</h2>
    <p class="status">
      {settings.bravePresent ? t('settings.key_present') : t('settings.key_missing')}
    </p>
    <div class="row">
      <input
        type="password"
        bind:value={braveInput}
        placeholder={t('settings.brave_key_placeholder')}
        autocomplete="off"
      />
      <button type="button" onclick={saveBrave}>{t('settings.save_key')}</button>
      <button type="button" onclick={removeBrave} disabled={!settings.bravePresent}>
        {t('settings.clear_key')}
      </button>
    </div>
  </section>

  <section>
    <h2>{t('settings.hotkey_label')}</h2>
    <input type="text" bind:value={local.hotkey} />
  </section>

  <section>
    <h2>{t('settings.model_label')}</h2>
    <input type="text" bind:value={local.model} />
  </section>

  <section>
    <h2>{t('settings.cache_ttl_label')}</h2>
    <input type="number" min="1" max="90" bind:value={local.cache_ttl_days} />
  </section>

  <footer>
    <button type="button" onclick={persistSettings} disabled={saving}>
      {t('settings.save_key')}
    </button>
    {#if message}
      <span class="msg">{message}</span>
    {/if}
  </footer>
</main>

<style>
  .page {
    max-width: 720px;
    margin: 0 auto;
    padding: 24px;
    display: grid;
    gap: 16px;
    font-family: system-ui, sans-serif;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  section {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 16px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .row input {
    flex: 1;
  }
  .status {
    font-size: 13px;
    color: #6b7280;
    margin: 0 0 8px;
  }
  footer {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .msg {
    color: #16a34a;
    font-size: 13px;
  }
</style>
```

- [ ] **Step 4: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. App boots, default route loads.
2. Navigate to `/settings` (temporarily by editing the URL in dev, or via a link added in Task 11).
3. Type a fake Anthropic key (`sk-ant-test`), click "Uložit klíč". Status flips to "Klíč je uložen…".
4. Restart the app. Status persists.
5. Click "Smazat klíč". Status flips to "Klíč zatím není uložen".

If any of these fail, fix before continuing.

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/settings.svelte.ts src/routes/+layout.svelte src/routes/settings/+page.svelte
git commit -m "feat(web): settings store + settings page with key entry"
```

---

## Task 11: Main page paste flow

**Files:**

- Modify: `src/routes/+page.svelte`
- Create: `src/lib/components/PasteInput.svelte`

The main page in M0 has one job: accept text via paste, drag&drop, or "Vložit ze schránky" button, and display the captured text. The Analyze button is wired but only logs the text — actual analysis is added in M1.

- [ ] **Step 1: Create `src/lib/components/PasteInput.svelte`**

```svelte
<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';

  let {
    value = $bindable(''),
    onAnalyze = () => {},
  }: { value?: string; onAnalyze?: (text: string) => void } = $props();

  let dragging = $state(false);

  async function paste() {
    const text = await readText();
    if (text) value = text;
  }

  function clear() {
    value = '';
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragging = true;
  }

  function onDragLeave() {
    dragging = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    const text = e.dataTransfer?.getData('text/plain') ?? '';
    if (text) value = text;
  }

  function analyze() {
    const trimmed = value.trim();
    if (trimmed) onAnalyze(trimmed);
  }
</script>

<div class="wrap" class:dragging ondragover={onDragOver} ondragleave={onDragLeave} ondrop={onDrop}>
  <textarea bind:value placeholder={t('input.placeholder')} rows={12} spellcheck="false"></textarea>
  <div class="bar">
    <button type="button" onclick={paste}>{t('input.paste_from_clipboard')}</button>
    <button type="button" onclick={clear} disabled={!value}>{t('input.clear')}</button>
    <div class="spacer"></div>
    <button type="button" class="primary" onclick={analyze} disabled={!value.trim()}>
      {t('input.analyze')}
    </button>
  </div>
</div>

<style>
  .wrap {
    display: grid;
    gap: 8px;
    border: 2px dashed transparent;
    border-radius: 8px;
    padding: 4px;
    transition: border-color 120ms ease;
  }
  .wrap.dragging {
    border-color: #3b82f6;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 12px;
    font-family: system-ui, sans-serif;
    font-size: 14px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    resize: vertical;
  }
  .bar {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .spacer {
    flex: 1;
  }
  .primary {
    background: #111827;
    color: white;
    border: none;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
  }
  .primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

- [ ] **Step 2: Replace `src/routes/+page.svelte`**

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import { t } from '$lib/stores/i18n.svelte';

  let inputText = $state('');

  function handleAnalyze(text: string) {
    // M1 will replace this with a real analysis call.
    console.log('[M0] would analyze:', text.slice(0, 80));
    inputText = text;
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
</main>

<style>
  .page {
    max-width: 900px;
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
</style>
```

- [ ] **Step 3: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. Main window shows the textarea and buttons.
2. Click "Nastavení" — navigates to settings page.
3. Back on the main page, click "Vložit ze schránky" — content of system clipboard appears.
4. Type some text, click "Analyzovat" — open the dev tools console; should see `[M0] would analyze: …`.
5. Drag a selection from any text editor onto the window — text appears in the textarea.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/PasteInput.svelte src/routes/+page.svelte
git commit -m "feat(web): paste flow with drag-drop and clipboard read"
```

---

## Task 12: Global hotkey

**Files:**

- Modify: `src-tauri/src/commands/capture.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs` (add `hotkey` module entry)
- Create: `src-tauri/src/hotkey.rs`

Behavior: pressing the configured hotkey (default `CommandOrControl+Shift+D`) brings the main window to focus and emits an event `capture-trigger` to the frontend. The frontend listens for the event and pastes the current clipboard contents into the input.

- [ ] **Step 1: Create `src-tauri/src/hotkey.rs`**

```rust
use crate::error::AppResult;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::{info, warn};

/// Installs the global hotkey using the accelerator string from settings.
/// Falls back to the default `CommandOrControl+Shift+D` if the configured one
/// fails to parse.
pub fn install<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let parsed: Shortcut = match accelerator.parse() {
        Ok(s) => s,
        Err(e) => {
            warn!("hotkey {accelerator} invalid ({e}); falling back to default");
            "CommandOrControl+Shift+D".parse().unwrap()
        }
    };

    let app_for_handler = app.clone();
    app.global_shortcut().on_shortcut(parsed, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            handle_trigger(&app_for_handler);
        }
    })?;

    info!("hotkey installed: {accelerator}");
    Ok(())
}

fn handle_trigger<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        let _ = win.unminimize();
        let _ = app.emit("capture-trigger", ());
    }
}
```

- [ ] **Step 2: Wire it into `src-tauri/src/main.rs`**

Replace the file:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use druhy_nazor_lib::commands::settings::{
    clear_api_key, get_settings, has_api_key, set_api_key, set_settings,
};
use druhy_nazor_lib::hotkey;
use druhy_nazor_lib::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            set_api_key,
            clear_api_key,
            has_api_key,
        ])
        .setup(|app| {
            let store = app.store(SETTINGS_FILE)?;
            let settings: Settings = store
                .get(SETTINGS_KEY)
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Re-export `hotkey` from `lib.rs`**

Replace `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod hotkey;
pub mod models;
pub mod storage;

pub use error::{AppError, AppResult};
```

- [ ] **Step 4: Frontend listener — update `src/routes/+page.svelte`**

Replace the `<script>` block:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { goto } from '$app/navigation';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import { t } from '$lib/stores/i18n.svelte';

  let inputText = $state('');

  function handleAnalyze(text: string) {
    console.log('[M0] would analyze:', text.slice(0, 80));
    inputText = text;
  }

  onMount(() => {
    const unlisten = listen('capture-trigger', async () => {
      const clipboard = await readText();
      if (clipboard) inputText = clipboard;
    });
    return () => {
      unlisten.then((u) => u());
    };
  });
</script>
```

- [ ] **Step 5: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. App launches.
2. Switch to another app (e.g. a browser or text editor) and copy a sentence.
3. Press `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows).
4. Druhý názor window comes to focus and the clipboard contents appear in the textarea.

If the hotkey is reported as taken by another app, change `hotkey` in Settings to e.g. `CommandOrControl+Shift+J`, restart the app, and retest.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/hotkey.rs src-tauri/src/lib.rs src-tauri/src/main.rs src/routes/+page.svelte
git commit -m "feat(hotkey): global shortcut focuses window and emits capture-trigger"
```

---

## Task 13: System tray

**Files:**

- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Add icon: `src-tauri/icons/tray.png` (32x32 monochrome PNG, transparent background)

- [ ] **Step 1: Create `src-tauri/src/tray.rs`**

```rust
use crate::error::AppResult;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub fn install<R: Runtime>(app: &AppHandle<R>) -> AppResult<TrayIcon<R>> {
    let show_item = MenuItem::with_id(app, "show", "Otevřít Druhý názor", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Ukončit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => focus_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if matches!(button, tauri::tray::MouseButton::Left) {
                    focus_main(tray.app_handle());
                }
            }
        })
        .build(app)?;
    Ok(tray)
}

fn focus_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
```

- [ ] **Step 2: Re-export `tray` from `lib.rs`**

Add `pub mod tray;` to `src-tauri/src/lib.rs`. Full file:

```rust
pub mod commands;
pub mod error;
pub mod hotkey;
pub mod models;
pub mod storage;
pub mod tray;

pub use error::{AppError, AppResult};
```

- [ ] **Step 3: Install the tray in `main.rs`**

In the `.setup` closure (after the hotkey install), add:

```rust
            druhy_nazor_lib::tray::install(&app.handle().clone())?;
```

The full `.setup` block becomes:

```rust
        .setup(|app| {
            let store = app.store(SETTINGS_FILE)?;
            let settings: Settings = store
                .get(SETTINGS_KEY)
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            druhy_nazor_lib::tray::install(&app.handle().clone())?;
            Ok(())
        })
```

- [ ] **Step 4: Add a placeholder tray icon**

The icon must exist at `src-tauri/icons/tray.png`. For M0 use a temporary placeholder: a 32×32 PNG of a single dot or filled circle on transparent background. Generate one with ImageMagick:

```bash
magick -size 32x32 xc:none -fill black -draw "circle 16,16 16,4" src-tauri/icons/tray.png
```

If ImageMagick is not installed, copy any 32×32 PNG into that path as a stub — the production icon arrives in the Polish phase.

- [ ] **Step 5: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. App boots; tray icon appears in the menu bar (macOS) or system tray (Windows).
2. Left-click the tray icon — main window focuses.
3. Right-click the tray icon — menu appears with "Otevřít Druhý názor" and "Ukončit".
4. Click "Ukončit" — app exits cleanly.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs src-tauri/src/main.rs src-tauri/icons/tray.png
git commit -m "feat(tray): system tray with show/quit menu and click-to-focus"
```

---

## Task 14: Single-instance enforcement and window behavior on close

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/capabilities/default.json`

Without single-instance enforcement, double-clicking the app icon could launch a second copy that fights for the global hotkey registration. We also want closing the window to hide it (so the tray icon stays) instead of quitting.

- [ ] **Step 1: Add the plugin to `Cargo.toml`**

Add to `[dependencies]`:

```toml
tauri-plugin-single-instance = "2"
```

- [ ] **Step 2: Wire it in `main.rs`**

In the `tauri::Builder::default()` chain, add the plugin _first_ (before others), and add a window-close handler. The full `main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use druhy_nazor_lib::commands::settings::{
    clear_api_key, get_settings, has_api_key, set_api_key, set_settings,
};
use druhy_nazor_lib::hotkey;
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
            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            druhy_nazor_lib::tray::install(&app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Update capabilities**

Append to the `permissions` array in `src-tauri/capabilities/default.json`:

```json
"core:window:allow-hide",
```

The final permissions list reads (order matches Task 2):

```json
"permissions": [
  "core:default",
  "core:window:default",
  "core:window:allow-close",
  "core:window:allow-show",
  "core:window:allow-set-focus",
  "core:window:allow-hide",
  "core:app:default",
  "global-shortcut:default",
  "global-shortcut:allow-register",
  "global-shortcut:allow-unregister",
  "global-shortcut:allow-is-registered",
  "store:default",
  "store:allow-get",
  "store:allow-set",
  "store:allow-save",
  "store:allow-load",
  "clipboard-manager:default",
  "clipboard-manager:allow-read-text",
  "clipboard-manager:allow-write-text"
]
```

- [ ] **Step 4: Manual smoke test**

```bash
pnpm tauri dev
```

Expected:

1. App opens, window visible.
2. Close the window with the OS close button — window hides, tray icon stays, app keeps running.
3. Click tray icon — window reappears.
4. Launch a second instance via terminal: `pnpm tauri dev` again (in a second shell). The existing window comes to focus; no second window opens.
5. Quit via tray "Ukončit" — process exits.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs src-tauri/capabilities/default.json
git commit -m "feat(app): single-instance + hide-on-close so tray stays alive"
```

---

## Task 15: M0 acceptance smoke + tag

**Goal:** Verify the M0 surface end-to-end and tag a `m0` snapshot.

- [ ] **Step 1: Run the M0 smoke checklist**

Run `pnpm tauri dev` and walk through:

1. App launches; window titled "Druhý názor".
2. Tray icon visible in menu bar / system tray.
3. Navigate to Settings; type an Anthropic key; save. Status reads "Klíč je uložen…".
4. Quit via tray and relaunch. Settings page still shows key present.
5. From the Settings page, change cache TTL to 14, save. Quit and relaunch. New value persists.
6. Back on the main page, paste some text via "Vložit ze schránky".
7. Drag&drop a selection from another app onto the window — text appears.
8. Copy a sentence in another app. Press the global hotkey. Window comes to focus; clipboard text auto-fills.
9. Close the window with the OS close button — window hides; app keeps running.
10. Click tray "Ukončit" — process exits.

If any item fails, fix before tagging.

- [ ] **Step 2: Run the full lint+test pass**

```bash
pnpm check
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all green.

- [ ] **Step 3: Tag the M0 snapshot**

```bash
git tag m0-foundation
```

- [ ] **Step 4: Move on to M1**

Open `2026-05-20-druhy-nazor-02-classification.md` to begin the classification phase.
