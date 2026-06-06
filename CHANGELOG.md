# Changelog

All notable changes to **PROVE** (Prompt · Response · Output · Verification · Engine).
Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/).

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
