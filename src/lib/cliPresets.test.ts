import { describe, expect, it } from 'vitest';
import { CLI_PRESETS, commandToCliPreset, presetCommand } from './cliPresets';

describe('CLI presets', () => {
  it('offers multiple CLI-backed providers', () => {
    expect(CLI_PRESETS.map((preset) => preset.id)).toEqual(['claude', 'codex', 'ollama', 'custom']);
  });

  it('matches known commands and preserves custom commands', () => {
    expect(commandToCliPreset('claude -p')).toBe('claude');
    expect(commandToCliPreset('codex exec -')).toBe('codex');
    expect(commandToCliPreset('my-llm --json')).toBe('custom');
    expect(presetCommand('custom')).toBeNull();
  });
});
