# PROVE — Glass UI Redesign

**Date:** 2026-05-30
**Status:** Approved design, pending implementation plan

## Goal

Modernize PROVE's UI to an Apple-style "glass" aesthetic with clear UX, easy use, and full
responsiveness. Keep all existing analysis logic unchanged. Add light/dark theming (auto +
manual toggle) and targeted UX improvements. No new npm dependencies — pure CSS custom
properties.

## Constraints & Decisions

- **Aesthetic:** Glass / Apple style — translucent panels, `backdrop-filter` blur + saturation,
  soft light borders, layered over a subtle gradient-mesh background.
- **Theme:** Light + dark, default follows the OS (`prefers-color-scheme`), with a manual
  Auto / Light / Dark toggle persisted in settings.
- **Scope:** Visual redesign **plus** UX improvements (verdict banner, better empty/loading
  states, theme switch). No screen-flow rework, no new screens, no onboarding/history.
- **Tech base:** CSS custom properties + design tokens. No Tailwind, no component library, no
  new dependencies. Stays SvelteKit + Svelte 5 runes + Tauri.
- **Functional logic:** Unchanged. Claim extraction, verification, aggregation, provider plumbing
  all stay as-is. Only presentation + a new `theme` settings field change.

## Architecture

### Design-token layer (new)

New file `src/lib/styles/tokens.css`, imported once from `+layout.svelte` (alongside `app.css`).
Single source of truth for the visual language.

- `:root` defines the **light** palette and all non-color tokens.
- `[data-theme="dark"]` (set on `<html>`) overrides color tokens for **dark**.
- Default OS preference handled by the theme store setting `data-theme` from
  `prefers-color-scheme` when the user picks "Auto".

Token groups:

- **Color:** `--bg`, `--bg-elevated`, `--surface-glass`, `--surface-glass-border`,
  `--text`, `--text-muted`, `--text-subtle`, `--accent` (indigo 600), `--accent-soft`,
  `--accent-contrast`.
- **Status:** `--ok`, `--bad`, `--warn`, `--neutral` plus `-soft` background variants (used by
  verdict banner, side-panel verdict chip, claim kinds, source stance).
- **Tier:** `--tier-a`, `--tier-b`, `--tier-c`, `--tier-d`.
- **Glass:** `--glass-blur: 20px`, `--glass-sat: 180%`.
- **Geometry / depth:** `--space-1..8`, `--radius-sm|md|lg|xl` (10–18px), `--shadow-sm|md|lg`.
- **Motion:** `--ease`, `--dur-fast` (150ms), `--dur` (200ms).

Glass surface pattern (applied via a shared class, e.g. `.glass`):

```css
background: var(--surface-glass);
backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-sat));
-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-sat));
border: 1px solid var(--surface-glass-border);
```

App background: a fixed gradient-mesh (2–3 low-saturation radial blobs) so glass panels have
something to refract. Darker, lower-luminance variant under `[data-theme="dark"]`.

`app.css` keeps the global reset/normalize but its hardcoded button/input/hex styles move to
token-driven rules. Component-local hardcoded hex values are replaced with tokens.

### Theme store (new)

`src/lib/stores/theme.svelte.ts`:

- Holds the effective theme preference: `'auto' | 'light' | 'dark'`.
- Reads/writes `Settings.theme` via the existing settings store.
- Computes the resolved theme (`light`/`dark`) and writes `data-theme` onto
  `document.documentElement`.
- When preference is `auto`, subscribes to `window.matchMedia('(prefers-color-scheme: dark)')`
  and updates `data-theme` live on OS change.
- Applied in `+layout.svelte` after settings load (before first paint where possible to avoid
  flash).

A small segmented control (Auto / ☀ / 🌙) in the topbar and in Settings drives it.

### Settings model change

Add a `theme` field across the stack (backward compatible):

- `src/lib/types.ts` — `Settings.theme: 'auto' | 'light' | 'dark'`, exported `ThemePref` type;
  default `'auto'`.
- `src/lib/stores/settings.svelte.ts` — add `theme: 'auto'` to `defaults`.
- `src-tauri/src/storage/settings_store.rs` — new `ThemePref` enum
  (`#[serde(rename_all = "snake_case")]`, `#[default] Auto`), `Settings.theme` field with
  `#[serde(default)]` so legacy settings.json without the field still deserialize. Update the
  `Default` impl, add `theme` to the round-trip / legacy tests, and accept any of the three
  values in `validate()` (enum makes it total — no extra validation needed).

## Components

All component reskins keep their props, events, and the data shapes in `types.ts`. Only
markup classes and CSS change, except where a small UX addition is noted.

