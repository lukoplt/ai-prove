import type { Locale } from './stores/i18n.svelte';

const LOCALE_TAGS: Record<Locale, string> = { cs: 'cs-CZ', en: 'en-US' };

/** Absolute, locale-aware timestamp for a history row. */
export function formatHistoryDate(ms: number, locale: Locale): string {
  if (!Number.isFinite(ms)) return '';

  return new Intl.DateTimeFormat(LOCALE_TAGS[locale], {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(ms));
}
