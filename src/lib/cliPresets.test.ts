import { describe, expect, it } from 'vitest';
import { CLI_PRESETS, commandToCliPreset, presetCommand } from './cliPresets';
import cs from './i18n/cs.json';
import en from './i18n/en.json';

describe('CLI presets', () => {
  it('offers multiple CLI-backed providers', () => {
    expect(CLI_PRESETS.map((preset) => preset.id)).toEqual(['claude', 'codex', 'ollama', 'custom']);
  });

  it('gives every non-custom preset a concrete command', () => {
    for (const preset of CLI_PRESETS) {
      if (preset.id === 'custom') {
        expect(preset.command).toBeNull();
      } else {
        expect(preset.command?.trim().length).toBeGreaterThan(0);
      }
    }
  });

  it('round-trips every preset command back to its id', () => {
    for (const preset of CLI_PRESETS) {
      if (preset.command === null) continue;
      expect(commandToCliPreset(preset.command)).toBe(preset.id);
      expect(presetCommand(preset.id)).toBe(preset.command);
    }
  });

  it('ignores surrounding whitespace when matching', () => {
    expect(commandToCliPreset('  claude -p  ')).toBe('claude');
  });

  it('falls back to custom for an unknown or empty command', () => {
    expect(commandToCliPreset('my-llm --json')).toBe('custom');
    expect(commandToCliPreset('')).toBe('custom');
    expect(presetCommand('custom')).toBeNull();
  });

  it('has a label for every preset in both locales', () => {
    for (const preset of CLI_PRESETS) {
      expect(cs.settings).toHaveProperty(`cli_preset_${preset.id}`);
      expect(en.settings).toHaveProperty(`cli_preset_${preset.id}`);
    }
  });
});
