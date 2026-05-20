<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { listen } from '@tauri-apps/api/event';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { onMount } from 'svelte';
  import PasteInput from '$lib/components/PasteInput.svelte';
  import { t } from '$lib/stores/i18n.svelte';

  let inputText = $state('');

  function handleAnalyze(text: string) {
    console.log('[M0] would analyze:', text.slice(0, 80));
    inputText = text;
  }

  onMount(() => {
    const unlisten = listen('capture-trigger', async () => {
      const clipboard = await readText();
      if (clipboard) inputText = clipboard;
    });

    return () => {
      unlisten.then((unsubscribe) => unsubscribe());
    };
  });
</script>

<main class="page">
  <header>
    <div>
      <h1>{t('app.title')}</h1>
      <p>{t('app.tagline')}</p>
    </div>
    <nav>
      <button type="button" onclick={() => goto(resolve('/settings'))}
        >{t('common.settings')}</button
      >
    </nav>
  </header>

  <PasteInput bind:value={inputText} onAnalyze={handleAnalyze} />
</main>

<style>
  .page {
    max-width: 960px;
    margin: 0 auto;
    padding: 28px;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 18px;
  }

  h1 {
    margin: 0 0 5px;
    font-size: 28px;
    line-height: 1.1;
  }

  p {
    margin: 0;
    color: #71717a;
  }
</style>
