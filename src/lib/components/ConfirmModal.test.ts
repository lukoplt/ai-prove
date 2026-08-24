import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ConfirmModal from './ConfirmModal.svelte';

afterEach(cleanup);

function open(overrides: Record<string, unknown> = {}) {
  return render(ConfirmModal, {
    props: {
      open: true,
      title: 'Send this?',
      confirmLabel: 'Send',
      cancelLabel: 'Cancel',
      onConfirm: vi.fn(),
      onCancel: vi.fn(),
      ...overrides,
    },
  });
}

describe('ConfirmModal', () => {
  it('exposes a labelled modal dialog', () => {
    open();
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAccessibleName('Send this?');
  });

  it('renders nothing when closed', () => {
    render(ConfirmModal, {
      props: {
        open: false,
        title: 'Send this?',
        confirmLabel: 'Send',
        cancelLabel: 'Cancel',
        onConfirm: vi.fn(),
        onCancel: vi.fn(),
      },
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('cancels on Escape', async () => {
    const onCancel = vi.fn();
    open({ onCancel });
    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('confirms via the confirm button', async () => {
    const onConfirm = vi.fn();
    open({ onConfirm });
    await fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('keeps Tab inside the dialog', async () => {
    open();
    const confirm = screen.getByRole('button', { name: 'Send' });
    const cancel = screen.getByRole('button', { name: 'Cancel' });

    confirm.focus();
    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab' });
    expect(document.activeElement).toBe(cancel);

    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(confirm);
  });
});
