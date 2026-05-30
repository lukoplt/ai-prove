import type { ThemePref } from './types';

export type ResolvedTheme = 'light' | 'dark';

export function resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme {
  if (pref === 'light') return 'light';
  if (pref === 'dark') return 'dark';
  return prefersDark ? 'dark' : 'light';
}
