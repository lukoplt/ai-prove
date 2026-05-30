<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { theme } from '$lib/stores/theme.svelte';
  import type { ThemePref } from '$lib/types';

  const OPTIONS: { value: ThemePref; glyph: string }[] = [
    { value: 'auto', glyph: 'A' },
    { value: 'light', glyph: '☀' },
    { value: 'dark', glyph: '☾' },
  ];

  async function choose(value: ThemePref) {
    theme.set(value);
    await settings.save({ ...settings.current, theme: value });
  }
</script>

<div class="seg" role="group" aria-label={t('theme.label')}>
  {#each OPTIONS as opt (opt.value)}
    <button
      type="button"
      class="opt"
      class:active={theme.pref === opt.value}
      aria-pressed={theme.pref === opt.value}
      title={t(`theme.${opt.value}`)}
      onclick={() => choose(opt.value)}
    >
      <span aria-hidden="true">{opt.glyph}</span>
      <span class="sr">{t(`theme.${opt.value}`)}</span>
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    padding: 2px;
    border-radius: 999px;
    background: var(--surface-glass);
    border: 1px solid var(--surface-glass-border);
  }
  .opt {
    padding: 4px 9px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--text-muted);
    font-size: 13px;
    line-height: 1;
  }
  .opt:hover {
    border-color: transparent;
    color: var(--text);
  }
  .opt.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
