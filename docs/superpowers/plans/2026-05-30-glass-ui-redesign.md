# Glass UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reskin PROVE to an Apple-style glass aesthetic with light/dark theming (auto + toggle), a results verdict banner, and improved states — using CSS design tokens, no new dependencies, all analysis logic unchanged.

**Architecture:** A new `tokens.css` defines the entire visual language as CSS custom properties (`:root` light, `[data-theme="dark"]` dark). A `theme` store resolves the user preference (`auto`/`light`/`dark`) against `prefers-color-scheme` and sets `data-theme` on `<html>`. Components are reskinned to token-driven glass surfaces. Two pure helpers (`resolveTheme`, `aggregateVerdict`) carry the only real logic and are unit-tested. A new `theme` settings field is threaded through TS + Rust with backward-compatible defaults.

**Tech Stack:** SvelteKit + Svelte 5 runes, Tauri (Rust), Vitest, cargo test. No Tailwind, no component library, no new npm deps.

---

## File Structure

**New files:**
- `src/lib/styles/tokens.css` — design tokens (light + dark), glass surface class, gradient-mesh background.
- `src/lib/theme.ts` — pure `resolveTheme(pref, prefersDark)` helper + `ThemePref` type.
- `src/lib/theme.test.ts` — unit tests for `resolveTheme`.
- `src/lib/stores/theme.svelte.ts` — runtime theme store (applies `data-theme`, watches matchMedia).
- `src/lib/verdict.ts` — pure `aggregateVerdict(claims)` helper + `Verdict` type.
- `src/lib/verdict.test.ts` — unit tests for `aggregateVerdict`.
- `src/lib/components/VerdictBanner.svelte` — renders the aggregated verdict.

**Modified files:**
- `src/app.css` — global reset becomes token-driven.
- `src/routes/+layout.svelte` — import tokens, init theme store.
- `src/routes/+page.svelte` — glass topbar, theme toggle, verdict banner, glass cards, states.
- `src/routes/settings/+page.svelte` — glass cards, section grouping, theme control.
- `src/lib/components/PasteInput.svelte`, `ClaimText.svelte`, `SidePanel.svelte`, `SourceCard.svelte`, `TierBadge.svelte`, `UpdateBanner.svelte` — token-driven glass reskin.
- `src/lib/types.ts` — `Settings.theme` + `ThemePref` re-export.
- `src/lib/stores/settings.svelte.ts` — `theme` default.
- `src/lib/i18n/cs.json`, `src/lib/i18n/en.json` — new keys.
- `src-tauri/src/storage/settings_store.rs` — `ThemePref` enum + `Settings.theme` field + tests.

---

## Task 1: Design tokens + global CSS

**Files:**
- Create: `src/lib/styles/tokens.css`
- Modify: `src/app.css`
- Modify: `src/routes/+layout.svelte:3` (add import)

- [ ] **Step 1: Create `src/lib/styles/tokens.css`**

