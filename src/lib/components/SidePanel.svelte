<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import type { Claim, VerificationStatus } from '$lib/types';
  import SourceCard from './SourceCard.svelte';

  let { claim }: { claim: Claim | null } = $props();

  function kindLabel(kind: Claim['kind']): string {
    return t(`sidepanel.kind_${kind}`);
  }

  function statusLabel(status: VerificationStatus): string {
    return t(`status.${status}`);
  }
</script>

<aside class="sp glass">
  {#if !claim}
    <div class="empty">
      <span class="empty-icon" aria-hidden="true">◎</span>
      <p>{t('sidepanel.empty')}</p>
    </div>
  {:else}
    <header>
      <span class="badge kind-{claim.kind}">{kindLabel(claim.kind)}</span>
    </header>
    <blockquote class="quote">"{claim.text}"</blockquote>
    <section>
      <h3>{t('sidepanel.why_label')}</h3>
      <p>{claim.reason}</p>
    </section>
    <section>
      <h3>{t('sidepanel.sources_label')}</h3>
      {#if claim.kind !== 'fact'}
        <p class="muted">{t('verification.skipped_kind')}</p>
      {:else if !claim.verification}
        <p class="muted">{t('verification.pending')}</p>
      {:else}
        <p class="verdict status-{claim.verification.status}">
          <strong>{statusLabel(claim.verification.status)}</strong>
          - {claim.verification.summary}
        </p>
        {#if claim.verification.sources.length === 0}
          <p class="muted">{t('verification.no_sources')}</p>
        {:else}
          {#each claim.verification.sources as source (source.url)}
            <SourceCard {source} />
          {/each}
        {/if}
      {/if}
    </section>
  {/if}
</aside>

<style>
  .sp {
    min-height: 320px;
    padding: var(--space-4);
    border-radius: var(--radius-lg);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6) var(--space-3);
    text-align: center;
  }
  .empty-icon {
    font-size: 28px;
    color: var(--text-subtle);
  }
  .empty p {
    margin: 0;
    color: var(--text-subtle);
    font-size: 14px;
  }

  header {
    margin-bottom: var(--space-2);
  }

  .badge {
    display: inline-block;
    padding: 3px var(--space-2);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .kind-fact {
    background: var(--ok-soft);
    color: var(--ok);
  }

  .kind-inference {
    background: var(--warn-soft);
    color: var(--warn);
  }

  .kind-opinion {
    background: var(--neutral-soft);
    color: var(--neutral);
  }

  .kind-contradiction {
    background: var(--bad-soft);
    color: var(--bad);
  }

  .quote {
    margin: 0 0 var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--surface-glass-border);
    background: var(--accent-soft);
    border-radius: var(--radius-sm);
    font-size: 14px;
  }

  section h3 {
    margin: 0 0 var(--space-1);
    color: var(--text-subtle);
    font-size: 12px;
    text-transform: uppercase;
  }

  section p {
    margin: 0 0 var(--space-3);
    font-size: 14px;
    color: var(--text);
  }

  .muted {
    color: var(--text-subtle);
    font-style: italic;
  }

  .verdict {
    margin: 0 0 var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: 13px;
    line-height: 1.4;
  }

  .status-supported {
    background: var(--ok-soft);
    color: var(--ok);
  }

  .status-contradicted {
    background: var(--bad-soft);
    color: var(--bad);
  }

  .status-no_consensus {
    background: var(--warn-soft);
    color: var(--warn);
  }

  .status-not_found,
  .status-not_verified {
    background: var(--neutral-soft);
    color: var(--text-muted);
  }
</style>
