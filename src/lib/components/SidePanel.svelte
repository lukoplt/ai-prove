<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import type { Claim } from '$lib/types';

  let { claim }: { claim: Claim | null } = $props();

  function kindLabel(kind: Claim['kind']): string {
    return t(`sidepanel.kind_${kind}`);
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
      <p class="pending">{t('sidepanel.sources_pending')}</p>
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

  .pending {
    color: #a1a1aa;
    font-style: italic;
  }
</style>
