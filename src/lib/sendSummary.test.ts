import { describe, expect, it } from 'vitest';
import { describeSend } from './sendSummary';
import { DEFAULT_SETTINGS, type Settings } from './types';

const base: Settings = { ...DEFAULT_SETTINGS, onboarded: true };

describe('describeSend', () => {
  it('reports a local CLI run and no web search', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: false,
      question: '',
      answer: 'Karel IV. se narodil v roce 1316.',
    });

    expect(lines.map((line) => line.key)).toEqual([
      'send.dest_cli',
      'send.web_off',
      'send.payload',
    ]);
    expect(lines[0].vars.command).toBe('claude -p');
  });

  it('reports the Anthropic endpoint with the model', () => {
    const lines = describeSend({
      settings: { ...base, provider: 'anthropic' },
      bravePresent: false,
      question: '',
      answer: 'x',
    });

    expect(lines[0].key).toBe('send.dest_anthropic');
    expect(lines[0].vars.model).toBe('claude-haiku-4-5-20251001');
  });

  it('reports web verification with the configured limit', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: true,
      question: '',
      answer: 'x',
    });

    const web = lines.find((line) => line.key === 'send.web_on');
    expect(web?.vars.limit).toBe(8);
  });

  it('says "all" when the verification limit is null', () => {
    const lines = describeSend({
      settings: { ...base, verified_claims_limit: null },
      bravePresent: true,
      question: '',
      answer: 'x',
    });

    expect(lines.some((line) => line.key === 'send.web_on_all')).toBe(true);
  });

  it('counts question and answer characters separately', () => {
    const lines = describeSend({
      settings: base,
      bravePresent: false,
      question: 'abc',
      answer: 'abcde',
    });

    const payload = lines.find((line) => line.key === 'send.payload');
    expect(payload?.vars).toEqual({ questionChars: 3, answerChars: 5 });
  });
});
