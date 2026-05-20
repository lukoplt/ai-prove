import { beforeEach, describe, expect, it } from 'vitest';
import { setLocale, t } from './i18n.svelte';

describe('i18n', () => {
  beforeEach(() => setLocale('cs'));

  it('returns the cs string for a known key', () => {
    expect(t('input.placeholder')).toBe('Vlož sem odpověď AI…');
  });

  it('returns the en string after switching locale', () => {
    setLocale('en');
    expect(t('input.placeholder')).toBe('Paste an AI response here…');
  });

  it('falls back to the key when missing', () => {
    expect(t('does.not.exist')).toBe('does.not.exist');
  });

  it('falls back when the lookup descends into a non-string', () => {
    expect(t('input')).toBe('input');
  });
});
