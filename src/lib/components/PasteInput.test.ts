import { fireEvent, render, screen } from '@testing-library/svelte';
import '@testing-library/jest-dom/vitest';
import { beforeEach, describe, expect, it } from 'vitest';
import PasteInput from './PasteInput.svelte';
import { setLocale } from '$lib/stores/i18n.svelte';

describe('PasteInput', () => {
  beforeEach(() => setLocale('cs'));

  it('clears only the answer when starting a new chat from the same question', async () => {
    render(PasteInput, {
      question: 'Původní dotaz',
      answer: 'Stará odpověď',
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Vymazat odpověď' }));

    expect(screen.getByLabelText('Dotaz')).toHaveValue('Původní dotaz');
    expect(screen.getByLabelText('Odpověď AI')).toHaveValue('');
  });
});
