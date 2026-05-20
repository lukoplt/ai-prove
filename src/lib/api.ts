import { invoke } from '@tauri-apps/api/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { Analysis, ApiAccount, Claim, Settings } from './types';

const SETTINGS_STORAGE_KEY = 'druhy-nazor:settings';
const browserStartedHandlers = new Set<(event: AnalysisStartedEvent) => void>();
const browserClaimsHandlers = new Set<(event: AnalysisClaimsEvent) => void>();

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

export async function analyzeText(text: string): Promise<string> {
  if (isTauriRuntime()) return invoke<string>('analyze_text', { text });

  const analysisId = crypto.randomUUID();
  const trimmed = text.trim();
  emitBrowserStarted({ analysisId });
  await new Promise((resolve) => setTimeout(resolve, 250));
  emitBrowserClaims({
    analysisId,
    analysis: buildBrowserAnalysis(analysisId, trimmed),
  });
  return analysisId;
}

export interface AnalysisStartedEvent {
  analysisId: string;
}

export interface AnalysisClaimsEvent {
  analysisId: string;
  analysis: Analysis;
}

export async function onAnalysisStarted(
  handler: (event: AnalysisStartedEvent) => void,
): Promise<UnlistenFn> {
  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<AnalysisStartedEvent>('analysis-started', (message) => handler(message.payload));
  }

  browserStartedHandlers.add(handler);
  return () => browserStartedHandlers.delete(handler);
}

export async function onAnalysisClaims(
  handler: (event: AnalysisClaimsEvent) => void,
): Promise<UnlistenFn> {
  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<AnalysisClaimsEvent>('analysis-claims', (message) => handler(message.payload));
  }

  browserClaimsHandlers.add(handler);
  return () => browserClaimsHandlers.delete(handler);
}

function emitBrowserStarted(event: AnalysisStartedEvent): void {
  for (const handler of browserStartedHandlers) handler(event);
}

function emitBrowserClaims(event: AnalysisClaimsEvent): void {
  for (const handler of browserClaimsHandlers) handler(event);
}

function buildBrowserAnalysis(id: string, input: string): Analysis {
  const claims = browserClaims(input);
  return {
    id,
    created_at: Date.now(),
    input,
    claims: claims.slice(0, 25),
    truncated: claims.length > 25,
  };
}

function browserClaims(input: string): Claim[] {
  const matches = [...input.matchAll(/[^.!?\n]+[.!?]?/g)].filter((match) => match[0].trim());
  return matches.map((match, index) => {
    const raw = match[0];
    const leading = raw.length - raw.trimStart().length;
    const text = raw.trim();
    const start = (match.index ?? 0) + leading;
    const end = start + text.length;
    return {
      id: `c${index + 1}`,
      text,
      start,
      end,
      kind: browserKind(text),
      reason: 'Lokální vývojový náhled bez volání LLM.',
      verification: null,
    };
  });
}

function browserKind(text: string): Claim['kind'] {
  const lower = text.toLowerCase();
  if (lower.includes('podle mě') || lower.includes('nejlepší') || lower.includes('skvěl')) {
    return 'opinion';
  }
  if (lower.includes('protože') || lower.includes('vyplývá')) return 'inference';
  return 'fact';
}
