<script lang="ts">
  import { readClipboardText } from '$lib/api';
  import { t } from '$lib/stores/i18n.svelte';
  import type { AnalyzeInput } from '$lib/types';

  let {
    question = $bindable(''),
    answer = $bindable(''),
    onAnalyze = () => {},
  }: {
    question?: string;
    answer?: string;
    onAnalyze?: (input: AnalyzeInput) => void;
  } = $props();

  let dragging = $state(false);

  async function paste() {
    const text = await readClipboardText();
    if (text) answer = text;
  }

  function clear() {
    question = '';
    answer = '';
  }

  function clearAnswer() {
    answer = '';
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
    if (text) answer = text;
  }

  function analyze() {
    const trimmedAnswer = answer.trim();
    if (trimmedAnswer) {
      onAnalyze({
        question: question.trim(),
        answer: trimmedAnswer,
      });
    }
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
  <label>
    <span>{t('input.question_label')}</span>
    <textarea
      bind:value={question}
      placeholder={t('input.question_placeholder')}
      rows={4}
      spellcheck="false"
    ></textarea>
  </label>
  <label>
    <span>{t('input.answer_label')}</span>
    <textarea
      bind:value={answer}
      placeholder={t('input.answer_placeholder')}
      rows={12}
      spellcheck="false"
    ></textarea>
  </label>
  <div class="bar">
    <button type="button" onclick={paste}>{t('input.paste_from_clipboard')}</button>
    <button type="button" onclick={clearAnswer} disabled={!answer}>
      {t('input.clear_answer')}
    </button>
    <button type="button" onclick={clear} disabled={!question && !answer}>
      {t('input.clear')}
    </button>
    <div class="spacer"></div>
    <button type="button" class="primary" onclick={analyze} disabled={!answer.trim()}>
      {t('input.analyze')}
    </button>
  </div>
</div>

<style>
  .wrap {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-1);
    border: 2px solid transparent;
    border-radius: var(--radius-lg);
    transition:
      border-color var(--dur) var(--ease),
      box-shadow var(--dur) var(--ease);
  }

  label {
    display: grid;
    gap: var(--space-2);
  }

  span {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 700;
  }

  .wrap.dragging {
    border-color: var(--accent);
    box-shadow: 0 0 0 4px var(--accent-soft);
  }

  textarea {
    width: 100%;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    resize: vertical;
  }

  label:last-of-type textarea {
    min-height: 200px;
  }

  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: center;
  }

  .spacer {
    flex: 1;
  }

  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
