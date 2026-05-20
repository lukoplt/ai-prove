import { invoke } from '@tauri-apps/api/core';
import type { ApiAccount, Settings } from './types';

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function setSettings(settings: Settings): Promise<void> {
  await invoke('set_settings', { settings });
}

export async function setApiKey(account: ApiAccount, secret: string): Promise<void> {
  await invoke('set_api_key', { account, secret });
}

export async function clearApiKey(account: ApiAccount): Promise<void> {
  await invoke('clear_api_key', { account });
}

export async function hasApiKey(account: ApiAccount): Promise<boolean> {
  return invoke<boolean>('has_api_key', { account });
}
