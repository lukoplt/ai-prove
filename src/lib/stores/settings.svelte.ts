import { getSettings, hasApiKey, setSettings } from '$lib/api';
import {
  ACCOUNT_ANTHROPIC,
  ACCOUNT_BRAVE,
  DEFAULT_ANTHROPIC_MODEL,
  DEFAULT_CLI_COMMAND,
  type Settings,
} from '$lib/types';

const defaults: Settings = {
  locale: 'cs',
  hotkey: 'CommandOrControl+Shift+D',
  cache_ttl_days: 7,
  onboarded: false,
  provider: 'cli',
  anthropic_model: DEFAULT_ANTHROPIC_MODEL,
  cli_command: DEFAULT_CLI_COMMAND,
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
