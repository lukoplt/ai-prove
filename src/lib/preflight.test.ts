import { describe, expect, it } from 'vitest';
import { analysisPreflightError } from './preflight';
import { DEFAULT_SETTINGS, type Settings } from './types';

const cliSettings: Settings = { ...DEFAULT_SETTINGS };

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
