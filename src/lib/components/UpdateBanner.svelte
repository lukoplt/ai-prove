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
  <aside class="banner" role="status">
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
    gap: 12px;
    margin: 0 0 14px;
    padding: 10px 14px;
    border-radius: 8px;
    border: 1px solid #bfdbfe;
    background: #eff6ff;
    color: #1e3a8a;
    font-size: 14px;
  }

  .msg {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .cur {
    color: #475569;
    font-size: 12px;
  }

  .actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .primary {
    background: #2563eb;
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
  }

  .primary:hover {
    background: #1d4ed8;
  }

  @media (max-width: 600px) {
    .banner {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
