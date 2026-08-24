<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { isTauriRuntime, readClipboardText } from '$lib/api';
  import ClaimText from '$lib/components/ClaimText.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import ErrorState from '$lib/components/ErrorState.svelte';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import SendConfirm from '$lib/components/SendConfirm.svelte';
  import SidePanel from '$lib/components/SidePanel.svelte';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import UpdateBanner from '$lib/components/UpdateBanner.svelte';
  import VerdictBanner from '$lib/components/VerdictBanner.svelte';
  import type { AppErrorPayload } from '$lib/errors';
  import { analysisPreflightError } from '$lib/preflight';
  import { analysisStore } from '$lib/stores/analysis.svelte';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import type { AnalyzeInput } from '$lib/types';

  let questionText = $state('');
  let answerText = $state('');
  let preflight = $state<AppErrorPayload | null>(null);
  let pendingInput = $state<AnalyzeInput | null>(null);
  let dontAskAgain = $state(false);

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
    const message = preflightError();
    if (message) {
      preflight = { code: 'invalid', message };
      return;
    }

    preflight = null;
    if (settings.current.confirm_before_send) {
      pendingInput = input;
      return;
    }

    await start(input);
  }

  async function confirmSend() {
    const input = pendingInput;
    pendingInput = null;
    if (!input) return;

    if (dontAskAgain) {
      await settings.save({ ...settings.current, confirm_before_send: false });
      dontAskAgain = false;
    }

    await start(input);
  }

  async function start(input: AnalyzeInput) {
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
  <header class="topbar glass">
    <div class="brand">
      <h1>{t('app.title')}</h1>
      <p>{t('app.tagline')}</p>
    </div>
    <nav>
      <ThemeToggle />
      <button type="button" onclick={() => goto(resolve('/history'))}>
        {t('common.history')}
      </button>
      <button type="button" onclick={() => goto(resolve('/settings'))}>
        {t('common.settings')}
      </button>
    </nav>
  </header>

  <UpdateBanner />

  {#if preflight}
    <ErrorState
      error={preflight}
      onSettings={() => goto(resolve('/settings'))}
      onDismiss={() => (preflight = null)}
    />
  {/if}

  <PasteInput bind:question={questionText} bind:answer={answerText} onAnalyze={handleAnalyze} />

  <section class="result">
    {#if analysisStore.status === 'running'}
      <div class="loading glass">
        <span class="spinner" aria-hidden="true"></span>
        <div>
          <p class="status">{t('summary.analyzing')}</p>
          <p class="hint">{t('summary.loading_hint')}</p>
        </div>
      </div>
    {:else if analysisStore.status === 'error' && analysisStore.error}
      <ErrorState
        error={analysisStore.error}
        onRetry={() => analysisStore.retry()}
        onSettings={() => goto(resolve('/settings'))}
      />
    {:else if analysisStore.status === 'done' && analysisStore.current}
      <div class="grid">
        <div class="left glass">
          <VerdictBanner claims={analysisStore.current.claims} />
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

  <ConfirmModal
    open={pendingInput !== null}
    title={t('send.title')}
    confirmLabel={t('send.confirm')}
    cancelLabel={t('send.cancel')}
    onConfirm={confirmSend}
    onCancel={() => {
      pendingInput = null;
      dontAskAgain = false;
    }}
  >
    <SendConfirm
      question={pendingInput?.question ?? ''}
      answer={pendingInput?.answer ?? ''}
      bind:dontAsk={dontAskAgain}
    />
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
    padding: var(--space-4) var(--space-6) var(--space-3);
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    margin-bottom: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    flex: 0 0 auto;
  }

  .brand h1 {
    margin: 0;
    font-size: 22px;
    line-height: 1.1;
    letter-spacing: -0.01em;
  }
  .brand p {
    margin: 2px 0 0;
    color: var(--text-subtle);
    font-size: 13px;
  }
  nav {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .result {
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
    margin-top: var(--space-3);
    overflow: hidden;
  }

  .loading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-md);
  }
  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid var(--accent-soft);
    border-top-color: var(--accent);
    border-radius: 999px;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .status {
    margin: 0;
    color: var(--text-muted);
    font-size: 14px;
  }
  .hint {
    margin: 2px 0 0;
    color: var(--text-subtle);
    font-size: 13px;
  }
  .grid {
    flex: 1 1 auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: var(--space-4);
    min-height: 0;
    width: 100%;
  }

  .left {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: var(--space-4);
    border-radius: var(--radius-lg);
  }

  .claim-scroll {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    padding-right: var(--space-1);
  }

  .side-scroll {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
  }

  .meta {
    margin: 0 0 var(--space-2);
    color: var(--text-subtle);
    font-size: 13px;
    flex: 0 0 auto;
  }

  .warning {
    margin: 0 0 var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--warn-soft);
    color: var(--warn);
    font-size: 13px;
    flex: 0 0 auto;
  }

  .disclaimer {
    flex: 0 0 auto;
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--surface-glass-border);
    color: var(--text-subtle);
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
