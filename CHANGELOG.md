# Changelog

All notable changes to **PROVE** (Prompt · Response · Output · Verification · Engine).
Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added

- **Pre-send confirmation.** Before any analysis starts, PROVE shows exactly what leaves your computer — which provider gets the text, whether web verification will run and for how many claims, and how many characters are involved. On by default; dismissible for one analysis or permanently in Settings → Privacy.
- **First-run onboarding.** A four-step introduction covering what the app does, what leaves your machine, provider setup (including optional API keys), and the global hotkey.
- **History view.** Analyses have been stored locally since 0.3.0 but nothing could read them — the backend wrote rows no UI ever queried. There is now a searchable list with per-entry open and delete, a "delete all" action, and a configurable retention window (default 90 days, pruned at launch).
- **Hotkey remapping.** The global shortcut is now recorded by pressing the combination instead of typed as free text, is validated before saving, and takes effect immediately instead of at next launch.
- **High-contrast mode.** Opt-in in Settings, and automatic under the OS "increase contrast" setting and Windows High Contrast — opaque surfaces, no blur, no mesh background.
- **Privacy documentation.** `docs/PRIVACY.md` states exactly what is sent where, what is stored on disk and at which path, and how to delete all of it. Linked from Settings and the README.

### Changed

- **Errors say what to do.** Backend failures now carry a stable machine-readable code (`cli_not_found`, `llm_auth`, `search_rate_limit`, …) and the UI renders a localized explanation with a retry and, where relevant, a jump to the setting that fixes it. Raw diagnostics stay available behind a disclosure instead of being the whole message. This replaces the blocking `alert()` on the pre-analysis check.
- **Accessibility.** Claims are real buttons carrying screen-reader labels that name the classification and the verification verdict, arrow keys move between them, results and verdicts announce as they stream in, and there is a skip link and landmark structure. Every text/background token pair now meets WCAG AA — enforced by `src/lib/contrast.test.ts`, which fails the build if a colour regresses.
- **Claim highlights no longer rely on colour alone.** Each classification also carries a distinct underline.
- Confirmation dialogs trap focus, close on Escape, and restore focus to where the user was.
- Destructive confirmations (deleting an analysis, clearing history) are styled as destructive rather than as the default action.

### Fixed

- Saving a hotkey that another application already owns no longer silently persists a dead shortcut: the re-registration runs before the write, so the failure surfaces and the previous working shortcut is kept.

### Internal

- The CLI provider — the default LLM path since it landed after M2 — was never recorded in the plan documents. It now appears in the overview's cross-cutting decisions and spec-coverage tables, with tests covering PATH discovery, absolute-path resolution, JSON repair, error classification, and per-provider prompt selection.
- Frontend settings defaults are defined once in `src/lib/types.ts` instead of being duplicated across the store, the browser-preview fallback, and every test fixture.
- New test suite: i18n key/placeholder parity between `cs.json` and `en.json`.

## [0.4.4] — 2026-06-09

### Security

- Hardened source fetching against SSRF. The web extractor now refuses to fetch loopback, private, link-local, carrier-grade-NAT, and unique-local addresses (plus `localhost`/`*.local` hosts), and re-checks the host and scheme on every redirect hop — closing a bypass where a public page could redirect to an internal address. Only http(s) is followed, capped at 10 redirects.
- CI now runs `cargo audit` on Rust dependencies on every push and pull request (the JavaScript side was already covered).

## [0.4.3] — 2026-06-06

### Added

- New **web-verification limit** setting: choose how many factual claims are checked against the internet per analysis (4 / 8 / 12 / 16 / 20), or **All** to verify every factual claim. Default stays 8. Lower values are faster and use fewer web searches; higher values are more thorough. The hint notes it needs a Brave Search API key.

### Changed

- The "only the first N claims are verified" message now reflects the configured limit instead of a fixed 8.

### Fixed

- The analysis path now validates persisted settings and falls back to defaults if the stored settings file is invalid (e.g. a hand-edited verification limit of 0), avoiding a stalled result.

## [0.4.2] — 2026-06-01

### Security

- Pinned every GitHub Actions dependency in `ci.yml` and `release.yml` to a full commit SHA instead of a mutable tag (`@v6`, `@stable`, `@v0`). Closes the supply-chain path where a compromised third-party action (the `tj-actions/changed-files` class of attack) could steal the release workflow's `GITHUB_TOKEN` and tamper with published binaries.
- `Extractor::fetch_and_extract` now rejects any non-`http(s)` URL scheme before issuing a request, as defense-in-depth against a stray `file://`, `ftp://`, or custom-scheme source URL reaching the HTTP client.

