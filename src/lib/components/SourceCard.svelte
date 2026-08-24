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

<article class="card glass stance-{source.stance}">
  <header>
    <TierBadge tier={source.tier} />
    <span class="stance-pill">{t(`stance.${source.stance}`)}</span>
    <span class="host">{host(source.url)}</span>
  </header>
  <h4>{source.title}</h4>
  {#if source.snippet}
    <p class="snippet">"{source.snippet}"</p>
  {/if}
  <button type="button" onclick={open} aria-label={`${t('source.open')}: ${source.title}`}>
    {t('source.open')}
  </button>
</article>

<style>
  .card {
    margin-bottom: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-md);
  }

  .stance-supports {
    border-left: 3px solid var(--ok);
  }

  .stance-contradicts {
    border-left: 3px solid var(--bad);
  }

  .stance-mentions {
    border-left: 3px solid var(--neutral);
  }

  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .stance-pill {
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--neutral-soft);
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 600;
  }

  .host {
    min-width: 0;
    margin-left: auto;
    color: var(--text-subtle);
    font-size: 11px;
    overflow-wrap: anywhere;
  }

  h4 {
    margin: 0 0 var(--space-1);
    color: var(--text);
    font-size: 13px;
    line-height: 1.3;
  }

  .snippet {
    margin: 0 0 var(--space-2);
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  button {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
  }
  button:hover {
    color: var(--accent-hover);
    border-color: transparent;
  }
</style>
