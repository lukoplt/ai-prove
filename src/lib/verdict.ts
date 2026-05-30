import type { Claim } from './types';

export type VerdictKind = 'mostly_verified' | 'disputed' | 'no_consensus' | 'unverified';

export interface Verdict {
  kind: VerdictKind;
  verified: number;
  total: number;
}

const VERIFIED_THRESHOLD = 0.6;

export function aggregateVerdict(claims: Claim[]): Verdict {
  const verifiable = claims.filter((c) => c.kind === 'fact' && c.verification !== null);
  const total = verifiable.length;
  const verified = verifiable.filter((c) => c.verification?.status === 'supported').length;
  const contradicted = verifiable.some((c) => c.verification?.status === 'contradicted');

  let kind: VerdictKind;
  if (contradicted) kind = 'disputed';
  else if (total === 0) kind = 'unverified';
  else if (verified / total >= VERIFIED_THRESHOLD) kind = 'mostly_verified';
  else kind = 'no_consensus';

  return { kind, verified, total };
}
