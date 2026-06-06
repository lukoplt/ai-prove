import { invoke } from '@tauri-apps/api/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import {
  DEFAULT_ANTHROPIC_MODEL,
  DEFAULT_CLI_COMMAND,
  DEFAULT_VERIFIED_CLAIMS_LIMIT,
  type AnalyzeInput,
  type Analysis,
  type ApiAccount,
  type Claim,
  type LatestRelease,
  type Settings,
  type Verification,
} from './types';

const SETTINGS_STORAGE_KEY = 'prove:settings';
const browserStartedHandlers = new Set<(event: AnalysisStartedEvent) => void>();
const browserClaimsHandlers = new Set<(event: AnalysisClaimsEvent) => void>();
const browserVerifiedHandlers = new Set<(event: ClaimVerifiedEvent) => void>();

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function browserDefaultSettings(): Settings {
  const language = typeof navigator === 'undefined' ? 'en' : navigator.language;
  return {
    locale: language.toLowerCase().startsWith('cs') ? 'cs' : 'en',
    hotkey: 'CommandOrControl+Shift+D',
    cache_ttl_days: 7,
    onboarded: false,
    provider: 'cli',
    anthropic_model: DEFAULT_ANTHROPIC_MODEL,
    cli_command: DEFAULT_CLI_COMMAND,
    check_updates_on_launch: false,
    theme: 'auto',
    verified_claims_limit: DEFAULT_VERIFIED_CLAIMS_LIMIT,
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
    sessionStorage.setItem(`prove:key:${account}`, String(secret.trim().length > 0));
    return;
  }

  await invoke('set_api_key', { account, secret });
}

export async function clearApiKey(account: ApiAccount): Promise<void> {
  if (!isTauriRuntime()) {
    sessionStorage.removeItem(`prove:key:${account}`);
    return;
  }

  await invoke('clear_api_key', { account });
}

export async function hasApiKey(account: ApiAccount): Promise<boolean> {
  if (!isTauriRuntime()) return sessionStorage.getItem(`prove:key:${account}`) === 'true';
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

export async function analyzeText(input: string | AnalyzeInput): Promise<string> {
  const request = normalizeAnalyzeInput(input);
  if (isTauriRuntime()) {
    return invoke<string>('analyze_text', {
      question: request.question,
      answer: request.answer,
    });
  }

  const analysisId = crypto.randomUUID();
  const trimmed = request.answer.trim();
  emitBrowserStarted({ analysisId });
  await new Promise((resolve) => setTimeout(resolve, 250));
  const analysis = buildBrowserAnalysis(analysisId, trimmed);
  emitBrowserClaims({
    analysisId,
    analysis,
  });
  queueBrowserVerifications(analysisId, analysis.claims);
  return analysisId;
}

function normalizeAnalyzeInput(input: string | AnalyzeInput): AnalyzeInput {
  if (typeof input === 'string') {
    return { answer: input };
  }

  return {
    question: input.question?.trim(),
    answer: input.answer,
  };
}

export interface AnalysisStartedEvent {
  analysisId: string;
}

export interface AnalysisClaimsEvent {
  analysisId: string;
  analysis: Analysis;
}

export interface ClaimVerifiedEvent {
  analysisId: string;
  claimId: string;
  verification: Verification;
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

export async function onClaimVerified(
  handler: (event: ClaimVerifiedEvent) => void,
): Promise<UnlistenFn> {
  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<ClaimVerifiedEvent>('claim-verified', (message) => handler(message.payload));
  }

  browserVerifiedHandlers.add(handler);
  return () => browserVerifiedHandlers.delete(handler);
}

export async function openInBrowser(url: string): Promise<void> {
  if (isTauriRuntime()) {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open(url);
    return;
  }

  window.open(url, '_blank', 'noopener');
}

/**
 * Opt-in check for newer GitHub Releases. Returns null when:
 * - the app is running in browser preview (no Tauri runtime), or
 * - the GitHub request fails for any reason (offline, rate limit, etc.).
 *
 * Never throws. The banner UX should treat null as "nothing to show".
 */
export async function checkLatestRelease(): Promise<LatestRelease | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await invoke<LatestRelease>('check_latest_release');
  } catch (error) {
    console.warn('check_latest_release failed', error);
    return null;
  }
}

