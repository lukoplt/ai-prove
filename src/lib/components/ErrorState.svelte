<script lang="ts">
  import { errorKey, isSettingsError, type AppErrorPayload } from '$lib/errors';
  import { t } from '$lib/stores/i18n.svelte';

  let {
    error,
    onRetry,
    onSettings,
    onDismiss,
  }: {
    error: AppErrorPayload;
    onRetry?: () => void;
    onSettings?: () => void;
    onDismiss?: () => void;
  } = $props();

  const headline = $derived(t(errorKey(error.code)));
  const showSettings = $derived(Boolean(onSettings) && isSettingsError(error.code));
</script>

<div class="err glass" role="alert">
  <div class="body">
    <strong class="title">{t('error.title')}</strong>
    <p class="msg">{headline}</p>
    <details>
      <summary>{t('error.details')}</summary>
      <pre>{error.message}</pre>
    </details>
  </div>
  <div class="actions">
    {#if onRetry}
      <button type="button" class="primary" onclick={onRetry}>{t('error.retry')}</button>
    {/if}
    {#if showSettings}
      <button type="button" onclick={onSettings}>{t('error.open_settings')}</button>
    {/if}
    {#if onDismiss}
      <button type="button" onclick={onDismiss}>{t('error.dismiss')}</button>
    {/if}
  </div>
</div>

<style>
  .err {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border-color: var(--bad);
  }
  .body {
    min-width: 0;
    flex: 1 1 320px;
  }
  .title {
    display: block;
    color: var(--bad);
    font-size: 14px;
  }
  .msg {
    margin: var(--space-1) 0 var(--space-2);
    color: var(--text);
    font-size: 14px;
    line-height: 1.45;
  }
  summary {
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
  }
  pre {
    max-height: 160px;
    margin: var(--space-2) 0 0;
    padding: var(--space-2);
    overflow: auto;
    border-radius: var(--radius-sm);
    background: var(--neutral-soft);
    color: var(--text-muted);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
