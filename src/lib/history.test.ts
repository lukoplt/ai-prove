import { describe, expect, it } from 'vitest';
import { formatHistoryDate } from './history';

// 2026-05-30T14:05:00Z
const STAMP = Date.UTC(2026, 4, 30, 14, 5, 0);

describe('formatHistoryDate', () => {
  it('formats Czech with the day and year present', () => {
    const formatted = formatHistoryDate(STAMP, 'cs');
    expect(formatted).toMatch(/30/);
    expect(formatted).toMatch(/2026/);
  });

  it('formats English', () => {
    const formatted = formatHistoryDate(STAMP, 'en');
    expect(formatted).toMatch(/2026/);
  });

  it('returns an empty string for a non-finite stamp', () => {
    expect(formatHistoryDate(Number.NaN, 'cs')).toBe('');
  });
});
