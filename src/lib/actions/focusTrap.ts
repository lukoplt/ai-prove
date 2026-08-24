const FOCUSABLE =
  'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

export interface FocusTrapOptions {
  /** Called when the user presses Escape. Omit to ignore Escape. */
  onEscape?: () => void;
}

/**
 * Svelte action for modal dialogs: focuses the first control, keeps Tab and
 * Shift+Tab inside the node, routes Escape to `onEscape`, and restores focus to
 * whatever was focused before the dialog opened.
 *
 * Apply it only to a node that is actually mounted while the dialog is open —
 * the lifecycle of the node is the lifecycle of the trap.
 */
export function focusTrap(node: HTMLElement, options: FocusTrapOptions = {}) {
  let current = options;
  const restoreTo = document.activeElement as HTMLElement | null;

  function focusable(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE));
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (!current.onEscape) return;
      event.preventDefault();
      current.onEscape();
      return;
    }

    if (event.key !== 'Tab') return;

    const items = focusable();
    if (items.length === 0) return;

    event.preventDefault();
    const index = items.indexOf(document.activeElement as HTMLElement);
    const delta = event.shiftKey ? -1 : 1;
    items[(index + delta + items.length) % items.length].focus();
  }

  node.addEventListener('keydown', onKeydown);
  queueMicrotask(() => focusable()[0]?.focus());

  return {
    update(next: FocusTrapOptions) {
      current = next;
    },
    destroy() {
      node.removeEventListener('keydown', onKeydown);
      restoreTo?.focus?.();
    },
  };
}
