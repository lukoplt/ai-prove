import { describe, expect, it } from 'vitest';
import { acceleratorFromEvent, formatAccelerator, isModifierOnly } from './hotkey';

function key(init: KeyboardEventInit & { code: string; key: string }): KeyboardEvent {
  return new KeyboardEvent('keydown', init);
}

describe('acceleratorFromEvent', () => {
  it('builds a portable accelerator from a letter plus modifiers', () => {
    expect(
      acceleratorFromEvent(key({ code: 'KeyD', key: 'd', metaKey: true, shiftKey: true })),
    ).toBe('CommandOrControl+Shift+D');
  });

  it('treats Ctrl and Meta as the same portable modifier', () => {
    expect(acceleratorFromEvent(key({ code: 'KeyD', key: 'd', ctrlKey: true }))).toBe(
      'CommandOrControl+D',
    );
  });

  it('orders modifiers deterministically', () => {
    expect(
      acceleratorFromEvent(
        key({ code: 'KeyK', key: 'k', altKey: true, shiftKey: true, ctrlKey: true }),
      ),
    ).toBe('CommandOrControl+Alt+Shift+K');
  });

  it('supports digits, function keys, and named keys', () => {
    expect(acceleratorFromEvent(key({ code: 'Digit1', key: '1', ctrlKey: true }))).toBe(
      'CommandOrControl+1',
    );
    expect(acceleratorFromEvent(key({ code: 'F5', key: 'F5' }))).toBe('F5');
    expect(acceleratorFromEvent(key({ code: 'Space', key: ' ', ctrlKey: true }))).toBe(
      'CommandOrControl+Space',
    );
    expect(acceleratorFromEvent(key({ code: 'ArrowUp', key: 'ArrowUp', altKey: true }))).toBe(
      'Alt+Up',
    );
  });

  it('rejects a bare letter with no modifier', () => {
    expect(acceleratorFromEvent(key({ code: 'KeyD', key: 'd' }))).toBeNull();
  });

  it('rejects a modifier-only press', () => {
    expect(
      acceleratorFromEvent(key({ code: 'ShiftLeft', key: 'Shift', shiftKey: true })),
    ).toBeNull();
  });

  it('rejects an unsupported key', () => {
    expect(
      acceleratorFromEvent(key({ code: 'IntlBackslash', key: '<', ctrlKey: true })),
    ).toBeNull();
  });
});

describe('isModifierOnly', () => {
  it('detects modifier keys', () => {
    expect(isModifierOnly(key({ code: 'ControlLeft', key: 'Control' }))).toBe(true);
    expect(isModifierOnly(key({ code: 'KeyD', key: 'd' }))).toBe(false);
  });
});

describe('formatAccelerator', () => {
  it('renders mac glyphs', () => {
    expect(formatAccelerator('CommandOrControl+Shift+D', 'mac')).toBe('⌘⇧D');
    expect(formatAccelerator('Alt+Up', 'mac')).toBe('⌥Up');
  });

  it('renders Windows/Linux names', () => {
    expect(formatAccelerator('CommandOrControl+Shift+D', 'other')).toBe('Ctrl+Shift+D');
  });

  it('passes unknown tokens through', () => {
    expect(formatAccelerator('', 'other')).toBe('');
  });
});
