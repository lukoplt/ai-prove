<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css';
  import { setLocale } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let { children } = $props();

  let bootLabel = $state('Loading… / Spouštím…');

  onMount(async () => {
    const nav = typeof navigator !== 'undefined' ? navigator.language : '';
    bootLabel = nav?.toLowerCase().startsWith('cs') ? 'Spouštím…' : 'Loading…';
    await settings.load();
    setLocale(settings.current.locale);
  });
</script>

{#if settings.loaded}
  {@render children()}
{:else}
  <div class="boot">{bootLabel}</div>
{/if}

<style>
  .boot {
    display: grid;
    min-height: 100vh;
    place-items: center;
    color: #71717a;
  }
</style>
