import { describe, expect, it } from 'vitest';
import { aggregateVerdict } from './verdict';
import type { Claim } from './types';

function claim(partial: Partial<Claim>): Claim {
  return {
    id: 'x',
    text: 't',
    start: 0,
    end: 1,
    kind: 'fact',
    reason: 'r',
    verification: null,
    ...partial,
  };
}

describe('aggregateVerdict', () => {
  it('is unverified when no fact claims are verified', () => {
    const v = aggregateVerdict([claim({ kind: 'opinion' }), claim({ verification: null })]);
    expect(v.kind).toBe('unverified');
    expect(v.total).toBe(0);
    expect(v.verified).toBe(0);
  });

  it('is disputed when any fact claim is contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'contradicted', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('disputed');
  });

  it('is mostly_verified when 60%+ supported and none contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'no_consensus', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('mostly_verified');
    expect(v.verified).toBe(2);
    expect(v.total).toBe(3);
  });

  it('is no_consensus when verified below 60% and none contradicted', () => {
    const v = aggregateVerdict([
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
      claim({ verification: { status: 'no_consensus', sources: [], summary: '' } }),
      claim({ verification: { status: 'not_found', sources: [], summary: '' } }),
    ]);
    expect(v.kind).toBe('no_consensus');
    expect(v.verified).toBe(1);
    expect(v.total).toBe(3);
  });

  it('ignores non-fact claims and unverified fact claims in totals', () => {
    const v = aggregateVerdict([
      claim({ kind: 'opinion' }),
      claim({ kind: 'fact', verification: null }),
      claim({ verification: { status: 'supported', sources: [], summary: '' } }),
    ]);
    expect(v.total).toBe(1);
    expect(v.verified).toBe(1);
    expect(v.kind).toBe('mostly_verified');
  });
});
