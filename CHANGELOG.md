# Changelog

All notable changes to **PROVE** (Prompt · Response · Output · Verification · Engine).
Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/).

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

[0.2.1]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.1
[0.2.0]: https://github.com/lukoplt/ai-prove/releases/tag/v0.2.0
