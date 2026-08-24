import { describe, expect, it } from 'vitest';
import cs from './cs.json';
import en from './en.json';

/** Every leaf path in a nested bundle, e.g. `settings.locale_label`. */
function leafPaths(node: unknown, prefix = ''): string[] {
  if (typeof node !== 'object' || node === null) return [prefix];

  return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
    leafPaths(value, prefix ? `${prefix}.${key}` : key),
  );
}

function valueAt(bundle: unknown, path: string): unknown {
  return path
    .split('.')
    .reduce<unknown>((node, part) => (node as Record<string, unknown> | undefined)?.[part], bundle);
}

function placeholders(bundle: unknown, path: string): string[] {
  const value = valueAt(bundle, path);
  if (typeof value !== 'string') return [];
  return [...value.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
}

const csKeys = leafPaths(cs).sort();
const enKeys = leafPaths(en).sort();

describe('i18n bundles', () => {
  it('have identical key sets', () => {
    expect(csKeys.filter((key) => !enKeys.includes(key))).toEqual([]);
    expect(enKeys.filter((key) => !csKeys.includes(key))).toEqual([]);
  });

  it('have no empty strings', () => {
    for (const [locale, bundle] of [
      ['cs', cs],
      ['en', en],
    ] as const) {
      const empties = leafPaths(bundle).filter((path) => {
        const value = valueAt(bundle, path);
        return typeof value === 'string' && value.trim().length === 0;
      });
      expect(empties, `${locale} has empty values`).toEqual([]);
    }
  });

  it('use the same interpolation placeholders in both locales', () => {
    for (const key of csKeys) {
      expect(placeholders(cs, key), `placeholder mismatch for ${key}`).toEqual(
        placeholders(en, key),
      );
    }
  });
});
