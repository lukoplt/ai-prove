<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import type { SourceTier } from '$lib/types';

  let { tier }: { tier: SourceTier } = $props();
</script>

<span class="badge tier-{tier}">{t(`tier.${tier}`)}</span>

<style>
  .badge {
    display: inline-block;
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .tier-a {
    background: var(--tier-a-bg);
    color: var(--tier-a-fg);
  }

  .tier-b {
    background: var(--tier-b-bg);
    color: var(--tier-b-fg);
  }

  .tier-c {
    background: var(--tier-c-bg);
    color: var(--tier-c-fg);
  }

  .tier-d {
    background: var(--tier-d-bg);
    color: var(--tier-d-fg);
  }
</style>
