export type Rgb = [number, number, number];

/** Parses `#rgb` / `#rrggbb` / `#rrggbbaa` (alpha ignored). */
export function parseHex(value: string): Rgb | null {
  const match = /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(value.trim());
  if (!match) return null;

  const digits = match[1];
  const expanded =
    digits.length === 3
      ? digits
          .split('')
          .map((digit) => digit + digit)
          .join('')
      : digits.slice(0, 6);

  return [
    Number.parseInt(expanded.slice(0, 2), 16),
    Number.parseInt(expanded.slice(2, 4), 16),
    Number.parseInt(expanded.slice(4, 6), 16),
  ];
}

/** WCAG 2.1 relative luminance. */
export function relativeLuminance(rgb: Rgb): number {
  const [r, g, b] = rgb.map((channel) => {
    const normalized = channel / 255;
    return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
  });

  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG 2.1 contrast ratio between two hex colours. Returns 0 on bad input. */
export function contrastRatio(a: string, b: string): number {
  const first = parseHex(a);
  const second = parseHex(b);
  if (!first || !second) return 0;

  const lighter = Math.max(relativeLuminance(first), relativeLuminance(second));
  const darker = Math.min(relativeLuminance(first), relativeLuminance(second));
  return (lighter + 0.05) / (darker + 0.05);
}
