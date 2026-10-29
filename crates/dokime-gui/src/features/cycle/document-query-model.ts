/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* How an evidence pane treats its document read.
 *
 * A document once read stays on screen while later reads fail. A busy refusal
 * (the resolver answering that it is busy, or the bridge refusing a second
 * stream) is the backend asking for patience: the read is retried with capped
 * exponential backoff for as long as it is refused and the pane shows it as
 * pending, never as a failure. RESOURCE_EXHAUSTED alone is not busy: the
 * backend also answers it for fixed budgets no retry can satisfy. Any other
 * failure is retried once, then shown with the backend's own words. The module
 * has no application imports, so a node test can load it directly.
 */

const GUI_RECONNECT_INITIAL_MS = 1_000;
const GUI_RECONNECT_MAX_MS = 10_000;

const RETRY_INITIAL_MS = GUI_RECONNECT_INITIAL_MS;
/** The longest wait between reads of a document resolver or plot stream. */
export const RECONNECT_MAX_MS = GUI_RECONNECT_MAX_MS;

export const DOCUMENT_FAILED_FALLBACK = "The document could not be read from the backend.";
export const BUSY_NOTICE = "The backend is busy; the document is read again shortly.";

export type EvidencePaneState =
  | { kind: "document"; staleReason: string | null }
  | { kind: "loading" }
  | { kind: "busy" }
  | { kind: "failed"; message: string };

/** Whether the backend refused the read for load rather than for a fault. */
export function isBusyRefusal(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  if ((error as { kind?: unknown }).kind === "busy") return true;
  return error instanceof Error && /\bbusy\b/i.test(error.message);
}

/** TanStack Query `retry`: busy refusals keep retrying, other failures retry once. */
export function documentRetry(failureCount: number, error: unknown): boolean {
  return isBusyRefusal(error) || failureCount < 1;
}

/** TanStack Query `retryDelay`: exponential backoff with the shared reconnect cap. */
export function documentRetryDelay(attempt: number, _error: unknown): number {
  return Math.min(RETRY_INITIAL_MS * 2 ** attempt, RECONNECT_MAX_MS);
}

/**
 * What the pane shows for one document read: the document whenever one has
 * been read, pending while the first read is under way or refused as busy,
 * and the failure only when no document exists and the backend truly failed.
 */
export function evidencePaneState({
  hasDocument,
  error,
  failureReason,
  pending,
}: {
  hasDocument: boolean;
  /** The settled failure, once retries are exhausted. */
  error: unknown;
  /** The latest failure while retries are still under way. */
  failureReason: unknown;
  pending: boolean;
}): EvidencePaneState {
  const failure = error ?? failureReason;
  const busy = failure != null && isBusyRefusal(failure);

  if (hasDocument) {
    return { kind: "document", staleReason: error != null && !busy ? failureMessage(error) : null };
  }
  if (busy) return { kind: "busy" };
  if (error != null) return { kind: "failed", message: failureMessage(error) };
  if (pending) return { kind: "loading" };
  return { kind: "document", staleReason: null };
}

function failureMessage(error: unknown): string {
  return error instanceof Error && error.message.trim() ? error.message : DOCUMENT_FAILED_FALLBACK;
}
