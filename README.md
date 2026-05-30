# PROVE

**P**rompt · **R**esponse · **O**utput · **V**erification · **E**ngine

A desktop verification layer for AI responses. Paste any AI answer, watch each claim get atomized, classified, and verified against the open web — all locally, with the LLM running on **your** machine via the CLI tool of your choice.

> Status: **alpha** · macOS (Apple Silicon + Intel) + Windows (x64) · Czech and English UI

---

## Why

LLMs sound confident even when they're wrong. PROVE doesn't trust the model — it pulls the claims apart, looks them up, and shows you which ones survive contact with the open web.

- **No cloud account required.** Your prompts never leave your machine unless you opt in to a cloud LLM.
- **No subscription.** No paywall. Local-first.
- **No "trust me" black box.** Every verified claim shows the actual source URLs, with tier badges and verbatim quotes.

---

## Features

- **Claim atomization.** AI answer is split into discrete claims (one verifiable fact per claim).
- **Epistemic classification.** Each claim is labeled `fact`, `inference`, `opinion`, or `contradiction`.
- **Web verification.** Fact-claims trigger Brave Search queries, fetch the top sources, run Mozilla Readability to extract clean article bodies, and ask the LLM to judge each source's stance.
- **Source tiering.** Domains scored A/B/C/D (Wikipedia & gov → A; major outlets → B; long tail → C; social/spam → D). Tier-aware aggregation: a Wikipedia "supports" outweighs a Reddit "contradicts".
- **Streaming UI.** Color-coded claim highlights appear as soon as classification completes; verification fills in per-claim asynchronously.
- **Local cache.** SQLite-backed verification cache keyed by claim hash (TTL configurable, default 7 days).
- **Privacy by design.** All settings local. API keys in OS keychain. Optional opt-in update check (default off).
- **Czech-first, English ready.** Full bilingual UI, system-locale auto-detect on first launch.

---

## Install

### macOS

