import { describe, expect, it } from 'vitest';
import { resolveContrast, resolveTheme } from './theme';

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

describe('resolveContrast', () => {
  it('maps the preference onto the data-contrast attribute', () => {
    expect(resolveContrast(true)).toBe('more');
    expect(resolveContrast(false)).toBe('normal');
  });
});
