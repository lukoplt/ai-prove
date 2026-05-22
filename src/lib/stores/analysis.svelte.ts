import { analyzeText, onAnalysisClaims, onAnalysisStarted, onClaimVerified } from '$lib/api';
import type { Analysis, AnalyzeInput, Claim } from '$lib/types';
import type { UnlistenFn } from '@tauri-apps/api/event';

type Status = 'idle' | 'running' | 'done' | 'error';

let status = $state<Status>('idle');
let current = $state<Analysis | null>(null);
let selectedId = $state<string | null>(null);
let error = $state<string | null>(null);
let started = false;
const unlistens: UnlistenFn[] = [];

async function ensureSubscriptions() {
  if (started) return;
  started = true;
  unlistens.push(
    await onAnalysisStarted(() => {
      status = 'running';
      current = null;
      selectedId = null;
      error = null;
    }),
  );
  unlistens.push(
    await onAnalysisClaims(({ analysis }) => {
      current = analysis;
      status = 'done';
      selectedId = analysis.claims[0]?.id ?? null;
    }),
  );
  unlistens.push(
    await onClaimVerified(({ analysisId, claimId, verification }) => {
      if (!current || current.id !== analysisId) return;
      current = {
        ...current,
        claims: current.claims.map((claim) =>
          claim.id === claimId ? { ...claim, verification } : claim,
        ),
      };
    }),
  );
}

export const analysisStore = {
  get status() {
    return status;
  },
  get current() {
    return current;
  },
  get selectedId() {
    return selectedId;
  },
  get error() {
    return error;
  },
  get selectedClaim(): Claim | null {
    if (!current || !selectedId) return null;
    return current.claims.find((claim) => claim.id === selectedId) ?? null;
  },

  async init(): Promise<void> {
    await ensureSubscriptions();
  },

  async run(input: string | AnalyzeInput): Promise<void> {
    await ensureSubscriptions();
    error = null;
    try {
      await analyzeText(input);
    } catch (caught) {
      status = 'error';
      error = String(caught);
    }
  },

  select(id: string): void {
    selectedId = id;
  },

  reset(): void {
    status = 'idle';
    current = null;
    selectedId = null;
    error = null;
  },
};
