import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ClaimText from './ClaimText.svelte';
import { setLocale } from '$lib/stores/i18n.svelte';
import type { Claim } from '$lib/types';

afterEach(cleanup);

const input = 'Karel IV. se narodil v roce 1316. Byl skvělý.';
const claims: Claim[] = [
  {
    id: 'c1',
    text: 'Karel IV. se narodil v roce 1316',
    start: 0,
    end: 32,
    kind: 'fact',
    reason: 'Datum.',
    verification: { status: 'supported', sources: [], summary: 'ok' },
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

function setup(selectedId: string | null = null, onSelect = vi.fn()) {
  setLocale('en');
  const utils = render(ClaimText, { props: { input, claims, selectedId, onSelect } });
  return { ...utils, onSelect };
}

describe('ClaimText rendering', () => {
  it('renders one interactive claim per claim, with its kind class', () => {
    const { container } = setup();
    const buttons = container.querySelectorAll('button.claim');
    expect(buttons.length).toBe(2);
    expect(buttons[0].classList.contains('kind-fact')).toBe(true);
    expect(buttons[1].classList.contains('kind-opinion')).toBe(true);
  });

  it('marks the selected claim', () => {
    const { container } = setup('c2');
    expect(container.querySelector('button.claim.selected')?.getAttribute('data-id')).toBe('c2');
  });

  it('preserves the visible text between and after claims', () => {
    const { container } = setup();
    // The screen-reader labels are extra text nodes; strip them before comparing.
    for (const node of container.querySelectorAll('.sr-only')) node.remove();
    expect(container.textContent?.replaceAll(/\s+/g, ' ').trim()).toBe(input);
  });
});

describe('ClaimText accessibility', () => {
  it('exposes each claim as a real button', () => {
    setup();
    expect(screen.getAllByRole('button')).toHaveLength(2);
  });

  it('names each claim with its text and epistemic kind', () => {
    setup();
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAccessibleName(
      /Verifiable fact/,
    );
    expect(screen.getByRole('button', { name: /skvělý/ })).toHaveAccessibleName(/Opinion/);
  });

  it('announces the verification status when there is one', () => {
    setup();
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAccessibleName(/Verified/);
  });

  it('marks the selected claim with aria-pressed', () => {
    setup('c1');
    expect(screen.getByRole('button', { name: /Karel IV\./ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('keeps exactly one claim in the tab order', () => {
    const { container } = setup('c2');
    const tabbable = container.querySelectorAll('button.claim[tabindex="0"]');
    expect(tabbable).toHaveLength(1);
    expect(tabbable[0].getAttribute('data-id')).toBe('c2');
  });

  it('selects the next claim with ArrowRight', async () => {
    const { onSelect } = setup('c1');
    await fireEvent.keyDown(screen.getByRole('button', { name: /Karel IV\./ }), {
      key: 'ArrowRight',
    });
    expect(onSelect).toHaveBeenCalledWith('c2');
  });

  it('wraps around from the last claim to the first', async () => {
    const { onSelect } = setup('c2');
    await fireEvent.keyDown(screen.getByRole('button', { name: /skvělý/ }), { key: 'ArrowRight' });
    expect(onSelect).toHaveBeenCalledWith('c1');
  });
});
