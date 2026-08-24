import { getSettings, hasApiKey, setSettings } from '$lib/api';
import { ACCOUNT_ANTHROPIC, ACCOUNT_BRAVE, DEFAULT_SETTINGS, type Settings } from '$lib/types';

let current = $state<Settings>(DEFAULT_SETTINGS);
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
