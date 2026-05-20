<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css';
  import { setLocale } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  let { children } = $props();

  onMount(async () => {
    await settings.load();
    setLocale(settings.current.locale);
  });
</script>

{#if settings.loaded}
  {@render children()}
{:else}
  <div class="boot">Spouštím…</div>
{/if}

<style>
  .boot {
    display: grid;
    min-height: 100vh;
    place-items: center;
    color: #71717a;
  }
</style>
