<script lang="ts">
  import { openInBrowser } from '$lib/api';
  import { t } from '$lib/stores/i18n.svelte';
  import type { SourceHit } from '$lib/types';
  import TierBadge from './TierBadge.svelte';

  let { source }: { source: SourceHit } = $props();

  function host(url: string): string {
    try {
      return new URL(url).host.replace(/^www\./, '');
    } catch {
      return url;
    }
  }

  async function open(): Promise<void> {
    await openInBrowser(source.url);
  }
</script>

<article class="card stance-{source.stance}">
  <header>
    <TierBadge tier={source.tier} />
    <span class="stance-pill">{t(`stance.${source.stance}`)}</span>
    <span class="host">{host(source.url)}</span>
  </header>
  <h4>{source.title}</h4>
  {#if source.snippet}
    <p class="snippet">"{source.snippet}"</p>
  {/if}
  <button type="button" onclick={open}>{t('source.open')}</button>
</article>

<style>
  .card {
    margin-bottom: 8px;
    padding: 10px 12px;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    background: #ffffff;
  }

  .stance-supports {
    border-left: 3px solid #22c55e;
  }

  .stance-contradicts {
    border-left: 3px solid #ef4444;
  }

  .stance-mentions {
    border-left: 3px solid #9ca3af;
  }

  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
  }

  .stance-pill {
    padding: 2px 6px;
    border-radius: 4px;
    background: #f3f4f6;
    color: #374151;
    font-size: 11px;
    font-weight: 600;
  }

  .host {
    min-width: 0;
    margin-left: auto;
    color: #6b7280;
    font-size: 11px;
    overflow-wrap: anywhere;
  }

  h4 {
    margin: 0 0 4px;
    color: #111827;
    font-size: 13px;
    line-height: 1.25;
  }

  .snippet {
    margin: 0 0 7px;
    color: #4b5563;
    font-size: 12px;
    line-height: 1.35;
  }

  button {
    padding: 0;
    border: 0;
    background: none;
    color: #2563eb;
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
  }
</style>
