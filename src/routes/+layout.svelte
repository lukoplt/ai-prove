<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css';
  import Onboarding from '$lib/components/Onboarding.svelte';
  import { setLocale } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { theme } from '$lib/stores/theme.svelte';

  let { children } = $props();

  let bootLabel = $state('Loading… / Spouštím…');
  let onboardingDone = $state(false);

  const showOnboarding = $derived(
    settings.loaded && !settings.current.onboarded && !onboardingDone,
  );

  onMount(async () => {
    const nav = typeof navigator !== 'undefined' ? navigator.language : '';
    bootLabel = nav?.toLowerCase().startsWith('cs') ? 'Spouštím…' : 'Loading…';
    await settings.load();
    setLocale(settings.current.locale);
    theme.init(settings.current.theme);
  });
</script>

{#if settings.loaded}
  <div class="app-mesh" aria-hidden="true"></div>
  {@render children()}
  {#if showOnboarding}
    <Onboarding onDone={() => (onboardingDone = true)} />
  {/if}
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
