import { invoke } from '@tauri-apps/api/core';
import type { ApiAccount, Settings } from './types';

const SETTINGS_STORAGE_KEY = 'druhy-nazor:settings';

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function browserDefaultSettings(): Settings {
  const language = typeof navigator === 'undefined' ? 'en' : navigator.language;
  return {
    locale: language.toLowerCase().startsWith('cs') ? 'cs' : 'en',
    hotkey: 'CommandOrControl+Shift+D',
    model: 'claude-haiku-4-5-20251001',
    cache_ttl_days: 7,
    onboarded: false,
  };
}

function browserReadSettings(): Settings {
  if (typeof localStorage === 'undefined') return browserDefaultSettings();

  const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (!raw) return browserDefaultSettings();

  try {
    return { ...browserDefaultSettings(), ...(JSON.parse(raw) as Partial<Settings>) };
  } catch {
    return browserDefaultSettings();
  }
}

export async function getSettings(): Promise<Settings> {
  if (!isTauriRuntime()) return browserReadSettings();
  return invoke<Settings>('get_settings');
}

export async function setSettings(settings: Settings): Promise<void> {
  if (!isTauriRuntime()) {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
    return;
  }

  await invoke('set_settings', { settings });
}

export async function setApiKey(account: ApiAccount, secret: string): Promise<void> {
  if (!isTauriRuntime()) {
    sessionStorage.setItem(`druhy-nazor:key:${account}`, String(secret.trim().length > 0));
    return;
  }

  await invoke('set_api_key', { account, secret });
}

export async function clearApiKey(account: ApiAccount): Promise<void> {
  if (!isTauriRuntime()) {
    sessionStorage.removeItem(`druhy-nazor:key:${account}`);
    return;
  }

  await invoke('clear_api_key', { account });
}

export async function hasApiKey(account: ApiAccount): Promise<boolean> {
  if (!isTauriRuntime()) return sessionStorage.getItem(`druhy-nazor:key:${account}`) === 'true';
  return invoke<boolean>('has_api_key', { account });
}

export async function readClipboardText(): Promise<string> {
  if (isTauriRuntime()) {
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
    return readText();
  }

  try {
    return (await navigator.clipboard?.readText()) ?? '';
  } catch {
    return '';
  }
}