function emitBrowserStarted(event: AnalysisStartedEvent): void {
  for (const handler of browserStartedHandlers) handler(event);
}

function emitBrowserClaims(event: AnalysisClaimsEvent): void {
  for (const handler of browserClaimsHandlers) handler(event);
}

function emitBrowserVerified(event: ClaimVerifiedEvent): void {
  for (const handler of browserVerifiedHandlers) handler(event);
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

function browserLocale(): 'cs' | 'en' {
  const language = typeof navigator === 'undefined' ? 'en' : navigator.language;
  return language.toLowerCase().startsWith('cs') ? 'cs' : 'en';
}

function browserPreviewStrings(locale: 'cs' | 'en') {
  if (locale === 'cs') {
    return {
      claimReason: 'Lokální vývojový náhled bez volání LLM.',
      verifiedSummary: 'Lokální vývojový náhled: tvrzení je označeno jako ověřené.',
      sampleSourceTitle: 'Ukázkový zdroj pro lokální náhled',
    };
  }
  return {
    claimReason: 'Local development preview without LLM calls.',
    verifiedSummary: 'Local development preview: claim is marked as supported.',
    sampleSourceTitle: 'Sample source for local preview',
  };
}

function skippedSummaryText(locale: 'cs' | 'en', limit: number): string {
  return locale === 'cs'
    ? `Ověřuje se jen prvních ${limit} faktických tvrzení.`
    : `Only the first ${limit} factual claims are verified.`;
}

function browserClaims(input: string): Claim[] {
  const locale = browserLocale();
  const reason = browserPreviewStrings(locale).claimReason;
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
      reason,
      verification: null,
    };
  });
}

function browserKind(text: string): Claim['kind'] {
  const lower = text.toLowerCase();
  const opinionMarkers = [
    'podle mě',
    'nejlepší',
    'skvěl',
    'in my opinion',
    'i think',
    'best',
    'great',
  ];
  if (opinionMarkers.some((marker) => lower.includes(marker))) {
    return 'opinion';
  }

  const inferenceMarkers = ['protože', 'vyplývá', 'therefore', 'because', 'follows that'];
  if (inferenceMarkers.some((marker) => lower.includes(marker))) {
    return 'inference';
  }

  return 'fact';
}

function queueBrowserVerifications(analysisId: string, claims: Claim[]): void {
  const locale = browserLocale();
  const { verifiedSummary, sampleSourceTitle } = browserPreviewStrings(locale);
  const factClaims = claims.filter((claim) => claim.kind === 'fact');

  const limit = browserReadSettings().verified_claims_limit;
  const verifiedCount = limit === null ? factClaims.length : limit;

  factClaims.slice(0, verifiedCount).forEach((claim, index) => {
    setTimeout(
      () => {
        emitBrowserVerified({
          analysisId,
          claimId: claim.id,
          verification: {
            status: 'supported',
            summary: verifiedSummary,
            sources: [
              {
                url: 'https://cs.wikipedia.org/wiki/Karel_IV',
                title: sampleSourceTitle,
                snippet: claim.text,
                tier: 'a',
                stance: 'supports',
              },
            ],
          },
        });
      },
      450 + index * 350,
    );
  });

  const skippedSummary = skippedSummaryText(locale, verifiedCount);
  factClaims.slice(verifiedCount).forEach((claim, index) => {
    setTimeout(
      () => {
        emitBrowserVerified({
          analysisId,
          claimId: claim.id,
          verification: {
            status: 'not_verified',
            summary: skippedSummary,
            sources: [],
          },
        });
      },
      450 + verifiedCount * 350 + index * 120,
    );
  });
}
