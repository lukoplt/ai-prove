export const DEFAULT_HOTKEY = 'CommandOrControl+Shift+D';

export type PlatformKind = 'mac' | 'other';

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'AltGraph', 'OS']);

/**
 * `event.code` → accelerator token, for keys whose `code` name is not already
 * the token Tauri expects. Everything else is derived structurally.
 */
const NAMED_CODES: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Insert: 'Insert',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Minus: 'Minus',
  Equal: 'Equal',
  Comma: 'Comma',
  Period: 'Period',
  Slash: 'Slash',
  Backslash: 'Backslash',
  Semicolon: 'Semicolon',
  Quote: 'Quote',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  Backquote: 'Backquote',
};

const FUNCTION_KEY = /^F([1-9]|1[0-9]|2[0-4])$/;

export function isModifierOnly(event: KeyboardEvent): boolean {
  return MODIFIER_KEYS.has(event.key);
}

function mainKey(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (FUNCTION_KEY.test(code)) return code;
  if (/^Numpad[0-9]$/.test(code)) return code;
  return NAMED_CODES[code] ?? null;
}

/**
 * Converts a keydown into a Tauri global-shortcut accelerator, or `null` when
 * the combination is not usable as a global hotkey. Requires at least one
 * modifier except for function keys, which are usable standalone.
 */
export function acceleratorFromEvent(event: KeyboardEvent): string | null {
  if (isModifierOnly(event)) return null;

  const key = mainKey(event.code);
  if (!key) return null;

  const parts: string[] = [];
  // macOS reports Command as metaKey, Windows/Linux report Control as ctrlKey.
  // `CommandOrControl` is the portable token Tauri resolves per platform.
  if (event.metaKey || event.ctrlKey) parts.push('CommandOrControl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');

  if (parts.length === 0 && !FUNCTION_KEY.test(key)) return null;

  parts.push(key);
  return parts.join('+');
}

const MAC_GLYPHS: Record<string, string> = {
  CommandOrControl: '⌘',
  Command: '⌘',
  Control: '⌃',
  Alt: '⌥',
  Option: '⌥',
  Shift: '⇧',
};

const OTHER_NAMES: Record<string, string> = {
  CommandOrControl: 'Ctrl',
  Command: 'Ctrl',
  Control: 'Ctrl',
  Alt: 'Alt',
  Option: 'Alt',
  Shift: 'Shift',
};

/** Human-readable rendering of an accelerator for the given platform. */
export function formatAccelerator(accelerator: string, platform: PlatformKind): string {
  const tokens = accelerator.split('+').filter(Boolean);
  if (tokens.length === 0) return '';

  if (platform === 'mac') {
    return tokens.map((token) => MAC_GLYPHS[token] ?? token).join('');
  }

  return tokens.map((token) => OTHER_NAMES[token] ?? token).join('+');
}

export function platformKind(): PlatformKind {
  if (typeof navigator === 'undefined') return 'other';
  return /Mac|iPhone|iPad/i.test(navigator.userAgent ?? '') ? 'mac' : 'other';
}
