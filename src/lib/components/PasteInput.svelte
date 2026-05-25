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
    gap: 10px;
    padding: 4px;
    border: 2px dashed transparent;
    border-radius: 8px;
    transition: border-color 120ms ease;
  }

  label {
    display: grid;
    gap: 6px;
  }

  span {
    color: #52525b;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0;
  }

  .wrap.dragging {
    border-color: #2563eb;
  }

  textarea {
    width: 100%;
    padding: 14px;
    resize: vertical;
  }

  label:last-of-type textarea {
    min-height: 200px;
  }

  .bar {
    display: flex;
    flex-wrap: wrap;
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