1. Download `PROVE_0.2.0_universal.dmg` from the [latest Release](https://github.com/lukoplt/ai-prove/releases/latest).
2. Open the DMG and drag **PROVE** into Applications.
3. First launch: macOS Gatekeeper blocks unsigned apps.
   - **macOS 26 and newer:** the "Open Anyway" action moved into **System Settings → Privacy & Security**. Try to open `PROVE.app` once (it gets blocked with a warning dialog), then go to System Settings → Privacy & Security, scroll to the **"PROVE.app was blocked…"** notice, and click **Open Anyway**. Confirm with Touch ID / password.
   - **macOS 14 / 15:** right-click (or Ctrl-click) `PROVE.app` in Finder → **Open** → confirm.
   - After the first allow, normal launch works.

> The current build is adhoc-signed (no Apple Developer ID). Production signing + notarization is on the roadmap.

### Windows

1. Download `PROVE_0.2.0_x64_en-US.msi` (or `PROVE_0.2.0_x64-setup.exe` for the NSIS variant) from the [latest Release](https://github.com/lukoplt/ai-prove/releases/latest).
2. **If Microsoft Defender / SmartScreen blocks or removes the download:**
   - In Edge or Chrome, the download bar shows a warning. Click the **`…`** (or down-arrow) next to the file → **Keep** → confirm **Keep anyway** when Windows asks.
   - If Defender SmartScreen already removed the file, open **Windows Security → Virus & threat protection → Protection history**, find the PROVE entry, click **Actions → Allow → Restore**. Then re-download or copy the restored file back to Downloads.
   - To preempt the block: **Windows Security → Virus & threat protection → Manage settings → Add or remove exclusions → File** and add the downloaded installer.
3. Double-click the installer and follow the wizard. SmartScreen may still warn about an unrecognized publisher — click **More info → Run anyway**.

> Production code-signing (EV cert) is on the roadmap. Until then, the warnings above are expected on every clean Windows install.

---

## Quick start

1. Open **Settings**.
2. Pick an **LLM provider**:
   - **Local CLI command (recommended)** — defaults to `claude -p`. Works with any tool that takes a prompt on stdin and prints JSON on stdout: `claude -p` (Claude Code), `codex --print` (OpenAI Codex CLI), `ollama run qwen2.5-coder` (Ollama with JSON mode), `aichat`, `llama-cli`, …
   - **Anthropic API (cloud)** — bring your own key. Stored in the OS keychain.
3. (Optional) Paste a **Brave Search API key** to enable web source verification. Without it, claims are still atomized and classified but not verified.
4. Pick your interface language (auto-detected from OS on first launch).
5. Save settings, paste any AI answer into the main window, click **Analyze**.

---

## Supported LLM providers

| Provider          | How                                                          | Authentication                                  | Cost                            |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------- | ------------------------------- |
| **Local CLI**     | `claude -p`, `codex --print`, `ollama run …`, `aichat`, etc. | Whatever the CLI uses (already on your machine) | Zero — uses your existing setup |
| **Anthropic API** | Direct HTTPS to `api.anthropic.com`                          | API key in OS keychain                          | Pay per token                   |

Anything that can read a prompt from stdin and print one JSON object to stdout works as a CLI provider. The shell command is parsed via `shlex::split`, so quoting follows POSIX rules.

---

## How it works

```
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ AI answer    │ ─→ │ Atomize +        │ ─→ │ Brave Search +   │ ─→ │ Tier-aware       │
│ (pasted)     │    │ classify (LLM)   │    │ Readability +    │    │ verdict +        │
│              │    │                  │    │ Judge (LLM)      │    │ clickable sources│
└──────────────┘    └──────────────────┘    └──────────────────┘    └──────────────────┘
```

1. **Atomize + classify.** One LLM call splits the answer into atomic claims and tags each with an epistemic type.
2. **Verify (fact-claims only).** Each `fact` claim spawns a Brave Search query. Top results are fetched and run through Mozilla Readability to extract the main article body.
3. **Judge.** A second LLM call per (claim, source) decides whether the source `supports`, `contradicts`, or merely `mentions` the claim, and returns a short verbatim quote.
4. **Aggregate.** Stances are weighted by tier. An A-tier `supports` outweighs a C-tier `contradicts`. The side panel shows the final verdict, the summary, and the top 3 sources.
5. **Cache.** The verdict is keyed by SHA-256 of the normalized claim text and stored in local SQLite for the configured TTL.

Caps: max 25 claims per analysis, max 8 fact-claims verified per analysis. Configurable in code, not yet in UI.

---

## Privacy

- **No telemetry.** PROVE never phones home.
- **No analytics.** No fingerprinting, no usage tracking.
- **API keys** are stored in the OS keychain (macOS Keychain, Windows Credential Manager). They never touch disk in plaintext.
- **History and cache** live in the OS-default app data directory, on your machine only.
- **Update check** is opt-in (default off). When enabled, makes exactly one anonymous GET to the GitHub Releases API on launch. No app data is sent.
- **LLM traffic** only goes where you point it. Local CLI → stays on the box. Anthropic API → goes to `api.anthropic.com` directly from your machine.
- **Brave Search** is the only required outbound call for verification, and only when you provide a key.

---

## Development

```bash
# Prereqs: Rust toolchain, Node 22+, pnpm 10+

git clone https://github.com/lukoplt/ai-prove.git
cd ai-prove
pnpm install
pnpm tauri dev
```

Verify:

```bash
pnpm check                                              # svelte-check
pnpm lint                                               # prettier + eslint
pnpm test                                               # vitest
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Build installers:

```bash
pnpm tauri build                                        # local platform only
```

Cross-platform release builds run in CI via `.github/workflows/release.yml`. Trigger:

```bash
gh workflow run release.yml --ref main                  # workflow_dispatch, no tag
# OR
git tag v0.2.0 && git push origin v0.2.0                # auto-triggers on tag push
```

---

## Architecture

| Layer              | Tech                                                                          |
| ------------------ | ----------------------------------------------------------------------------- |
| Shell              | Tauri 2                                                                       |
| Frontend           | Svelte 5 + TypeScript + Vite                                                  |
| Backend            | Rust (tokio, reqwest, rusqlite, keyring)                                      |
| LLM                | User-configured CLI subprocess OR Anthropic HTTPS API                         |
| Search             | Brave Search API (optional)                                                   |
| Article extraction | Mozilla Readability (Rust port)                                               |
| Storage            | SQLite (cache + history) + OS keychain (secrets) + tauri-plugin-store (prefs) |
| i18n               | Bilingual JSON bundles (cs, en), system-locale default                        |

Code is organized so that adding a new LLM provider means implementing a single `LlmProvider` trait in Rust; adding a new language means dropping a JSON bundle into `src/lib/i18n/`.

---

## Roadmap

- Apple Developer ID signing + notarization
- Windows code-signing certificate
- More languages (DE, PL, SK)
- Multiple search providers (DuckDuckGo, Tavily)
- Conversational mode (analyze multi-turn dialogs)
- Citation back-export (copy "you said X but source Y disagrees" to clipboard)
- History view with full-text search
- Browser extension companion (auto-capture from ChatGPT / Claude / Gemini)

---

## License

Apache-2.0. See [LICENSE](./LICENSE).
