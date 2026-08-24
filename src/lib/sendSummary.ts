import type { Settings } from './types';

export interface SendDestination {
  /** i18n key rendered with `tf()`. */
  key: string;
  vars: Record<string, string | number>;
}

export interface SendSummaryInput {
  settings: Settings;
  bravePresent: boolean;
  question: string;
  answer: string;
}

/**
 * Describes, line by line, where this analysis's text is about to go. Pure so
 * the disclosure can be asserted in tests — the modal must never drift from
 * what the pipeline actually does.
 */
export function describeSend(input: SendSummaryInput): SendDestination[] {
  const { settings, bravePresent, question, answer } = input;
  const lines: SendDestination[] = [];

  if (settings.provider === 'anthropic') {
    lines.push({ key: 'send.dest_anthropic', vars: { model: settings.anthropic_model } });
  } else {
    lines.push({ key: 'send.dest_cli', vars: { command: settings.cli_command } });
  }

  if (!bravePresent) {
    lines.push({ key: 'send.web_off', vars: {} });
  } else if (settings.verified_claims_limit === null) {
    lines.push({ key: 'send.web_on_all', vars: {} });
  } else {
    lines.push({ key: 'send.web_on', vars: { limit: settings.verified_claims_limit } });
  }

  lines.push({
    key: 'send.payload',
    vars: { questionChars: question.trim().length, answerChars: answer.trim().length },
  });

  return lines;
}
