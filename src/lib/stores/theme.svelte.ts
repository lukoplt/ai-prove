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
