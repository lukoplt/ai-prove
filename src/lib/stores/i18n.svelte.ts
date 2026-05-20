import cs from '../i18n/cs.json';
import en from '../i18n/en.json';

export type Locale = 'cs' | 'en';

const bundles: Record<Locale, unknown> = { cs, en };

let currentLocale = $state<Locale>('cs');

export function setLocale(locale: Locale): void {
  currentLocale = locale;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function t(key: string): string {
  const parts = key.split('.');
  let node: unknown = bundles[currentLocale];

  for (const part of parts) {
    if (typeof node !== 'object' || node === null) return key;
    node = (node as Record<string, unknown>)[part];
    if (node === undefined) return key;
  }

  return typeof node === 'string' ? node : key;
}