```css
/* Design tokens — single source of truth. Light in :root, dark overrides under [data-theme="dark"]. */
:root {
  /* color */
  --bg: #eef0f4;
  --bg-elevated: #ffffff;
  --surface-glass: rgba(255, 255, 255, 0.62);
  --surface-glass-strong: rgba(255, 255, 255, 0.82);
  --surface-glass-border: rgba(17, 17, 26, 0.08);
  --text: #18181b;
  --text-muted: #52525b;
  --text-subtle: #8b8b94;
  --accent: #4f46e5;
  --accent-hover: #4338ca;
  --accent-soft: rgba(79, 70, 229, 0.12);
  --accent-contrast: #ffffff;
  --focus-ring: rgba(99, 102, 241, 0.55);

  /* status */
  --ok: #15803d;
  --ok-soft: rgba(34, 197, 94, 0.16);
  --bad: #b91c1c;
  --bad-soft: rgba(239, 68, 68, 0.15);
  --warn: #b45309;
  --warn-soft: rgba(234, 179, 8, 0.18);
  --neutral: #52525b;
  --neutral-soft: rgba(113, 113, 122, 0.14);

  /* tier */
  --tier-a-bg: #dbeafe; --tier-a-fg: #1e3a8a;
  --tier-b-bg: #e0e7ff; --tier-b-fg: #3730a3;
  --tier-c-bg: #eef0f4; --tier-c-fg: #374151;
  --tier-d-bg: #fee2e2; --tier-d-fg: #7f1d1d;

  /* glass */
  --glass-blur: 20px;
  --glass-sat: 180%;

  /* geometry */
  --space-1: 4px;  --space-2: 8px;  --space-3: 12px; --space-4: 16px;
  --space-5: 20px; --space-6: 24px; --space-7: 32px; --space-8: 40px;
  --radius-sm: 10px; --radius-md: 14px; --radius-lg: 18px; --radius-xl: 24px;
  --shadow-sm: 0 1px 2px rgba(17, 17, 26, 0.06), 0 2px 8px rgba(17, 17, 26, 0.05);
  --shadow-md: 0 4px 16px rgba(17, 17, 26, 0.10);
  --shadow-lg: 0 12px 40px rgba(17, 17, 26, 0.16);

  /* motion */
  --ease: cubic-bezier(0.22, 1, 0.36, 1);
  --dur-fast: 150ms;
  --dur: 200ms;

  /* mesh background blobs */
  --mesh-1: rgba(99, 102, 241, 0.18);
  --mesh-2: rgba(56, 189, 248, 0.16);
  --mesh-3: rgba(168, 85, 247, 0.14);
}

[data-theme='dark'] {
  --bg: #0b0c10;
  --bg-elevated: #16171d;
  --surface-glass: rgba(30, 31, 40, 0.55);
  --surface-glass-strong: rgba(30, 31, 40, 0.78);
  --surface-glass-border: rgba(255, 255, 255, 0.10);
  --text: #f3f4f6;
  --text-muted: #b4b7c0;
  --text-subtle: #8a8d97;
  --accent: #818cf8;
  --accent-hover: #a5b4fc;
  --accent-soft: rgba(129, 140, 248, 0.20);
  --accent-contrast: #0b0c10;
  --focus-ring: rgba(129, 140, 248, 0.6);

  --ok: #4ade80;  --ok-soft: rgba(34, 197, 94, 0.20);
  --bad: #f87171;  --bad-soft: rgba(239, 68, 68, 0.22);
  --warn: #fbbf24; --warn-soft: rgba(234, 179, 8, 0.22);
  --neutral: #a1a1aa; --neutral-soft: rgba(161, 161, 170, 0.16);

  --tier-a-bg: rgba(59, 130, 246, 0.22); --tier-a-fg: #bfdbfe;
  --tier-b-bg: rgba(99, 102, 241, 0.22); --tier-b-fg: #c7d2fe;
  --tier-c-bg: rgba(148, 163, 184, 0.18); --tier-c-fg: #d1d5db;
  --tier-d-bg: rgba(239, 68, 68, 0.22); --tier-d-fg: #fecaca;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
  --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.5);
  --shadow-lg: 0 12px 40px rgba(0, 0, 0, 0.6);

  --mesh-1: rgba(79, 70, 229, 0.26);
  --mesh-2: rgba(14, 116, 144, 0.24);
  --mesh-3: rgba(126, 34, 206, 0.22);
}

/* shared glass surface */
.glass {
  background: var(--surface-glass);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-sat));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-sat));
  border: 1px solid var(--surface-glass-border);
  box-shadow: var(--shadow-sm);
}

/* app mesh background — fixed, sits behind everything */
.app-mesh {
  position: fixed;
  inset: 0;
  z-index: -1;
  background:
    radial-gradient(60% 50% at 12% 8%, var(--mesh-1), transparent 60%),
    radial-gradient(55% 45% at 88% 12%, var(--mesh-2), transparent 60%),
    radial-gradient(50% 50% at 50% 100%, var(--mesh-3), transparent 60%),
    var(--bg);
}

@media (prefers-reduced-motion: reduce) {
  * { transition: none !important; animation: none !important; }
}
```

- [ ] **Step 2: Replace hardcoded values in `src/app.css` with tokens**

Replace the whole file with:

```css
@import '$lib/styles/tokens.css';

:root {
  color-scheme: light dark;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
}

* { box-sizing: border-box; }

html, body {
  min-width: 320px;
  height: 100vh;
  margin: 0;
  overflow: hidden;
  background: var(--bg);
  color: var(--text);
}

button, input, select, textarea { font: inherit; }

button {
  border: 1px solid var(--surface-glass-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  background: var(--surface-glass-strong);
  color: var(--text);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
}
button:hover { border-color: var(--accent); }
button:active { transform: translateY(1px); }
button:disabled { cursor: not-allowed; opacity: 0.5; }

input, select, textarea {
  border: 1px solid var(--surface-glass-border);
  border-radius: var(--radius-sm);
  background: var(--bg-elevated);
  color: var(--text);
  transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
}
input, select { min-height: 38px; padding: var(--space-2) var(--space-3); }
textarea { line-height: 1.5; }
input:focus, select:focus, textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
  outline: none;
}

:focus-visible { outline: 3px solid var(--focus-ring); outline-offset: 2px; }
```

- [ ] **Step 3: Wire mesh background into `src/routes/+layout.svelte`**

In the markup block (currently lines 19-23), add the mesh div before the content render:

```svelte
{#if settings.loaded}
  <div class="app-mesh" aria-hidden="true"></div>
  {@render children()}
{:else}
  <div class="boot">{bootLabel}</div>
{/if}
```

- [ ] **Step 4: Verify build compiles**

Run: `pnpm check`
Expected: `0 ERRORS`

- [ ] **Step 5: Commit**

```bash
git add src/lib/styles/tokens.css src/app.css src/routes/+layout.svelte
git commit -m "feat(ui): add glass design tokens and mesh background"
```

---

## Task 2: Settings `theme` field (Rust + TS)

**Files:**
- Modify: `src-tauri/src/storage/settings_store.rs`
- Modify: `src/lib/types.ts:51-78`
- Modify: `src/lib/stores/settings.svelte.ts:11-20`

- [ ] **Step 1: Add `theme` round-trip + legacy tests in `settings_store.rs`**

Add these tests inside the `mod tests` block:

