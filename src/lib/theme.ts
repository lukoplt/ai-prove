import type { ThemePref } from './types';

export type ResolvedTheme = 'light' | 'dark';

export function resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme {
  if (pref === 'light') return 'light';
  if (pref === 'dark') return 'dark';
  return prefersDark ? 'dark' : 'light';
}

export type ContrastAttr = 'more' | 'normal';

/**
 * Always resolved to an explicit attribute value: `normal` is what lets the
 * `prefers-contrast: more` media rule stand down for a user who deliberately
 * turned high contrast off in the app.
 */
export function resolveContrast(highContrast: boolean): ContrastAttr {
  return highContrast ? 'more' : 'normal';
}
