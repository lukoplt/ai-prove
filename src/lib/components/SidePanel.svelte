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

<aside class="sp">
  {#if !claim}
    <p class="empty">{t('sidepanel.empty')}</p>
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
    padding: 16px;
    border: 1px solid #e4e4e7;
    border-radius: 8px;
    background: #ffffff;
  }

  .empty {
    margin: 0;
    color: #71717a;
    font-size: 14px;
  }

  header {
    margin-bottom: 8px;
  }

  .badge {
    display: inline-block;
    padding: 3px 8px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .kind-fact {
    background: rgba(34, 197, 94, 0.22);
    color: #14532d;
  }

  .kind-inference {
    background: rgba(234, 179, 8, 0.25);
    color: #713f12;
  }

  .kind-opinion {
    background: rgba(249, 115, 22, 0.25);
    color: #7c2d12;
  }

  .kind-contradiction {
    background: rgba(239, 68, 68, 0.25);
    color: #7f1d1d;
  }

  .quote {
    margin: 0 0 12px;
    padding: 8px 12px;
    border-left: 3px solid #d4d4d8;
    background: #fafafa;
    font-size: 14px;
  }

  section h3 {
    margin: 0 0 4px;
    color: #71717a;
    font-size: 12px;
    text-transform: uppercase;
  }

  section p {
    margin: 0 0 12px;
    font-size: 14px;
  }

  .muted {
    color: #a1a1aa;
    font-style: italic;
  }

  .verdict {
    margin: 0 0 10px;
    padding: 8px 10px;
    border-radius: 6px;
    font-size: 13px;
    line-height: 1.35;
  }

  .status-supported {
    background: rgba(34, 197, 94, 0.15);
    color: #14532d;
  }

  .status-contradicted {
    background: rgba(239, 68, 68, 0.15);
    color: #7f1d1d;
  }

  .status-no_consensus {
    background: rgba(234, 179, 8, 0.18);
    color: #713f12;
  }

  .status-not_found,
  .status-not_verified {
    background: #f3f4f6;
    color: #4b5563;
  }
</style>