```rust
    #[test]
    fn default_theme_is_auto() {
        assert_eq!(Settings::default().theme, ThemePref::Auto);
    }

    #[test]
    fn legacy_settings_without_theme_deserializes_to_auto() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.theme, ThemePref::Auto);
    }

    #[test]
    fn theme_roundtrips_json() {
        let settings = Settings { theme: ThemePref::Dark, ..Settings::default() };
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, ThemePref::Dark);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test storage::settings_store`
Expected: FAIL — `no field 'theme'` / `cannot find type 'ThemePref'`.

- [ ] **Step 3: Add `ThemePref` enum and `theme` field**

After the `ProviderKind` enum (line ~16), add:

```rust
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePref {
    #[default]
    Auto,
    Light,
    Dark,
}
```

In `struct Settings`, after `check_updates_on_launch`, add:

```rust

    /// UI theme preference. `Auto` follows the OS color scheme.
    #[serde(default)]
    pub theme: ThemePref,
```

In `impl Default for Settings`, add `theme: ThemePref::Auto,` after `check_updates_on_launch: false,`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test storage::settings_store`
Expected: PASS (all, including the new three).

- [ ] **Step 5: Add `ThemePref` + `theme` to `src/lib/types.ts`**

After the `ProviderKind` type (line 51) add:

```typescript
export type ThemePref = 'auto' | 'light' | 'dark';
```

In `interface Settings`, after `check_updates_on_launch: boolean;` add:

```typescript
  theme: ThemePref;
```

- [ ] **Step 6: Add default in `src/lib/stores/settings.svelte.ts`**

In the `defaults` object, after `check_updates_on_launch: false,` add:

```typescript
  theme: 'auto',
```

- [ ] **Step 7: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/storage/settings_store.rs src/lib/types.ts src/lib/stores/settings.svelte.ts
git commit -m "feat: add theme preference to settings (auto/light/dark)"
```

---

## Task 3: Theme resolution helper + store + layout wiring

**Files:**
- Create: `src/lib/theme.ts`
- Create: `src/lib/theme.test.ts`
- Create: `src/lib/stores/theme.svelte.ts`
- Modify: `src/routes/+layout.svelte`

- [ ] **Step 1: Write the failing test `src/lib/theme.test.ts`**

```typescript
import { describe, expect, it } from 'vitest';
import { resolveTheme } from './theme';

describe('resolveTheme', () => {
  it('returns explicit light regardless of OS', () => {
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('light', false)).toBe('light');
  });

  it('returns explicit dark regardless of OS', () => {
    expect(resolveTheme('dark', false)).toBe('dark');
    expect(resolveTheme('dark', true)).toBe('dark');
  });

  it('follows OS when auto', () => {
    expect(resolveTheme('auto', true)).toBe('dark');
    expect(resolveTheme('auto', false)).toBe('light');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test -- src/lib/theme.test.ts`
Expected: FAIL — cannot resolve `./theme`.

- [ ] **Step 3: Write `src/lib/theme.ts`**

```typescript
import type { ThemePref } from './types';

export type ResolvedTheme = 'light' | 'dark';

export function resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme {
  if (pref === 'light') return 'light';
  if (pref === 'dark') return 'dark';
  return prefersDark ? 'dark' : 'light';
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test -- src/lib/theme.test.ts`
Expected: PASS (3 tests).

- [ ] **Step 5: Write `src/lib/stores/theme.svelte.ts`**

```typescript
import { resolveTheme, type ResolvedTheme } from '$lib/theme';
import type { ThemePref } from '$lib/types';

let pref = $state<ThemePref>('auto');
let resolved = $state<ResolvedTheme>('light');
let media: MediaQueryList | null = null;

function prefersDark(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function apply(): void {
  resolved = resolveTheme(pref, prefersDark());
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', resolved);
  }
}

export const theme = {
  get pref() {
    return pref;
  },
  get resolved() {
    return resolved;
  },

  /** Call once on mount with the persisted preference. */
  init(initial: ThemePref): void {
    pref = initial;
    apply();
    if (typeof window !== 'undefined' && !media) {
      media = window.matchMedia('(prefers-color-scheme: dark)');
      media.addEventListener('change', () => {
        if (pref === 'auto') apply();
      });
    }
  },

  /** Update preference at runtime (e.g. from the toggle). Caller persists to settings. */
  set(next: ThemePref): void {
    pref = next;
    apply();
  },
};
```

- [ ] **Step 6: Init theme in `src/routes/+layout.svelte`**

In the `<script>`, add the import:

```typescript
import { theme } from '$lib/stores/theme.svelte';
```

In the `onMount` callback, after `setLocale(settings.current.locale);` add:

```typescript
    theme.init(settings.current.theme);
```

- [ ] **Step 7: Run full check + tests**

Run: `pnpm check && pnpm test`
Expected: `0 ERRORS`; all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/lib/theme.ts src/lib/theme.test.ts src/lib/stores/theme.svelte.ts src/routes/+layout.svelte
git commit -m "feat: theme store applies data-theme from preference + OS"
```

---

## Task 4: Verdict aggregation helper + test

**Files:**
- Create: `src/lib/verdict.ts`
- Create: `src/lib/verdict.test.ts`

- [ ] **Step 1: Write the failing test `src/lib/verdict.test.ts`**

```typescript
import { describe, expect, it } from 'vitest';
import { aggregateVerdict } from './verdict';
import type { Claim } from './types';

function claim(partial: Partial<Claim>): Claim {
  return {
    id: 'x', text: 't', start: 0, end: 1, kind: 'fact', reason: 'r',
    verification: null, ...partial,
  };
}

