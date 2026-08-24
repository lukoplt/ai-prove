<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = false,
    title,
    confirmLabel,
    cancelLabel,
    confirmDisabled = false,
    onConfirm,
    onCancel,
    children,
  }: {
    open?: boolean;
    title: string;
    confirmLabel: string;
    cancelLabel: string;
    confirmDisabled?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
    children?: Snippet;
  } = $props();

  const titleId = `confirm-title-${Math.random().toString(36).slice(2, 9)}`;

  const FOCUSABLE =
    'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

  let dialog = $state<HTMLDivElement | null>(null);
  let restoreFocus: HTMLElement | null = null;

  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE));
  }

  // Focus the first control when the dialog opens and restore the previously
  // focused element when it closes, so keyboard and screen-reader users land
  // back where they were.
  $effect(() => {
    if (!open) return;

    restoreFocus = document.activeElement as HTMLElement | null;
    queueMicrotask(() => focusable()[0]?.focus());

    return () => {
      restoreFocus?.focus?.();
      restoreFocus = null;
    };
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
      return;
    }

    if (event.key !== 'Tab') return;

    const items = focusable();
    if (items.length === 0) return;

    event.preventDefault();
    const index = items.indexOf(document.activeElement as HTMLElement);
    const delta = event.shiftKey ? -1 : 1;
    const next = (index + delta + items.length) % items.length;
    items[next].focus();
  }
</script>

{#if open}
  <div class="backdrop">
    <div
      class="dialog glass"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      bind:this={dialog}
      onkeydown={onKeydown}
    >
      <h2 id={titleId}>{title}</h2>
      <div class="body">
        {@render children?.()}
      </div>
      <div class="actions">
        <button type="button" onclick={onCancel}>{cancelLabel}</button>
        <button type="button" class="primary" disabled={confirmDisabled} onclick={onConfirm}>
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: rgba(9, 9, 14, 0.45);
  }

  .dialog {
    width: min(520px, 100%);
    max-height: 85vh;
    overflow-y: auto;
    padding: var(--space-5);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: 18px;
    letter-spacing: -0.01em;
  }

  .body {
    margin-bottom: var(--space-4);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
