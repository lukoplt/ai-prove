<script lang="ts">
  import { t, tf } from '$lib/stores/i18n.svelte';
  import type { Claim } from '$lib/types';
  import { aggregateVerdict } from '$lib/verdict';

  let { claims }: { claims: Claim[] } = $props();

  const verdict = $derived(aggregateVerdict(claims));

  const ICONS = {
    mostly_verified: '✓',
    disputed: '✕',
    no_consensus: '~',
    unverified: '?',
  } as const;
</script>

<div class="banner glass kind-{verdict.kind}">
  <span class="icon" aria-hidden="true">{ICONS[verdict.kind]}</span>
  <div class="text">
    <strong class="headline">{t(`verdict.${verdict.kind}`)}</strong>
    {#if verdict.total > 0}
      <span class="count"
        >{tf('verdict.count', { verified: verdict.verified, total: verdict.total })}</span
      >
    {/if}
  </div>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 999px;
    font-size: 16px;
    font-weight: 800;
    flex: 0 0 auto;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .headline {
    font-size: 15px;
  }
  .count {
    color: var(--text-muted);
    font-size: 13px;
  }

  .kind-mostly_verified {
    border-color: var(--ok);
  }
  .kind-mostly_verified .icon {
    background: var(--ok-soft);
    color: var(--ok);
  }
  .kind-disputed {
    border-color: var(--bad);
  }
  .kind-disputed .icon {
    background: var(--bad-soft);
    color: var(--bad);
  }
  .kind-no_consensus {
    border-color: var(--warn);
  }
  .kind-no_consensus .icon {
    background: var(--warn-soft);
    color: var(--warn);
  }
  .kind-unverified .icon {
    background: var(--neutral-soft);
    color: var(--neutral);
  }
</style>
