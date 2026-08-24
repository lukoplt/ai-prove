import { describe, expect, it } from 'vitest';
import { canAdvance, nextStep, ONBOARDING_STEPS, prevStep } from './onboarding';
import { DEFAULT_SETTINGS, type Settings } from './types';

const draft: Settings = { ...DEFAULT_SETTINGS };

describe('onboarding steps', () => {
  it('runs welcome → privacy → provider → ready', () => {
    expect(ONBOARDING_STEPS).toEqual(['welcome', 'privacy', 'provider', 'ready']);
  });

  it('clamps at both ends', () => {
    expect(prevStep('welcome')).toBe('welcome');
    expect(nextStep('ready')).toBe('ready');
    expect(nextStep('welcome')).toBe('privacy');
    expect(prevStep('provider')).toBe('privacy');
  });
});

describe('canAdvance', () => {
  it('always allows leaving the informational steps', () => {
    expect(canAdvance('welcome', draft, false)).toBe(true);
    expect(canAdvance('privacy', draft, false)).toBe(true);
  });

  it('requires a non-empty CLI command', () => {
    expect(canAdvance('provider', draft, false)).toBe(true);
    expect(canAdvance('provider', { ...draft, cli_command: '  ' }, false)).toBe(false);
  });

  it('requires a model and a key for the Anthropic provider', () => {
    const anthropic: Settings = { ...draft, provider: 'anthropic' };
    expect(canAdvance('provider', anthropic, false)).toBe(false);
    expect(canAdvance('provider', anthropic, true)).toBe(true);
    expect(canAdvance('provider', { ...anthropic, anthropic_model: '' }, true)).toBe(false);
  });

  it('always allows finishing from the last step', () => {
    expect(canAdvance('ready', draft, false)).toBe(true);
  });
});
