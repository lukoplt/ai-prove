<script lang="ts">
  import { onMount } from 'svelte';
  import { checkLatestRelease, openInBrowser } from '$lib/api';
  import { settings } from '$lib/stores/settings.svelte';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import type { LatestRelease } from '$lib/types';

  let release = $state<LatestRelease | null>(null);
  let dismissed = $state(false);

  onMount(async () => {
    if (!settings.current.check_updates_on_launch) return;
    const result = await checkLatestRelease();
    if (result?.isNewer) release = result;
  });

  function openDownload() {
    if (release) void openInBrowser(release.htmlUrl);
    dismissed = true;
  }
</script>

{#if release && !dismissed}
  <aside class="banner glass" role="status">
    <div class="msg">
      <strong>{tf('updater.available', { version: release.latestVersion })}</strong>
      <span class="cur">{tf('updater.current', { version: release.currentVersion })}</span>
    </div>
    <div class="actions">
      <button type="button" class="primary" onclick={openDownload}>
        {t('updater.open_download')}
      </button>
      <button type="button" onclick={() => (dismissed = true)}>
        {t('updater.later')}
      </button>
    </div>
  </aside>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin: 0 0 var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border-color: var(--accent);
    color: var(--text);
    font-size: 14px;
  }

  .msg {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .cur {
    color: var(--text-muted);
    font-size: 12px;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .primary {
    background: var(--accent);
    color: var(--accent-contrast);
    border: 1px solid var(--accent);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  @media (max-width: 600px) {
    .banner {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
