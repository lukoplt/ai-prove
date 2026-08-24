import type { Settings } from './types';

export const ONBOARDING_STEPS = ['welcome', 'privacy', 'provider', 'ready'] as const;

export type OnboardingStep = (typeof ONBOARDING_STEPS)[number];

function shift(step: OnboardingStep, delta: number): OnboardingStep {
  const index = ONBOARDING_STEPS.indexOf(step);
  const next = Math.min(Math.max(index + delta, 0), ONBOARDING_STEPS.length - 1);
  return ONBOARDING_STEPS[next];
}

export function nextStep(step: OnboardingStep): OnboardingStep {
  return shift(step, 1);
}

export function prevStep(step: OnboardingStep): OnboardingStep {
  return shift(step, -1);
}

/**
 * Whether the user may leave `step`. Only the provider step gates: finishing
 * onboarding with no working provider would drop the user straight into a
 * failed analysis.
 *
 * `anthropicKeyEntered` is true when a key is already in the keychain or the
 * user typed one into the onboarding field.
 */
export function canAdvance(
  step: OnboardingStep,
  draft: Settings,
  anthropicKeyEntered: boolean,
): boolean {
  if (step !== 'provider') return true;

  if (draft.provider === 'anthropic') {
    return anthropicKeyEntered && draft.anthropic_model.trim().length > 0;
  }

  return draft.cli_command.trim().length > 0;
}
