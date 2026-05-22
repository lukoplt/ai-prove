export type CliPresetId = 'claude' | 'codex' | 'ollama' | 'custom';

export interface CliPreset {
  id: CliPresetId;
  command: string | null;
}

export const CLI_PRESETS: CliPreset[] = [
  { id: 'claude', command: 'claude -p' },
  { id: 'codex', command: 'codex exec -' },
  { id: 'ollama', command: 'ollama run qwen2.5-coder:7b' },
  { id: 'custom', command: null },
];

export function commandToCliPreset(command: string): CliPresetId {
  const normalized = command.trim();
  return CLI_PRESETS.find((preset) => preset.command === normalized)?.id ?? 'custom';
}

export function presetCommand(id: CliPresetId): string | null {
  return CLI_PRESETS.find((preset) => preset.id === id)?.command ?? null;
}