## [0.4.1] — 2026-05-31

### Changed

- New app icon: glass squircle with an indigo-to-violet gradient and a speech bubble + checkmark mark, evoking a verified second opinion. Regenerated all macOS/Windows icon assets from a single SVG source (`src-tauri/app-icon.svg`).

## [0.4.0] — 2026-05-30

### Changed

- New Apple-style glass UI: translucent panels over a gradient-mesh background, driven by a central CSS design-token system (no new dependencies).

### Added

- Light/dark theming that follows the OS by default, with an Auto/Light/Dark toggle persisted in settings (new `theme` setting).
- Results verdict banner that summarizes claim verification (Mostly verified / Disputed / No consensus / Unverified) with a verified-count.
- Improved loading and empty states.

## [0.3.1] — 2026-05-30

### Removed

- Dropped the "Buy me a coffee" donation link from the settings footer and README, along with its i18n strings and styling.

## [0.3.0] — 2026-05-25

First clean minor release. Folds in everything from the 0.2.x iteration cycle: the disclaimer footer, the locked viewport with internal-only scroll, the Brave Search fix for the Czech locale (`country=ALL`), and the explicit minimal macOS entitlements that stop the TCC prompts for Downloads / Desktop / Music. No behavioral changes versus 0.2.3 — this is the cut-over point where the project stops deleting prior releases.

## [0.2.3] — 2026-05-25

### Fixed

- macOS 26: app no longer triggers TCC permission prompts for **Downloads**, **Desktop**, or **Music**. The bundle now ships with an explicit `Entitlements.plist` declaring only `network.client`, `cs.allow-jit`, and `cs.allow-unsigned-executable-memory` — no file-access entitlements at all. The main window also sets `dragDropEnabled: false`, which stops Tauri from registering the WebView as a native OS-level file-drop target (HTML-level drag & drop into the answer textarea keeps working because it uses WebView events, not native OS file drops).

## [0.2.2] — 2026-05-25

### Fixed

- Brave Search verification on Czech locale failed with `422 Unprocessable Entity` because the Brave API enum does not include `CZ` as a country code. The client now sends `country=ALL` for `cs` (and `country=US` for `en`) while keeping `search_lang=cs`/`en`, so Czech-language pages are still preferred but the request is accepted.

## [0.2.1] — 2026-05-25

### Fixed

- macOS / Windows: default window height bumped from 720 → 860 and `minHeight` from 480 → 640 so the disclaimer footer always fits at the bottom of the window.
- Tighter vertical paddings on the main page (header, result area) so the answer textarea and side panel stay readable at small heights.
- Disclaimer footer font tightened to 11px with reduced top spacing to keep the visible content area generous.
- PasteInput answer textarea `min-height` lowered from 260 → 200 to leave more room for the result grid on minimum-height windows.

## [0.2.0] — 2026-05-25

### Added

- Disclaimer footer on the main window: _"Even PROVE cannot guarantee 100% accuracy. The goal is to help reduce the spread of misinformation — the final judgment is always yours."_ (CS + EN).
- Settings page now scrolls internally inside the locked viewport.

### Changed

- Viewport locked: `html, body { height: 100vh; overflow: hidden }`. No more document-level scrollbar; scrolling lives inside the claim text panel and the side panel only.
- Main page restructured as a flex column with three regions: header (auto), result grid (scrollable, fills remaining space), disclaimer footer (auto).

## [0.1.x] — pre-public history

The `v0.1.0` and `v0.1.1` tags from the rebrand cycle (formerly _Druhý názor_) were removed when PROVE was promoted to a clean 0.2 release. Nothing about their content is preserved here; the project history is intact on `main`.

[0.4.2]: https://github.com/lukoplt/ai-prove/releases/tag/v0.4.2
[0.4.1]: https://github.com/lukoplt/ai-prove/releases/tag/v0.4.1
[0.4.0]: https://github.com/lukoplt/ai-prove/releases/tag/v0.4.0
[0.3.1]: https://github.com/lukoplt/ai-prove/releases/tag/v0.3.1
[0.3.0]: https://github.com/lukoplt/ai-prove/releases/tag/v0.3.0
[0.2.3]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.3
[0.2.2]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.2
[0.2.1]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.1
[0.2.0]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.0
