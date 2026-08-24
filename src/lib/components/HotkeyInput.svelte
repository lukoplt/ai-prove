<script lang="ts">
  import {
    acceleratorFromEvent,
    DEFAULT_HOTKEY,
    formatAccelerator,
    isModifierOnly,
    platformKind,
  } from '$lib/hotkey';
  import { t } from '$lib/stores/i18n.svelte';

  let { value = $bindable(DEFAULT_HOTKEY) }: { value?: string } = $props();

  let recording = $state(false);
  let rejected = $state(false);

  const platform = platformKind();
  const label = $derived(formatAccelerator(value, platform));

  function onKeydown(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();
    event.stopPropagation();

    if (event.key === 'Escape') {
      recording = false;
      rejected = false;
      return;
    }

    if (isModifierOnly(event)) return;

    const accelerator = acceleratorFromEvent(event);
    if (!accelerator) {
      rejected = true;
      return;
    }

    value = accelerator;
    rejected = false;
    recording = false;
  }
</script>

<div class="hk">
  <button
    type="button"
    class="capture"
    class:recording
    aria-live="polite"
    onclick={() => {
      recording = !recording;
      rejected = false;
    }}
    onkeydown={onKeydown}
    onblur={() => (recording = false)}
  >
    {recording ? t('hotkey.recording') : label}
  </button>
  <button
    type="button"
    onclick={() => {
      value = DEFAULT_HOTKEY;
      rejected = false;
    }}
  >
    {t('hotkey.reset')}
  </button>
</div>
<small class="hint" role={rejected ? 'alert' : undefined}>
  {rejected ? t('hotkey.rejected') : t('hotkey.hint')}
</small>

<style>
  .hk {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .capture {
    min-width: 160px;
    font-variant-numeric: tabular-nums;
  }
  .capture.recording {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .hint {
    display: block;
    margin-top: var(--space-1);
    color: var(--text-subtle);
    font-size: 12px;
    line-height: 1.45;
  }
</style>
