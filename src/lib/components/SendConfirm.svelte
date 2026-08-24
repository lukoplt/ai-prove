<script lang="ts">
  import { describeSend } from '$lib/sendSummary';
  import { t, tf } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let {
    question,
    answer,
    dontAsk = $bindable(false),
  }: {
    question: string;
    answer: string;
    dontAsk?: boolean;
  } = $props();

  const lines = $derived(
    describeSend({
      settings: settings.current,
      bravePresent: settings.bravePresent,
      question,
      answer,
    }),
  );
</script>

<p class="intro">{t('send.intro')}</p>
<ul>
  {#each lines as line (line.key)}
    <li>{tf(line.key, line.vars)}</li>
  {/each}
</ul>
<label class="dont-ask">
  <input type="checkbox" bind:checked={dontAsk} />
  <span>{t('send.dont_ask')}</span>
</label>

<style>
  .intro {
    margin: 0 0 var(--space-2);
    color: var(--text-muted);
    font-size: 14px;
  }
  ul {
    margin: 0 0 var(--space-3);
    padding-left: var(--space-5);
    color: var(--text);
    font-size: 14px;
    line-height: 1.55;
  }
  li + li {
    margin-top: var(--space-1);
  }
  .dont-ask {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-muted);
    font-size: 13px;
  }
  .dont-ask input {
    width: auto;
    min-height: 0;
  }
</style>
