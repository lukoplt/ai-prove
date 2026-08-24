<script lang="ts">
  import { setApiKey } from '$lib/api';
  import {
    CLI_PRESETS,
    commandToCliPreset,
    presetCommand,
    type CliPresetId,
  } from '$lib/cliPresets';
  import { toAppError, type AppErrorPayload } from '$lib/errors';
  import { formatAccelerator, platformKind } from '$lib/hotkey';
  import {
    canAdvance,
    nextStep,
    ONBOARDING_STEPS,
    prevStep,
    type OnboardingStep,
  } from '$lib/onboarding';
  import { setLocale, t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type Settings } from '$lib/types';
  import ErrorState from './ErrorState.svelte';

  let { onDone }: { onDone: () => void } = $props();

  let step = $state<OnboardingStep>('welcome');
  let draft = $state<Settings>({ ...settings.current });
  let cliPreset = $state<CliPresetId>(commandToCliPreset(settings.current.cli_command));
  let anthropicInput = $state('');
  let braveInput = $state('');
  let busy = $state(false);
  let failure = $state<AppErrorPayload | null>(null);

  const platform = platformKind();
  const index = $derived(ONBOARDING_STEPS.indexOf(step));
  const isLast = $derived(step === 'ready');
  const advanceAllowed = $derived(
    canAdvance(step, draft, settings.anthropicPresent || anthropicInput.trim().length > 0),
  );
  const hotkeyLabel = $derived(formatAccelerator(draft.hotkey, platform));

  function applyCliPreset() {
    const command = presetCommand(cliPreset);
    if (command) draft.cli_command = command;
  }

  async function persistKeys() {
    if (anthropicInput.trim()) await setApiKey(ACCOUNT_ANTHROPIC, anthropicInput.trim());
    if (braveInput.trim()) await setApiKey(ACCOUNT_BRAVE, braveInput.trim());
    if (anthropicInput.trim() || braveInput.trim()) await settings.refreshKeyState();
    anthropicInput = '';
    braveInput = '';
  }

  async function advance() {
    failure = null;
    if (step === 'provider') {
      busy = true;
      try {
        await persistKeys();
      } catch (caught) {
        failure = toAppError(caught);
        return;
      } finally {
        busy = false;
      }
    }

    step = nextStep(step);
  }

  async function finish() {
    busy = true;
    failure = null;
    try {
      await settings.save({ ...draft, onboarded: true });
      setLocale(draft.locale);
      onDone();
    } catch (caught) {
      failure = toAppError(caught);
    } finally {
      busy = false;
    }
  }
</script>

<div class="backdrop">
  <div class="card glass" role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
    <p class="progress">
      {tf('onboarding.step_of', { current: index + 1, total: ONBOARDING_STEPS.length })}
    </p>

    {#if step === 'welcome'}
      <h2 id="onboarding-title">{t('onboarding.welcome_title')}</h2>
      <p>{t('onboarding.welcome_body')}</p>
      <p class="note">{t('onboarding.welcome_note')}</p>
      <label class="inline">
        <span>{t('settings.locale_label')}</span>
        <select bind:value={draft.locale} onchange={() => setLocale(draft.locale)}>
          <option value="cs">Čeština</option>
          <option value="en">English</option>
        </select>
      </label>
    {:else if step === 'privacy'}
      <h2 id="onboarding-title">{t('onboarding.privacy_title')}</h2>
      <ul>
        <li>{t('onboarding.privacy_local')}</li>
        <li>{t('onboarding.privacy_llm')}</li>
        <li>{t('onboarding.privacy_web')}</li>
        <li>{t('onboarding.privacy_keys')}</li>
        <li>{t('onboarding.privacy_confirm')}</li>
      </ul>
    {:else if step === 'provider'}
      <h2 id="onboarding-title">{t('onboarding.provider_title')}</h2>
      <p>{t('onboarding.provider_body')}</p>
      <label>
        <span>{t('settings.provider_label')}</span>
        <select bind:value={draft.provider}>
          <option value="cli">{t('settings.provider_cli')}</option>
          <option value="anthropic">{t('settings.provider_anthropic')}</option>
        </select>
      </label>
      {#if draft.provider === 'cli'}
        <label>
          <span>{t('settings.cli_preset_label')}</span>
          <select bind:value={cliPreset} onchange={applyCliPreset}>
            {#each CLI_PRESETS as preset (preset.id)}
              <option value={preset.id}>{t(`settings.cli_preset_${preset.id}`)}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>{t('settings.cli_command_label')}</span>
          <input
            type="text"
            bind:value={draft.cli_command}
            oninput={() => (cliPreset = commandToCliPreset(draft.cli_command))}
            placeholder={t('settings.cli_command_placeholder')}
            autocomplete="off"
            spellcheck="false"
          />
        </label>
      {:else}
        <label>
          <span>{t('settings.anthropic_model_label')}</span>
          <input type="text" bind:value={draft.anthropic_model} autocomplete="off" />
        </label>
        <label>
          <span>{t('settings.anthropic_key_label')}</span>
          <input
            type="password"
            bind:value={anthropicInput}
            placeholder={t('settings.anthropic_key_placeholder')}
            autocomplete="off"
          />
        </label>
      {/if}
      <label>
        <span>{t('onboarding.brave_optional')}</span>
        <input
          type="password"
          bind:value={braveInput}
          placeholder={t('settings.brave_key_placeholder')}
          autocomplete="off"
        />
      </label>
    {:else}
      <h2 id="onboarding-title">{t('onboarding.ready_title')}</h2>
      <p>{tf('onboarding.ready_hotkey', { hotkey: hotkeyLabel })}</p>
      <p class="note">{t('onboarding.ready_body')}</p>
    {/if}

    {#if failure}
      <ErrorState error={failure} onDismiss={() => (failure = null)} />
    {/if}

    <div class="actions">
      <button type="button" onclick={finish} disabled={busy}>{t('onboarding.skip')}</button>
      <div class="spacer"></div>
      <button type="button" onclick={() => (step = prevStep(step))} disabled={index === 0 || busy}>
        {t('onboarding.back')}
      </button>
      {#if isLast}
        <button type="button" class="primary" onclick={finish} disabled={busy}>
          {t('onboarding.finish')}
        </button>
      {:else}
        <button type="button" class="primary" onclick={advance} disabled={!advanceAllowed || busy}>
          {t('onboarding.next')}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: rgba(9, 9, 14, 0.5);
  }
  .card {
    display: grid;
    gap: var(--space-3);
    width: min(560px, 100%);
    max-height: 88vh;
    overflow-y: auto;
    padding: var(--space-6);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }
  .progress {
    margin: 0;
    color: var(--text-subtle);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  h2 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0;
    color: var(--text);
    font-size: 14px;
    line-height: 1.55;
  }
  .note {
    color: var(--text-muted);
    font-size: 13px;
  }
  ul {
    margin: 0;
    padding-left: var(--space-5);
    font-size: 14px;
    line-height: 1.6;
  }
  li + li {
    margin-top: var(--space-2);
  }
  label {
    display: grid;
    gap: var(--space-2);
  }
  label.inline {
    grid-template-columns: auto minmax(0, 200px);
    align-items: center;
  }
  label span {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
  .spacer {
    flex: 1;
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
