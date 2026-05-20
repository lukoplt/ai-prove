export type ClaimKind = 'fact' | 'inference' | 'opinion' | 'contradiction';

export type VerificationStatus =
  | 'supported'
  | 'contradicted'
  | 'no_consensus'
  | 'not_found'
  | 'not_verified';

export type SourceTier = 'a' | 'b' | 'c' | 'd';

export type SourceStance = 'supports' | 'contradicts' | 'mentions';

export interface SourceHit {
  url: string;
  title: string;
  snippet: string;
  tier: SourceTier;
  stance: SourceStance;
}

export interface Verification {
  status: VerificationStatus;
  sources: SourceHit[];
  summary: string;
}

export interface Claim {
  id: string;
  text: string;
  start: number;
  end: number;
  kind: ClaimKind;
  reason: string;
  verification: Verification | null;
}

export interface Analysis {
  id: string;
  created_at: number;
  input: string;
  claims: Claim[];
  truncated: boolean;
}

export interface Settings {
  locale: 'cs' | 'en';
  hotkey: string;
  model: string;
  cache_ttl_days: number;
  onboarded: boolean;
}

export const ACCOUNT_ANTHROPIC = 'anthropic';
export const ACCOUNT_BRAVE = 'brave';
export type ApiAccount = typeof ACCOUNT_ANTHROPIC | typeof ACCOUNT_BRAVE;