describe('aggregateVerdict', () => {
  it('is unverified when no fact claims are verified', () => {
    const v = aggregateVerdict([claim({ kind: 'opinion' }), claim({ verification: null })]);
    expect(v.kind).toBe('unverified');
    expect(v.total).toBe(0);
    expect(v.verified).toBe(0);
  });

  it('is disputed when any fact claim is contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'contradicted', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('disputed');
  });

  it('is mostly_verified when 60%+ supported and none contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'no_consensus', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('mostly_verified');
    expect(v.verified).toBe(2);
    expect(v.total).toBe(3);
  });

  it('is no_consensus when verified below 60% and none contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'no_consensus', sources: [], summary: '' } }),
      claim({ verification: { status: 'not_found', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('no_consensus');
    expect(v.verified).toBe(1);
    expect(v.total).toBe(3);
  });

  it('ignores non-fact claims and unverified fact claims in totals', () => {
    const v = aggregateVerdict([
      claim({ kind: 'opinion' }),
      claim({ kind: 'fact', verification: null }),
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
    ]);
    expect(v.total).toBe(1);
    expect(v.verified).toBe(1);
    expect(v.kind).toBe('mostly_verified');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test -- src/lib/verdict.test.ts`
Expected: FAIL — cannot resolve `./verdict`.

- [ ] **Step 3: Write `src/lib/verdict.ts`**

```typescript
import type { Claim } from './types';

export type VerdictKind = 'mostly_verified' | 'disputed' | 'no_consensus' | 'unverified';

export interface Verdict {
  kind: VerdictKind;
  verified: number;
  total: number;
}

const VERIFIED_THRESHOLD = 0.6;

export function aggregateVerdict(claims: Claim[]): Verdict {
  const verifiable = claims.filter((c) => c.kind === 'fact' && c.verification !== null);
  const total = verifiable.length;
  const verified = verifiable.filter((c) => c.verification?.status === 'supported').length;
  const contradicted = verifiable.some((c) => c.verification?.status === 'contradicted');

  let kind: VerdictKind;
  if (contradicted) kind = 'disputed';
  else if (total === 0) kind = 'unverified';
  else if (verified / total >= VERIFIED_THRESHOLD) kind = 'mostly_verified';
  else kind = 'no_consensus';

  return { kind, verified, total };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test -- src/lib/verdict.test.ts`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src/lib/verdict.ts src/lib/verdict.test.ts
git commit -m "feat: add verdict aggregation helper"
```

---

## Task 5: i18n keys

**Files:**
- Modify: `src/lib/i18n/en.json`
- Modify: `src/lib/i18n/cs.json`

- [ ] **Step 1: Add keys to `src/lib/i18n/en.json`**

Add a `verdict` block and a `theme` block (top level, alongside existing groups). Also add states under the existing `summary`/`sidepanel` groups:

```json
  "verdict": {
    "mostly_verified": "Mostly verified",
    "disputed": "Disputed",
    "no_consensus": "No consensus",
    "unverified": "Unverified",
    "count": "{verified} of {total} claims verified"
  },
  "theme": {
    "label": "Theme",
    "auto": "Auto",
    "light": "Light",
    "dark": "Dark"
  }
```

Under `summary`, add: `"loading_hint": "Checking claims against sources…"`.
Under `sidepanel`, the existing `empty` key is reused for the icon state — no new key needed.

- [ ] **Step 2: Add the same keys to `src/lib/i18n/cs.json`**

```json
  "verdict": {
    "mostly_verified": "Převážně ověřeno",
    "disputed": "Sporné",
    "no_consensus": "Bez konsenzu",
    "unverified": "Neověřeno",
    "count": "Ověřeno {verified} z {total} tvrzení"
  },
  "theme": {
    "label": "Motiv",
    "auto": "Auto",
    "light": "Světlý",
    "dark": "Tmavý"
  }
```

Under `summary`, add: `"loading_hint": "Ověřuji tvrzení proti zdrojům…"`.

- [ ] **Step 3: Verify JSON parses + check passes**

Run: `pnpm check`
Expected: `0 ERRORS` (i18n JSON imported by the store; malformed JSON would fail the build).

- [ ] **Step 4: Commit**

```bash
git add src/lib/i18n/en.json src/lib/i18n/cs.json
git commit -m "feat(i18n): add verdict, theme, and loading-state strings"
```

---

## Task 6: VerdictBanner component

**Files:**
- Create: `src/lib/components/VerdictBanner.svelte`

- [ ] **Step 1: Write `src/lib/components/VerdictBanner.svelte`**

```svelte
<script lang="ts">
  import { t, tf } from '$lib/stores/i18n.svelte';
  import type { Claim } from '$lib/types';
  import { aggregateVerdict } from '$lib/verdict';

  let { claims }: { claims: Claim[] } = $props();

  const verdict = $derived(aggregateVerdict(claims));

  const ICONS = {
    mostly_verified: '✓',
    disputed: '✕',
    no_consensus: '~',
    unverified: '?',
  } as const;
</script>

<div class="banner glass kind-{verdict.kind}">
  <span class="icon" aria-hidden="true">{ICONS[verdict.kind]}</span>
  <div class="text">
    <strong class="headline">{t(`verdict.${verdict.kind}`)}</strong>
    {#if verdict.total > 0}
      <span class="count">{tf('verdict.count', { verified: verdict.verified, total: verdict.total })}</span>
    {/if}
  </div>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 999px;
    font-size: 16px;
    font-weight: 800;
    flex: 0 0 auto;
  }

  .text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .headline { font-size: 15px; }
  .count { color: var(--text-muted); font-size: 13px; }

  .kind-mostly_verified { border-color: var(--ok); }
  .kind-mostly_verified .icon { background: var(--ok-soft); color: var(--ok); }
  .kind-disputed { border-color: var(--bad); }
  .kind-disputed .icon { background: var(--bad-soft); color: var(--bad); }
  .kind-no_consensus { border-color: var(--warn); }
  .kind-no_consensus .icon { background: var(--warn-soft); color: var(--warn); }
  .kind-unverified .icon { background: var(--neutral-soft); color: var(--neutral); }
</style>
```

- [ ] **Step 2: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/VerdictBanner.svelte
git commit -m "feat(ui): add verdict banner component"
```

---

## Task 7: Theme toggle component

**Files:**
- Create: `src/lib/components/ThemeToggle.svelte`

- [ ] **Step 1: Write `src/lib/components/ThemeToggle.svelte`**

```svelte
<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { theme } from '$lib/stores/theme.svelte';
  import type { ThemePref } from '$lib/types';

  const OPTIONS: { value: ThemePref; glyph: string }[] = [
    { value: 'auto', glyph: 'A' },
    { value: 'light', glyph: '☀' },
    { value: 'dark', glyph: '☾' },
  ];

  async function choose(value: ThemePref) {
    theme.set(value);
    await settings.save({ ...settings.current, theme: value });
  }
</script>

<div class="seg" role="group" aria-label={t('theme.label')}>
  {#each OPTIONS as opt (opt.value)}
    <button
      type="button"
      class="opt"
      class:active={theme.pref === opt.value}
      aria-pressed={theme.pref === opt.value}
      title={t(`theme.${opt.value}`)}
      onclick={() => choose(opt.value)}
    >
      <span aria-hidden="true">{opt.glyph}</span>
      <span class="sr">{t(`theme.${opt.value}`)}</span>
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    padding: 2px;
    border-radius: 999px;
    background: var(--surface-glass);
    border: 1px solid var(--surface-glass-border);
  }
  .opt {
    padding: 4px 9px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--text-muted);
    font-size: 13px;
    line-height: 1;
  }
  .opt:hover { border-color: transparent; color: var(--text); }
  .opt.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .sr {
    position: absolute;
    width: 1px; height: 1px;
    padding: 0; margin: -1px;
    overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
  }
</style>
```

- [ ] **Step 2: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ThemeToggle.svelte
git commit -m "feat(ui): add theme toggle segmented control"
```

---

## Task 8: Main page — topbar, verdict banner, glass cards, states

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add imports**

In the `<script>`, after the existing component imports add:

```typescript
import ThemeToggle from '$lib/components/ThemeToggle.svelte';
import VerdictBanner from '$lib/components/VerdictBanner.svelte';
```

- [ ] **Step 2: Replace the `<header>` markup (lines 63-73) with a glass topbar**

```svelte
  <header class="topbar glass">
    <div class="brand">
      <h1>{t('app.title')}</h1>
      <p>{t('app.tagline')}</p>
    </div>
    <nav>
      <ThemeToggle />
      <button type="button" onclick={() => goto(resolve('/settings'))}>
        {t('common.settings')}
      </button>
    </nav>
  </header>
```

- [ ] **Step 3: Replace the result `running`/`done` blocks (lines 79-107)**

```svelte
  <section class="result">
    {#if analysisStore.status === 'running'}
      <div class="loading glass">
        <span class="spinner" aria-hidden="true"></span>
        <div>
          <p class="status">{t('summary.analyzing')}</p>
          <p class="hint">{t('summary.loading_hint')}</p>
        </div>
      </div>
    {:else if analysisStore.status === 'error'}
      <p class="status error glass">{tf('summary.error_prefix', { msg: analysisStore.error ?? '?' })}</p>
    {:else if analysisStore.status === 'done' && analysisStore.current}
      <div class="grid">
        <div class="left glass">
          <VerdictBanner claims={analysisStore.current.claims} />
          <p class="meta">{tf('summary.claims_count', { count: analysisStore.current.claims.length })}</p>
          {#if analysisStore.current.truncated}
            <p class="warning">{t('summary.truncated_warning')}</p>
          {/if}
          <div class="claim-scroll">
            <ClaimText
              input={analysisStore.current.input}
              claims={analysisStore.current.claims}
              selectedId={analysisStore.selectedId}
              onSelect={(id) => analysisStore.select(id)}
            />
          </div>
        </div>
        <div class="side-scroll">
          <SidePanel claim={analysisStore.selectedClaim} />
        </div>
      </div>
    {/if}
  </section>
```

- [ ] **Step 4: Replace the `<style>` block with token-driven glass styles**

Replace the contents of the existing `<style>` (lines 112-229). Keep the layout structure; swap hardcoded values for tokens and add topbar/loading rules:

```svelte
<style>
  .page {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    height: 100vh;
    max-width: 980px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-3);
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    margin-bottom: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    flex: 0 0 auto;
  }

  .brand h1 { margin: 0; font-size: 22px; line-height: 1.1; letter-spacing: -0.01em; }
  .brand p { margin: 2px 0 0; color: var(--text-subtle); font-size: 13px; }
  nav { display: flex; align-items: center; gap: var(--space-2); }

  .result {
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
    margin-top: var(--space-3);
    overflow: hidden;
  }

  .loading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-md);
  }
  .spinner {
    width: 18px; height: 18px;
    border: 2px solid var(--accent-soft);
    border-top-color: var(--accent);
    border-radius: 999px;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .status { margin: 0; color: var(--text-muted); font-size: 14px; }
  .hint { margin: 2px 0 0; color: var(--text-subtle); font-size: 13px; }
  .status.error {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border-color: var(--bad);
    color: var(--bad);
  }

  .grid {
    flex: 1 1 auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: var(--space-4);
    min-height: 0;
    width: 100%;
  }

  .left {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: var(--space-4);
    border-radius: var(--radius-lg);
  }

  .claim-scroll { flex: 1 1 auto; min-height: 0; overflow-y: auto; padding-right: var(--space-1); }
  .side-scroll { display: flex; flex-direction: column; min-height: 0; overflow-y: auto; }

  .meta { margin: 0 0 var(--space-2); color: var(--text-subtle); font-size: 13px; flex: 0 0 auto; }

  .warning {
    margin: 0 0 var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--warn-soft);
    color: var(--warn);
    font-size: 13px;
    flex: 0 0 auto;
  }

  .disclaimer {
    flex: 0 0 auto;
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--surface-glass-border);
    color: var(--text-subtle);
    font-size: 11px;
    line-height: 1.4;
    text-align: center;
  }

  @media (max-width: 820px) {
    .grid { grid-template-columns: 1fr; }
  }
</style>
```

Note: keep the existing `<footer class="disclaimer">` markup (line 109) unchanged.

- [ ] **Step 5: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(ui): glass topbar, verdict banner, and loading state on main page"
```

---

## Task 9: PasteInput reskin

**Files:**
- Modify: `src/lib/components/PasteInput.svelte`

- [ ] **Step 1: Replace the `<style>` block (lines 101-153) with token-driven styles**

```svelte
<style>
  .wrap {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-1);
    border: 2px solid transparent;
    border-radius: var(--radius-lg);
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
  }

  label { display: grid; gap: var(--space-2); }

  span {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 700;
  }

  .wrap.dragging {
    border-color: var(--accent);
    box-shadow: 0 0 0 4px var(--accent-soft);
  }

  textarea {
    width: 100%;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    resize: vertical;
  }

  label:last-of-type textarea { min-height: 200px; }

  .bar { display: flex; flex-wrap: wrap; gap: var(--space-2); align-items: center; }
  .spacer { flex: 1; }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover { background: var(--accent-hover); border-color: var(--accent-hover); }
</style>
```

- [ ] **Step 2: Verify**

Run: `pnpm check && pnpm test -- src/lib/components/PasteInput.test.ts`
Expected: `0 ERRORS`; PasteInput tests pass (behavior unchanged).

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/PasteInput.svelte
git commit -m "feat(ui): glass reskin for paste input"
```

---

## Task 10: ClaimText reskin

**Files:**
- Modify: `src/lib/components/ClaimText.svelte`

- [ ] **Step 1: Replace the `<style>` block (lines 65-100) with token-driven styles**

```svelte
<style>
  .ct { margin: 0; line-height: 1.75; font-size: 15px; white-space: pre-wrap; color: var(--text); }

  .claim {
    border-radius: var(--radius-sm);
    padding: 1px 3px;
    cursor: pointer;
    outline: 2px solid transparent;
    transition: outline-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
  }

  .claim:hover { background: var(--accent-soft); }
  .claim.selected { outline-color: var(--accent); }

  .kind-fact { background: var(--ok-soft); }
  .kind-inference { background: var(--warn-soft); }
  .kind-opinion { background: var(--neutral-soft); }
  .kind-contradiction { background: var(--bad-soft); }
</style>
```

- [ ] **Step 2: Verify**

Run: `pnpm check && pnpm test -- src/lib/components/ClaimText.test.ts`
Expected: `0 ERRORS`; ClaimText tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ClaimText.svelte
git commit -m "feat(ui): glass reskin for claim text highlights"
```

---

## Task 11: SourceCard + TierBadge reskin

**Files:**
- Modify: `src/lib/components/SourceCard.svelte`
- Modify: `src/lib/components/TierBadge.svelte`

- [ ] **Step 1: Replace the `<style>` block in `TierBadge.svelte` (lines 10-39)**

```svelte
<style>
  .badge {
    display: inline-block;
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }
  .tier-a { background: var(--tier-a-bg); color: var(--tier-a-fg); }
  .tier-b { background: var(--tier-b-bg); color: var(--tier-b-fg); }
  .tier-c { background: var(--tier-c-bg); color: var(--tier-c-fg); }
  .tier-d { background: var(--tier-d-bg); color: var(--tier-d-fg); }
</style>
```

- [ ] **Step 2: Change `SourceCard.svelte` root class to use glass**

In the markup (line 22), change `class="card stance-{source.stance}"` to `class="card glass stance-{source.stance}"`.

- [ ] **Step 3: Replace the `<style>` block in `SourceCard.svelte` (lines 35-104)**

```svelte
<style>
  .card {
    margin-bottom: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-md);
  }

  .stance-supports { border-left: 3px solid var(--ok); }
  .stance-contradicts { border-left: 3px solid var(--bad); }
  .stance-mentions { border-left: 3px solid var(--neutral); }

  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .stance-pill {
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--neutral-soft);
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 600;
  }

  .host {
    min-width: 0;
    margin-left: auto;
    color: var(--text-subtle);
    font-size: 11px;
    overflow-wrap: anywhere;
  }

  h4 { margin: 0 0 var(--space-1); color: var(--text); font-size: 13px; line-height: 1.3; }

  .snippet { margin: 0 0 var(--space-2); color: var(--text-muted); font-size: 12px; line-height: 1.4; }

  button {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
  }
  button:hover { color: var(--accent-hover); border-color: transparent; }
</style>
```

- [ ] **Step 4: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/SourceCard.svelte src/lib/components/TierBadge.svelte
git commit -m "feat(ui): glass reskin for source cards and tier badges"
```

---

## Task 12: SidePanel reskin + empty state

**Files:**
- Modify: `src/lib/components/SidePanel.svelte`

- [ ] **Step 1: Change the root class and empty state markup**

Change `<aside class="sp">` (line 17) to `<aside class="sp glass">`. Replace the empty branch (line 19) with an icon + text:

```svelte
  {#if !claim}
    <div class="empty">
      <span class="empty-icon" aria-hidden="true">◎</span>
      <p>{t('sidepanel.empty')}</p>
    </div>
```

- [ ] **Step 2: Replace the `<style>` block (lines 52-153) with token-driven styles**

```svelte
<style>
  .sp {
    min-height: 320px;
    padding: var(--space-4);
    border-radius: var(--radius-lg);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6) var(--space-3);
    text-align: center;
  }
  .empty-icon { font-size: 28px; color: var(--text-subtle); }
  .empty p { margin: 0; color: var(--text-subtle); font-size: 14px; }

  header { margin-bottom: var(--space-2); }

  .badge {
    display: inline-block;
    padding: 3px var(--space-2);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }
  .kind-fact { background: var(--ok-soft); color: var(--ok); }
  .kind-inference { background: var(--warn-soft); color: var(--warn); }
  .kind-opinion { background: var(--neutral-soft); color: var(--neutral); }
  .kind-contradiction { background: var(--bad-soft); color: var(--bad); }

  .quote {
    margin: 0 0 var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--surface-glass-border);
    background: var(--accent-soft);
    border-radius: var(--radius-sm);
    font-size: 14px;
  }

  section h3 { margin: 0 0 var(--space-1); color: var(--text-subtle); font-size: 12px; text-transform: uppercase; }
  section p { margin: 0 0 var(--space-3); font-size: 14px; color: var(--text); }

  .muted { color: var(--text-subtle); font-style: italic; }

  .verdict {
    margin: 0 0 var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: 13px;
    line-height: 1.4;
  }
  .status-supported { background: var(--ok-soft); color: var(--ok); }
  .status-contradicted { background: var(--bad-soft); color: var(--bad); }
  .status-no_consensus { background: var(--warn-soft); color: var(--warn); }
  .status-not_found,
  .status-not_verified { background: var(--neutral-soft); color: var(--text-muted); }
</style>
```

- [ ] **Step 3: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/SidePanel.svelte
git commit -m "feat(ui): glass reskin for side panel with icon empty state"
```

---

## Task 13: UpdateBanner reskin

**Files:**
- Modify: `src/lib/components/UpdateBanner.svelte`

- [ ] **Step 1: Read the file first to see its current markup/styles**

Run: `cat src/lib/components/UpdateBanner.svelte`

- [ ] **Step 2: Add `glass` to the banner root element's class and replace hardcoded colors**

Add `glass` to the outermost element's `class`. In its `<style>`, replace any hardcoded hex/background/border/`color` values with the nearest tokens following this mapping:
- background panel → remove (the `glass` class provides it) or `var(--surface-glass)`
- border → `1px solid var(--surface-glass-border)`
- border-radius → `var(--radius-md)`
- primary text → `var(--text)`; muted text → `var(--text-muted)`
- accent link/button → `var(--accent)` with `:hover { color: var(--accent-hover); }`
- padding/margins → nearest `var(--space-N)`

Do not change any script logic, props, or markup structure beyond adding the `glass` class.

- [ ] **Step 3: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/UpdateBanner.svelte
git commit -m "feat(ui): glass reskin for update banner"
```

---

## Task 14: Settings page reskin + theme control

**Files:**
- Modify: `src/routes/settings/+page.svelte`

- [ ] **Step 1: Read the current file to map sections**

Run: `cat src/routes/settings/+page.svelte`

- [ ] **Step 2: Add ThemeToggle import + a Theme section**

In the `<script>`, add:

```typescript
import ThemeToggle from '$lib/components/ThemeToggle.svelte';
```

Add a new `<section>` (after the header, before the provider sections) using the existing section markup pattern:

```svelte
  <section>
    <h2>{t('theme.label')}</h2>
    <ThemeToggle />
  </section>
```

- [ ] **Step 3: Add `glass` to each `<section>` and the header**

For every `<section>` element and the `<header>`/`<footer>` in this file, add the `glass` class to its class list (e.g. `<section>` → `<section class="glass">`). Do not alter any form logic, bindings, or handlers.

- [ ] **Step 4: Replace hardcoded values in the `<style>` block with tokens**

Map every hardcoded value in the file's `<style>` to tokens using the same scheme as Task 13:
- panel backgrounds → provided by `.glass`; drop standalone `background: #ffffff`
- `border: 1px solid #e4e4e7` (and similar) → `1px solid var(--surface-glass-border)`
- `border-radius` 6–8px → `var(--radius-sm)` / `var(--radius-md)`; cards → `var(--radius-lg)`
- text `#18181b`/`#3f3f46` → `var(--text)`; `#52525b`/`#71717a` → `var(--text-muted)`; `#a1a1aa` → `var(--text-subtle)`
- `.primary` button background `#18181b` → `var(--accent)`, color → `var(--accent-contrast)`, add `:hover { background: var(--accent-hover); }`
- status/success/error colors → `var(--ok)`/`var(--bad)` and their `-soft` backgrounds
- spacing px → nearest `var(--space-N)`
- Add `section { padding: var(--space-4); border-radius: var(--radius-lg); margin-bottom: var(--space-3); }` if sections lacked a uniform card style.

- [ ] **Step 5: Verify**

Run: `pnpm check`
Expected: `0 ERRORS`.

- [ ] **Step 6: Commit**

```bash
git add src/routes/settings/+page.svelte
git commit -m "feat(ui): glass reskin for settings with theme control"
```

---

## Task 15: Full verification + version bump

**Files:**
- Modify: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `CHANGELOG.md`

- [ ] **Step 1: Run the full gate**

Run: `pnpm lint && pnpm check && pnpm test`
Expected: prettier+eslint clean; `0 ERRORS`; all Vitest suites pass.

- [ ] **Step 2: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: all pass (including new theme tests).

- [ ] **Step 3: Manual smoke test (Tauri dev)**

Run: `pnpm tauri:dev`
Verify by hand:
- App loads with glass panels over the mesh background.
- Theme toggle Auto/Light/Dark switches instantly; OS dark-mode change flips Auto live.
- Toggle choice persists across relaunch.
- Paste a Q+A, analyze → verdict banner shows correct headline + count; claims highlight; clicking a claim fills the side panel; sources render.
- Resize below 820px → grid stacks (side panel under claim text); topbar stays usable.
- Reload Settings → theme control reflects current choice.

- [ ] **Step 4: Bump version to 0.4.0**

Edit `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (`version = "0.4.0"`), and the `druhy-nazor` entry in `src-tauri/Cargo.lock` from `0.3.1` to `0.4.0`.

- [ ] **Step 5: Add CHANGELOG entry**

Add above the `## [0.3.1]` entry:

```markdown
## [0.4.0] — 2026-05-30

### Changed

- New Apple-style glass UI: translucent panels over a gradient-mesh background, driven by a
  central CSS design-token system (no new dependencies).

### Added

- Light/dark theming that follows the OS by default, with an Auto/Light/Dark toggle persisted
  in settings (new `theme` setting).
- Results verdict banner that summarizes claim verification (Mostly verified / Disputed / No
  consensus / Unverified) with a verified-count.
- Improved loading and empty states.
```

- [ ] **Step 6: Format + final check**

Run: `pnpm format && pnpm check`
Expected: files formatted; `0 ERRORS`.

- [ ] **Step 7: Commit + tag + push**

```bash
git add -A
git commit -m "release: glass UI redesign (0.4.0)"
git tag v0.4.0
git push origin main --tags
```

---

## Self-Review

**Spec coverage:**
- Glass design system / tokens → Task 1. ✓
- Light/dark + auto + toggle → Tasks 2 (field), 3 (resolution+store), 7 (toggle), 14 (settings control). ✓
- Verdict banner → Tasks 4 (helper), 6 (component), 8 (wired into main page). ✓
- Improved empty/loading states → Task 8 (loading), Task 12 (empty side panel). ✓
- Component reskins (all 6 + 2 pages) → Tasks 8–14. ✓
- `theme` settings field across TS + Rust, backward compatible → Task 2. ✓
- i18n keys → Task 5. ✓
- Responsive < 820px → Task 8 media query (unchanged breakpoint, restyled). ✓
- Testing (check/lint/test/cargo + manual) → Task 15. ✓
- No new dependencies → confirmed; only CSS + Svelte + existing tooling. ✓

**Type consistency:** `ThemePref` = `'auto'|'light'|'dark'` used identically in `types.ts`, `theme.ts`, `theme.svelte.ts`, `ThemeToggle.svelte`, Rust `ThemePref` (snake_case serde matches the lowercase string union). `aggregateVerdict` returns `{ kind, verified, total }` consumed unchanged by `VerdictBanner`. `theme.set`/`theme.init`/`theme.pref`/`theme.resolved` names match between store and consumers.

**Placeholder scan:** No TBD/TODO. Tasks 13 and 14 use an explicit token-mapping table against the already-known hardcoded values in those files (read in their first step) rather than re-pasting unverified line numbers — concrete and actionable, not a placeholder.
