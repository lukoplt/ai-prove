<script lang="ts">
  import { focusTrap } from '$lib/actions/focusTrap';
  import type { Snippet } from 'svelte';

  let {
    open = false,
    title,
    confirmLabel,
    cancelLabel,
    confirmDisabled = false,
    destructive = false,
    onConfirm,
    onCancel,
    children,
  }: {
    open?: boolean;
    title: string;
    confirmLabel: string;
    cancelLabel: string;
    confirmDisabled?: boolean;
    /** Styles the confirm action as destructive (permanent deletion). */
    destructive?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
    children?: Snippet;
  } = $props();

  const titleId = `confirm-title-${Math.random().toString(36).slice(2, 9)}`;
</script>

{#if open}
  <div class="backdrop">
    <div
      class="dialog glass"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      use:focusTrap={{ onEscape: onCancel }}
    >
      <h2 id={titleId}>{title}</h2>
      <div class="body">
        {@render children?.()}
      </div>
      <div class="actions">
        <button type="button" onclick={onCancel}>{cancelLabel}</button>
        <button
          type="button"
          class="primary"
          class:destructive
          disabled={confirmDisabled}
          onclick={onConfirm}
        >
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

  /* Permanent deletion reads as destructive, not as the happy path. */
  .primary.destructive {
    border-color: var(--bad);
    background: var(--bad);
  }
  .primary.destructive:hover:not(:disabled) {
    border-color: var(--bad);
    background: var(--bad);
    filter: brightness(0.92);
  }
</style>
