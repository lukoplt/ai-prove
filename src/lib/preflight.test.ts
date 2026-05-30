import { describe, expect, it } from 'vitest';
import { analysisPreflightError } from './preflight';
import type { Settings } from './types';

const cliSettings: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  cache_ttl_days: 7,
  onboarded: false,
  provider: 'cli',
  anthropic_model: 'claude-haiku-4-5-20251001',
  cli_command: 'claude -p',
  check_updates_on_launch: false,
  theme: 'auto',
};

const messages = {
  missingAnthropicKey: 'missing anthropic key',
  missingAnthropicModel: 'missing anthropic model',
  missingCliCommand: 'missing cli command',
};

describe('analysisPreflightError', () => {
  it('allows native CLI analysis without a Brave Search API key', () => {
    expect(
      analysisPreflightError({
        isNative: true,
        anthropicPresent: false,
        settings: cliSettings,
        messages,
      }),
    ).toBeNull();
  });
});
