/** Mirrors `ErrorCode` in `src-tauri/src/error.rs`. Keep the two in sync. */
export const ERROR_CODES = [
  'cli_not_found',
  'cli_failed',
  'cli_timeout',
  'cli_bad_output',
  'llm_auth',
  'llm_rate_limit',
  'llm_http',
  'search_auth',
  'search_rate_limit',
  'search_http',
  'network',
  'keychain',
  'store',
  'io',
  'serde',
  'tauri',
  'hotkey',
  'not_found',
  'invalid',
  'other',
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export interface AppErrorPayload {
  code: ErrorCode;
  message: string;
}

/** Codes whose remedy lives in Settings, so the UI offers a jump there. */
const SETTINGS_CODES: ReadonlySet<ErrorCode> = new Set<ErrorCode>([
  'cli_not_found',
  'cli_bad_output',
  'llm_auth',
  'search_auth',
  'keychain',
  'invalid',
]);

function isErrorCode(value: unknown): value is ErrorCode {
  return typeof value === 'string' && (ERROR_CODES as readonly string[]).includes(value);
}

/**
 * Normalizes anything a rejected `invoke()` (or a thrown JS error) can produce
 * into `{ code, message }`. Tauri rejects with the serialized `AppError`, but a
 * frontend bug, a plugin, or the browser-preview path can throw other shapes.
 */
export function toAppError(caught: unknown): AppErrorPayload {
  if (typeof caught === 'object' && caught !== null && 'message' in caught) {
    const record = caught as { code?: unknown; message?: unknown };
    const message = String(record.message ?? '').trim();
    return {
      code: isErrorCode(record.code) ? record.code : 'other',
      message: message.length > 0 ? message : 'unknown error',
    };
  }

  const message = String(caught ?? '').trim();
  return { code: 'other', message: message.length > 0 ? message : 'unknown error' };
}

export function isSettingsError(code: ErrorCode): boolean {
  return SETTINGS_CODES.has(code);
}

export function errorKey(code: ErrorCode): string {
  return `error.${code}`;
}
