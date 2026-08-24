<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { clearHistory, deleteAnalysis, getAnalysis, listHistory } from '$lib/api';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import ErrorState from '$lib/components/ErrorState.svelte';
  import { toAppError, type AppErrorPayload } from '$lib/errors';
  import { formatHistoryDate } from '$lib/history';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { getLocale, t, tf } from '$lib/stores/i18n.svelte';
  import type { HistoryEntry } from '$lib/types';

  let entries = $state<HistoryEntry[]>([]);
  let query = $state('');
  let loading = $state(true);
  let failure = $state<AppErrorPayload | null>(null);
  let pendingDelete = $state<HistoryEntry | null>(null);
  let confirmClear = $state(false);
  let notice = $state<string | null>(null);
  let debounce: ReturnType<typeof setTimeout> | null = null;

  async function refresh() {
    loading = true;
    failure = null;
    try {
      entries = await listHistory(query);
    } catch (caught) {
      failure = toAppError(caught);
    } finally {
      loading = false;
    }
  }

  function onSearch() {
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(() => void refresh(), 200);
  }

  async function open(entry: HistoryEntry) {
    failure = null;
    try {
      analysisStore.show(await getAnalysis(entry.id));
      await goto(resolve('/'));
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  async function confirmDelete() {
    const entry = pendingDelete;
    pendingDelete = null;
    if (!entry) return;

    try {
      await deleteAnalysis(entry.id);
      await refresh();
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  async function confirmClearAll() {
    confirmClear = false;
    try {
      const removed = await clearHistory();
      notice = tf('history.cleared', { count: removed });
      await refresh();
    } catch (caught) {
      failure = toAppError(caught);
    }
  }

  onMount(refresh);
</script>

<main id="main" class="page">
  <header class="topbar glass">
    <button type="button" onclick={() => goto(resolve('/'))}>{t('settings.back')}</button>
    <h1>{t('history.title')}</h1>
    <div class="spacer"></div>
    <button type="button" onclick={() => (confirmClear = true)} disabled={entries.length === 0}>
      {t('history.clear_all')}
    </button>
  </header>

  <label class="search">
    <span class="sr-only">{t('history.search_label')}</span>
    <input
      type="search"
      bind:value={query}
      oninput={onSearch}
      placeholder={t('history.search_placeholder')}
    />
  </label>

  {#if failure}
    <ErrorState error={failure} onRetry={refresh} onDismiss={() => (failure = null)} />
  {/if}

  {#if notice}
    <p class="notice" role="status">{notice}</p>
  {/if}

  <section class="list" aria-busy={loading} aria-live="polite">
    {#if loading}
      <p class="muted">{t('history.loading')}</p>
    {:else if entries.length === 0}
      <p class="muted">{query.trim() ? t('history.empty_search') : t('history.empty')}</p>
    {:else}
      <ul>
        {#each entries as entry (entry.id)}
          <li class="row glass">
            <div class="meta">
              <time datetime={new Date(entry.created_at).toISOString()}>
                {formatHistoryDate(entry.created_at, getLocale())}
              </time>
              <span class="count">{tf('history.claims', { count: entry.claim_count })}</span>
            </div>
            <p class="preview">{entry.preview}</p>
            <div class="row-actions">
              <button type="button" class="primary" onclick={() => open(entry)}>
                {t('history.open')}
              </button>
              <button type="button" onclick={() => (pendingDelete = entry)}>
                {t('history.delete')}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <ConfirmModal
    open={pendingDelete !== null}
    title={t('history.delete_confirm_title')}
    confirmLabel={t('history.delete')}
    cancelLabel={t('common.cancel')}
    onConfirm={confirmDelete}
    onCancel={() => (pendingDelete = null)}
  >
    <p>{t('history.delete_confirm_body')}</p>
  </ConfirmModal>

  <ConfirmModal
    open={confirmClear}
    title={t('history.clear_confirm_title')}
    confirmLabel={t('history.clear_all')}
    cancelLabel={t('common.cancel')}
    onConfirm={confirmClearAll}
    onCancel={() => (confirmClear = false)}
  >
    <p>{t('history.clear_confirm_body')}</p>
  </ConfirmModal>
</main>

<style>
  .page {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    height: 100vh;
    max-width: 980px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6);
    gap: var(--space-3);
    overflow: hidden;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    flex: 0 0 auto;
  }
  h1 {
    margin: 0;
    font-size: 22px;
    letter-spacing: -0.01em;
  }
  .spacer {
    flex: 1;
  }
  .search input {
    width: 100%;
  }
  .list {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
  }
  ul {
    display: grid;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .row {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }
  .meta {
    display: flex;
    gap: var(--space-3);
    color: var(--text-muted);
    font-size: 12px;
  }
  .preview {
    margin: 0;
    color: var(--text);
    font-size: 14px;
    line-height: 1.5;
  }
  .row-actions {
    display: flex;
    gap: var(--space-2);
  }
  .muted {
    color: var(--text-muted);
    font-size: 14px;
  }
  .notice {
    margin: 0;
    color: var(--ok);
    font-size: 13px;
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
