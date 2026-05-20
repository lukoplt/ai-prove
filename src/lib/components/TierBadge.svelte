<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import type { SourceTier } from '$lib/types';

  let { tier }: { tier: SourceTier } = $props();
</script>

<span class="badge tier-{tier}">{t(`tier.${tier}`)}</span>

<style>
  .badge {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .tier-a {
    background: #dbeafe;
    color: #1e3a8a;
  }

  .tier-b {
    background: #e0e7ff;
    color: #3730a3;
  }

  .tier-c {
    background: #f3f4f6;
    color: #374151;
  }

  .tier-d {
    background: #fee2e2;
    color: #7f1d1d;
  }
</style>
