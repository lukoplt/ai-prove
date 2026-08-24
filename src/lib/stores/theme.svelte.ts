import { resolveContrast, resolveTheme, type ResolvedTheme } from '$lib/theme';
import type { ThemePref } from '$lib/types';

let pref = $state<ThemePref>('auto');
let resolved = $state<ResolvedTheme>('light');
let highContrast = $state(false);
let media: MediaQueryList | null = null;

function prefersDark(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function apply(): void {
  resolved = resolveTheme(pref, prefersDark());
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', resolved);
    document.documentElement.setAttribute('data-contrast', resolveContrast(highContrast));
  }
}

export const theme = {
  get pref() {
    return pref;
  },
  get resolved() {
    return resolved;
  },
  get highContrast() {
    return highContrast;
  },

  /** Call once on mount with the persisted preferences. */
  init(initial: ThemePref, initialContrast = false): void {
    pref = initial;
    highContrast = initialContrast;
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

  /** Update the high-contrast preference. Caller persists to settings. */
  setContrast(next: boolean): void {
    highContrast = next;
    apply();
  },
};
