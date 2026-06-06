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

export interface AnalyzeInput {
  question?: string;
  answer: string;
}

export type ProviderKind = 'cli' | 'anthropic';

export type ThemePref = 'auto' | 'light' | 'dark';

export interface Settings {
  locale: 'cs' | 'en';
  hotkey: string;
  cache_ttl_days: number;
  onboarded: boolean;
  provider: ProviderKind;
  anthropic_model: string;
  cli_command: string;
  check_updates_on_launch: boolean;
  theme: ThemePref;
  /** How many factual claims to verify against the web. `null` means all. */
  verified_claims_limit: number | null;
}

export interface LatestRelease {
  currentVersion: string;
  latestVersion: string;
  isNewer: boolean;
  htmlUrl: string;
  publishedAt: string;
  body: string;
}

export const DEFAULT_ANTHROPIC_MODEL = 'claude-haiku-4-5-20251001';
export const DEFAULT_CLI_COMMAND = 'claude -p';
export const DEFAULT_VERIFIED_CLAIMS_LIMIT = 8;

/** Selectable web-verification limits for the settings UI. `null` = all. */
export const VERIFIED_CLAIMS_LIMIT_OPTIONS: Array<number | null> = [4, 8, 12, 16, 20, null];

export const ACCOUNT_ANTHROPIC = 'anthropic';
export const ACCOUNT_BRAVE = 'brave';
export type ApiAccount = typeof ACCOUNT_ANTHROPIC | typeof ACCOUNT_BRAVE;
