import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ClaimText from './ClaimText.svelte';
import type { Claim } from '$lib/types';

const input = 'Karel IV. se narodil v roce 1316. Byl skvělý.';
const claims: Claim[] = [
  {
    id: 'c1',
    text: 'Karel IV. se narodil v roce 1316',
    start: 0,
    end: 32,
    kind: 'fact',
    reason: 'Datum.',
    verification: null,
  },
  {
    id: 'c2',
    text: 'Byl skvělý',
    start: 34,
    end: 44,
    kind: 'opinion',
    reason: 'Hodnocení.',
    verification: null,
  },
];

describe('ClaimText', () => {
  it('renders highlighted spans with kind classes', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: null });
    const spans = container.querySelectorAll('span.claim');
    expect(spans.length).toBe(2);
    expect(spans[0].classList.contains('kind-fact')).toBe(true);
    expect(spans[1].classList.contains('kind-opinion')).toBe(true);
  });

  it('marks the selected claim', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: 'c2' });
    const selected = container.querySelector('span.claim.selected');
    expect(selected?.getAttribute('data-id')).toBe('c2');
  });

  it('preserves text between and after claims', () => {
    const { container } = render(ClaimText, { input, claims, selectedId: null });
    expect(container.textContent?.replaceAll(/\s+/g, ' ').trim()).toBe(input);
  });
});
