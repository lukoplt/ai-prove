import { describe, expect, it } from 'vitest';
import { errorKey, isSettingsError, toAppError } from './errors';

describe('toAppError', () => {
  it('passes through a structured Tauri rejection', () => {
    expect(toAppError({ code: 'cli_not_found', message: "cli spawn 'claude'" })).toEqual({
      code: 'cli_not_found',
      message: "cli spawn 'claude'",
    });
  });

  it('falls back to `other` for an unknown code', () => {
    expect(toAppError({ code: 'wat', message: 'boom' })).toEqual({
      code: 'other',
      message: 'boom',
    });
  });

  it('wraps a thrown Error', () => {
    expect(toAppError(new Error('kaput'))).toEqual({ code: 'other', message: 'kaput' });
  });

  it('wraps a bare string', () => {
    expect(toAppError('kaput')).toEqual({ code: 'other', message: 'kaput' });
  });

  it('never returns an empty message', () => {
    expect(toAppError(null).message.length).toBeGreaterThan(0);
  });
});

describe('isSettingsError', () => {
  it('is true for codes a settings change fixes', () => {
    for (const code of ['cli_not_found', 'llm_auth', 'search_auth', 'invalid'] as const) {
      expect(isSettingsError(code)).toBe(true);
    }
  });

  it('is false for transient codes', () => {
    for (const code of ['network', 'llm_rate_limit', 'cli_timeout'] as const) {
      expect(isSettingsError(code)).toBe(false);
    }
  });
});

describe('errorKey', () => {
  it('namespaces codes under `error.`', () => {
    expect(errorKey('cli_timeout')).toBe('error.cli_timeout');
  });
});
