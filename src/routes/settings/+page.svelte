<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { clearApiKey, setApiKey } from '$lib/api';
  import { setLocale } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { t } from '$lib/stores/i18n.svelte';
  import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type ApiAccount, type Settings } from '$lib/types';

  let local: Settings = $state({ ...settings.current });
  let anthropicInput = $state('');
  let braveInput = $state('');
  let saving = $state(false);
  let message = $state<string | null>(null);

  async function persistSettings() {
    saving = true;
    message = null;

    try {
      await settings.save(local);
      setLocale(local.locale);
      message = t('settings.saved');
    } catch (error) {
      message = String(error);
    } finally {
      saving = false;
    }
  }

  async function saveKey(account: ApiAccount, secret: string) {
    const value = secret.trim();
    if (!value) {
      message = t('errors.key_empty');
      return;
    }

    await setApiKey(account, value);
    if (account === ACCOUNT_ANTHROPIC) anthropicInput = '';
    if (account === ACCOUNT_BRAVE) braveInput = '';
    await settings.refreshKeyState();
    message = t('settings.key_present');
  }

  async function removeKey(account: ApiAccount) {
    await clearApiKey(account);
    await settings.refreshKeyState();
  }
</script>

<main class="page">
  <header>
    <button type="button" onclick={() => goto(resolve('/'))}>{t('settings.back')}</button>
    <h1>{t('settings.title')}</h1>
  </header>

  {#if local.provider === 'anthropic'}
    <section>
      <h2>{t('settings.anthropic_key_label')}</h2>
      <p class="status">
        {settings.anthropicPresent ? t('settings.key_present') : t('settings.key_missing')}
      </p>
      <div class="row">
        <input
          type="password"
          bind:value={anthropicInput}
          placeholder={t('settings.anthropic_key_placeholder')}
          autocomplete="off"
        />
        <button type="button" onclick={() => saveKey(ACCOUNT_ANTHROPIC, anthropicInput)}>
          {t('settings.save_key')}
        </button>
        <button
          type="button"
          onclick={() => removeKey(ACCOUNT_ANTHROPIC)}
          disabled={!settings.anthropicPresent}
        >
          {t('settings.clear_key')}
        </button>
      </div>
    </section>
  {/if}

  <section>
    <h2>{t('settings.brave_key_label')}</h2>
    <p class="status">
      {settings.bravePresent ? t('settings.key_present') : t('settings.key_missing')}
    </p>
    <div class="row">
      <input
        type="password"
        bind:value={braveInput}
        placeholder={t('settings.brave_key_placeholder')}
        autocomplete="off"
      />
      <button type="button" onclick={() => saveKey(ACCOUNT_BRAVE, braveInput)}>
        {t('settings.save_key')}
      </button>
      <button
        type="button"
        onclick={() => removeKey(ACCOUNT_BRAVE)}
        disabled={!settings.bravePresent}
      >
        {t('settings.clear_key')}
      </button>
    </div>
  </section>

  <section class="settings-grid">
    <h2>{t('settings.provider_section')}</h2>
    <label>
      <span>{t('settings.provider_label')}</span>
      <select bind:value={local.provider}>
        <option value="cli">{t('settings.provider_cli')}</option>
        <option value="anthropic">{t('settings.provider_anthropic')}</option>
      </select>
    </label>
    {#if local.provider === 'cli'}
      <label>
        <span>{t('settings.cli_command_label')}</span>
        <input
          type="text"
          bind:value={local.cli_command}
          placeholder={t('settings.cli_command_placeholder')}
          autocomplete="off"
          spellcheck="false"
        />
        <small class="hint">{t('settings.cli_command_hint')}</small>
      </label>
    {:else}
      <label>
        <span>{t('settings.anthropic_model_label')}</span>
        <input
          type="text"
          bind:value={local.anthropic_model}
          placeholder={t('settings.anthropic_model_placeholder')}
          autocomplete="off"
          spellcheck="false"
        />
      </label>
    {/if}
  </section>

  <section class="settings-grid">
    <label>
      <span>{t('settings.locale_label')}</span>
      <select bind:value={local.locale}>
        <option value="cs">Čeština</option>
        <option value="en">English</option>
      </select>
    </label>
    <label>
      <span>{t('settings.hotkey_label')}</span>
      <input type="text" bind:value={local.hotkey} />
    </label>
    <label>
      <span>{t('settings.cache_ttl_label')}</span>
      <input type="number" min="1" max="90" bind:value={local.cache_ttl_days} />
    </label>
  </section>

  <footer>
    <button type="button" class="primary" onclick={persistSettings} disabled={saving}>
      {t('settings.save_settings')}
    </button>
    {#if message}
      <span class="msg">{message}</span>
    {/if}
  </footer>
</main>

<style>
  .page {
    max-width: 760px;
    margin: 0 auto;
    padding: 28px;
    display: grid;
    gap: 16px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h1 {
    margin: 0;
    font-size: 26px;
  }

  section {
    padding: 16px;
    border: 1px solid #e4e4e7;
    border-radius: 8px;
    background: #ffffff;
  }

  h2 {
    margin: 0 0 8px;
    color: #71717a;
    font-size: 13px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .row {
    display: flex;
    gap: 8px;
  }

  .row input {
    min-width: 0;
    flex: 1;
  }

  .status {
    margin: 0 0 10px;
    color: #71717a;
    font-size: 13px;
  }

  .settings-grid {
    display: grid;
    gap: 12px;
  }

  label {
    display: grid;
    gap: 6px;
  }

  label span {
    color: #52525b;
    font-size: 13px;
    font-weight: 600;
  }

  footer {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .primary {
    border-color: #18181b;
    background: #18181b;
    color: #ffffff;
  }

  .msg {
    color: #166534;
    font-size: 13px;
  }

  @media (max-width: 680px) {
    .row,
    footer {
      align-items: stretch;
      flex-direction: column;
    }
  }
</style>
