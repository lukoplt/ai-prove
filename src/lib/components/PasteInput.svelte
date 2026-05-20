<script lang="ts">
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { t } from '$lib/stores/i18n.svelte';

  let {
    value = $bindable(''),
    onAnalyze = () => {},
  }: { value?: string; onAnalyze?: (text: string) => void } = $props();

  let dragging = $state(false);

  async function paste() {
    const text = await readText();
    if (text) value = text;
  }

  function clear() {
    value = '';
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault();
    dragging = true;
  }

  function onDragLeave() {
    dragging = false;
  }

  function onDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    const text = event.dataTransfer?.getData('text/plain') ?? '';
    if (text) value = text;
  }

  function analyze() {
    const trimmed = value.trim();
    if (trimmed) onAnalyze(trimmed);
  }
</script>

<div
  class="wrap"
  class:dragging
  role="group"
  aria-label={t('input.placeholder')}
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
>
  <textarea bind:value placeholder={t('input.placeholder')} rows={12} spellcheck="false"></textarea>
  <div class="bar">
    <button type="button" onclick={paste}>{t('input.paste_from_clipboard')}</button>
    <button type="button" onclick={clear} disabled={!value}>{t('input.clear')}</button>
    <div class="spacer"></div>
    <button type="button" class="primary" onclick={analyze} disabled={!value.trim()}>
      {t('input.analyze')}
    </button>
  </div>
</div>

<style>
  .wrap {
    display: grid;
    gap: 10px;
    padding: 4px;
    border: 2px dashed transparent;
    border-radius: 8px;
    transition: border-color 120ms ease;
  }

  .wrap.dragging {
    border-color: #2563eb;
  }

  textarea {
    width: 100%;
    min-height: 300px;
    padding: 14px;
    resize: vertical;
  }

  .bar {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .spacer {
    flex: 1;
  }

  .primary {
    border-color: #18181b;
    background: #18181b;
    color: #ffffff;
  }
</style>