| Component                      | Change                                                                                                                                                 |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `+layout.svelte`               | Import `tokens.css`; init theme store; set `data-theme`; keep boot screen (glass-styled).                                                              |
| `routes/+page.svelte`          | Glass sticky topbar (wordmark + tagline, theme toggle, Settings). Glass result cards. **New verdict banner** above claims. Responsive stack < 820px.   |
| `routes/settings/+page.svelte` | Glass cards, clearer section grouping (provider / keys / behavior / about), theme segmented control. Footer credit kept (coffee link already removed). |
| `PasteInput.svelte`            | Glass fields, accent focus ring, drag-over = accent glow (not dashed border), icon+text buttons.                                                       |
| `ClaimText.svelte`             | Softer inline highlight (kind-colored underline + soft hover bg), selected = accent ring. Better long-text readability.                                |
| `SidePanel.svelte`             | Glass panel; kind badge; verdict as colored status chip block; sources list. Empty state = icon + text.                                                |
| `SourceCard.svelte`            | Glass card; tier badge + colored stance pill; hostname; quoted snippet; "Open →". Stance left-border kept.                                             |
| `TierBadge.svelte`             | Token-driven tier colors, consistent with system.                                                                                                      |
| `UpdateBanner.svelte`          | Subtle glass banner, accent action.                                                                                                                    |

### New: Verdict banner

A summary block above the claim text on the result screen. Aggregates claim verification
statuses into one headline so the user sees the outcome without reading details:

- Green "Převážně ověřeno" / "Mostly verified" when most fact-claims are `supported`.
- Red "Sporné" / "Disputed" when any are `contradicted` (or contradictions dominate).
- Amber "Bez konsenzu" / "No consensus" when mixed/`no_consensus`.
- Neutral "Neověřeno" / "Unverified" when nothing verifiable / not found.

Shows a status icon and a count, e.g. "3 ze 4 tvrzení ověřeno". Aggregation is a pure
derived function over `analysisStore.current.claims` (only `kind === 'fact'` claims with a
`verification` count toward verifiable totals). Lives as a small helper + inline render in
`+page.svelte` (or a tiny `VerdictBanner.svelte` component).

### States

- **Loading:** glass skeleton / soft pulse placeholder instead of the bare "Analyzuji…" line.
- **Empty side panel:** icon + helper text instead of a single muted sentence.
- **Error:** keep behavior; restyle as a status-colored glass notice.

## Data flow

Unchanged. `analysisStore` and `settings` stores keep their current API. New `theme` store reads
the persisted preference from settings and only touches `document.documentElement`'s
`data-theme`. The verdict banner derives from existing analysis state — no new state, no backend
calls.

## Motion & accessibility

- Transitions 150–200ms on hover/focus/theme change; all gated behind
  `@media (prefers-reduced-motion: reduce)`.
- Glass surfaces use opaque-enough backgrounds that body/label text meets WCAG AA contrast in
  both themes. Verify the muted/subtle text tokens against their backgrounds.
- Keep `:focus-visible` rings (token-driven accent).
- Theme toggle is a labeled control reachable by keyboard.

## i18n

Existing keys preserved. New keys added to both `cs.json` and `en.json`:

- `verdict.mostly_verified`, `verdict.disputed`, `verdict.no_consensus`, `verdict.unverified`,
  `verdict.count` (formatted "{verified} of {total}").
- `theme.label`, `theme.auto`, `theme.light`, `theme.dark`.
- Any new empty/loading-state strings.

## Affected files

- New: `src/lib/styles/tokens.css`, `src/lib/stores/theme.svelte.ts`, optional
  `src/lib/components/VerdictBanner.svelte`.
- Changed: `src/app.css`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`,
  `src/routes/settings/+page.svelte`, all 6 existing components, `src/lib/types.ts`,
  `src/lib/stores/settings.svelte.ts`, `src/lib/i18n/cs.json`, `src/lib/i18n/en.json`,
  `src-tauri/src/storage/settings_store.rs`.
- No new npm dependencies.

## Testing

- `pnpm check` (svelte-check) — 0 errors.
- `pnpm lint` — prettier + eslint clean.
- `pnpm test` — existing Vitest suite stays green; add a unit test for the verdict-aggregation
  helper (input claims → headline + counts).
- Rust: `cargo test` in `src-tauri` — settings round-trip + legacy-deserialize tests cover the
  new `theme` field.
- Manual: launch app, verify glass renders in both themes, OS auto-switch works, toggle persists,
  responsive stack under 820px, analysis flow end-to-end.

## Out of scope

- New screens (history, dashboard, onboarding).
- Screen-flow / navigation changes.
- Backend/analysis logic changes.
- New dependencies or build-tooling changes.
