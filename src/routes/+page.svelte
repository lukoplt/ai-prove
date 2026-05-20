<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { isTauriRuntime, readClipboardText } from '$lib/api';
  import ClaimText from '$lib/components/ClaimText.svelte';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import SidePanel from '$lib/components/SidePanel.svelte';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let inputText = $state('');

  async function handleAnalyze(text: string) {
    if (isTauriRuntime() && (!settings.anthropicPresent || !settings.bravePresent)) {
      alert(t('summary.missing_keys'));
      await goto(resolve('/settings'));
      return;
    }

    inputText = text;
    await analysisStore.run(text);
  }

  onMount(() => {
    void analysisStore.init();
    if (!isTauriRuntime()) return;

    const unlisten = import('@tauri-apps/api/event').then(({ listen }) =>
      listen('capture-trigger', async () => {
        const clipboard = await readClipboardText();
        if (clipboard) inputText = clipboard;
      }),
    );

    return () => {
      unlisten.then((unsubscribe) => unsubscribe());
    };
  });
</script>

<main class="page">
  <header>
    <div>
      <h1>{t('app.title')}</h1>
      <p>{t('app.tagline')}</p>
    </div>
    <nav>
      <button type="button" onclick={() => goto(resolve('/settings'))}>
        {t('common.settings')}
      </button>
    </nav>
  </header>

  <PasteInput bind:value={inputText} onAnalyze={handleAnalyze} />

  <section class="result">
    {#if analysisStore.status === 'running'}
      <p class="status">{t('summary.analyzing')}</p>
    {:else if analysisStore.status === 'error'}
      <p class="status error">{tf('summary.error_prefix', { msg: analysisStore.error ?? '?' })}</p>
    {:else if analysisStore.status === 'done' && analysisStore.current}
      <div class="grid">
        <div class="left">
          <p class="meta">
            {tf('summary.claims_count', { count: analysisStore.current.claims.length })}
          </p>
          {#if analysisStore.current.truncated}
            <p class="warning">{t('summary.truncated_warning')}</p>
          {/if}
          <ClaimText
            input={analysisStore.current.input}
            claims={analysisStore.current.claims}
            selectedId={analysisStore.selectedId}
            onSelect={(id) => analysisStore.select(id)}
          />
        </div>
        <SidePanel claim={analysisStore.selectedClaim} />
      </div>
    {/if}
  </section>
</main>

<style>
  .page {
    max-width: 960px;
    margin: 0 auto;
    padding: 28px;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 18px;
  }

  h1 {
    margin: 0 0 5px;
    font-size: 28px;
    line-height: 1.1;
  }

  p {
    margin: 0;
    color: #71717a;
  }

  .result {
    margin-top: 18px;
  }

  .status {
    color: #71717a;
    font-size: 14px;
  }

  .status.error {
    color: #b91c1c;
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    gap: 16px;
  }

  .left {
    min-width: 0;
    padding: 16px;
    border: 1px solid #e4e4e7;
    border-radius: 8px;
    background: #ffffff;
  }

  .meta {
    margin: 0 0 8px;
    color: #71717a;
    font-size: 13px;
  }

  .warning {
    margin: 0 0 8px;
    padding: 7px 10px;
    border-radius: 6px;
    background: #fef3c7;
    color: #92400e;
    font-size: 13px;
  }

  @media (max-width: 820px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
