import { getSettings, hasApiKey, setSettings } from '$lib/api';
import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, type Settings } from '$lib/types';

const defaults: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  model: 'claude-haiku-4-5-20251001',
  cache_ttl_days: 7,
  onboarded: false,
};

let current = $state<Settings>(defaults);
let anthropicPresent = $state(false);
let bravePresent = $state(false);
let loaded = $state(false);

export const settings = {
  get current() {
    return current;
  },
  get anthropicPresent() {
    return anthropicPresent;
  },
  get bravePresent() {
    return bravePresent;
  },
  get loaded() {
    return loaded;
  },

  async load(): Promise<void> {
    current = await getSettings();
    anthropicPresent = await hasApiKey(ACCOUNT_ANTHROPIC);
    bravePresent = await hasApiKey(ACCOUNT_BRAVE);
    loaded = true;
  },

  async save(next: Settings): Promise<void> {
    await setSettings(next);
    current = next;
  },

  async refreshKeyState(): Promise<void> {
    anthropicPresent = await hasApiKey(ACCOUNT_ANTHROPIC);
    bravePresent = await hasApiKey(ACCOUNT_BRAVE);
  },
};
