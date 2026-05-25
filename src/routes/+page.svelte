<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { isTauriRuntime, readClipboardText } from '$lib/api';
  import ClaimText from '$lib/components/ClaimText.svelte';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import SidePanel from '$lib/components/SidePanel.svelte';
  import UpdateBanner from '$lib/components/UpdateBanner.svelte';
  import { analysisPreflightError } from '$lib/preflight';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import type { AnalyzeInput } from '$lib/types';

  let questionText = $state('');
  let answerText = $state('');

  function preflightError(): string | null {
    return analysisPreflightError({
      isNative: isTauriRuntime(),
      anthropicPresent: settings.anthropicPresent,
      settings: settings.current,
      messages: {
        missingAnthropicKey: t('summary.missing_anthropic_key'),
        missingAnthropicModel: t('summary.missing_anthropic_model'),
        missingCliCommand: t('summary.missing_cli_command'),
      },
    });
  }

  async function handleAnalyze(input: AnalyzeInput) {
    const error = preflightError();
    if (error) {
      alert(error);
      await goto(resolve('/settings'));
      return;
    }

    questionText = input.question ?? '';
    answerText = input.answer;
    await analysisStore.run(input);
  }

  onMount(() => {
    void analysisStore.init();
    if (!isTauriRuntime()) return;

    const unlisten = import('@tauri-apps/api/event').then(({ listen }) =>
      listen('capture-trigger', async () => {
        const clipboard = await readClipboardText();
        if (clipboard) answerText = clipboard;
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

  <UpdateBanner />

  <PasteInput bind:question={questionText} bind:answer={answerText} onAnalyze={handleAnalyze} />

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
          <div class="claim-scroll">
            <ClaimText
              input={analysisStore.current.input}
              claims={analysisStore.current.claims}
              selectedId={analysisStore.selectedId}
              onSelect={(id) => analysisStore.select(id)}
            />
          </div>
        </div>
        <div class="side-scroll">
          <SidePanel claim={analysisStore.selectedClaim} />
        </div>
      </div>
    {/if}
  </section>

  <footer class="disclaimer">{t('footer.disclaimer')}</footer>
</main>

<style>
  .page {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    height: 100vh;
    max-width: 960px;
    margin: 0 auto;
    padding: 18px 24px 12px;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 10px;
    flex: 0 0 auto;
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
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
    margin-top: 10px;
    overflow: hidden;
  }

  .status {
    color: #71717a;
    font-size: 14px;
  }

  .status.error {
    color: #b91c1c;
  }

  .grid {
    flex: 1 1 auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    gap: 16px;
    min-height: 0;
    width: 100%;
  }

  .left {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 16px;
    border: 1px solid #e4e4e7;
    border-radius: 8px;
    background: #ffffff;
  }

  .claim-scroll {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    padding-right: 4px;
  }

  .side-scroll {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
  }

  .meta {
    margin: 0 0 8px;
    color: #71717a;
    font-size: 13px;
    flex: 0 0 auto;
  }

  .warning {
    margin: 0 0 8px;
    padding: 7px 10px;
    border-radius: 6px;
    background: #fef3c7;
    color: #92400e;
    font-size: 13px;
    flex: 0 0 auto;
  }

  .disclaimer {
    flex: 0 0 auto;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid #e4e4e7;
    color: #71717a;
    font-size: 11px;
    line-height: 1.4;
    text-align: center;
  }

  @media (max-width: 820px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
