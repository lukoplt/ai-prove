import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { contrastRatio, parseHex, relativeLuminance } from './contrast';

const tokens = readFileSync(resolve(process.cwd(), 'src/lib/styles/tokens.css'), 'utf8');

/** Reads a `--name: #rrggbb;` declaration from a scope block of tokens.css. */
function token(name: string, scope: 'light' | 'dark'): string {
  const darkAt = tokens.indexOf("[data-theme='dark'] {");
  const block = scope === 'light' ? tokens.slice(0, darkAt) : tokens.slice(darkAt);
  const match = new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{3,8})`).exec(block);
  if (!match) throw new Error(`token --${name} not found in ${scope} scope`);
  return match[1];
}

describe('contrast helpers', () => {
  it('parses 6- and 3-digit hex', () => {
    expect(parseHex('#ffffff')).toEqual([255, 255, 255]);
    expect(parseHex('#fff')).toEqual([255, 255, 255]);
    expect(parseHex('nope')).toBeNull();
  });

  it('computes the reference luminances', () => {
    expect(relativeLuminance([255, 255, 255])).toBeCloseTo(1, 5);
    expect(relativeLuminance([0, 0, 0])).toBeCloseTo(0, 5);
  });

  it('computes the reference ratio', () => {
    expect(contrastRatio('#ffffff', '#000000')).toBeCloseTo(21, 2);
  });
});

describe('token contrast meets WCAG AA', () => {
  const pairs: Array<[string, string, 'light' | 'dark']> = [
    ['text', 'bg', 'light'],
    ['text-muted', 'bg', 'light'],
    ['text-subtle', 'bg', 'light'],
    ['ok', 'bg', 'light'],
    ['bad', 'bg', 'light'],
    ['warn', 'bg', 'light'],
    ['accent', 'bg', 'light'],
    ['tier-a-fg', 'tier-a-bg', 'light'],
    ['tier-b-fg', 'tier-b-bg', 'light'],
    ['tier-c-fg', 'tier-c-bg', 'light'],
    ['tier-d-fg', 'tier-d-bg', 'light'],
    ['text', 'bg', 'dark'],
    ['text-muted', 'bg', 'dark'],
    ['text-subtle', 'bg', 'dark'],
    ['ok', 'bg', 'dark'],
    ['bad', 'bg', 'dark'],
    ['warn', 'bg', 'dark'],
    ['accent', 'bg', 'dark'],
  ];

  for (const [fg, bg, scope] of pairs) {
    it(`--${fg} on --${bg} (${scope}) is at least 4.5:1`, () => {
      expect(contrastRatio(token(fg, scope), token(bg, scope))).toBeGreaterThanOrEqual(4.5);
    });
  }
});
