import type { Settings } from './types';

export interface PreflightMessages {
  missingAnthropicKey: string;
  missingAnthropicModel: string;
  missingCliCommand: string;
}

export interface AnalysisPreflightInput {
  isNative: boolean;
  anthropicPresent: boolean;
  settings: Settings;
  messages: PreflightMessages;
}

export function analysisPreflightError(input: AnalysisPreflightInput): string | null {
  if (!input.isNative) return null;

  const current = input.settings;
  if (current.provider === 'anthropic') {
    if (!input.anthropicPresent) return input.messages.missingAnthropicKey;
    if (!current.anthropic_model?.trim()) return input.messages.missingAnthropicModel;
    return null;
  }

  if (current.provider === 'cli') {
    if (!current.cli_command?.trim()) return input.messages.missingCliCommand;
    return null;
  }

  return null;
}
