// Dokime — SPDX-License-Identifier: Apache-2.0
import type { BackendDiagnostics } from "@/lib/desktop/session";

/** The diagnostics a failed owned-backend start carries, or `null` for every
 * other error. */
export function startupDiagnostics(error: unknown): BackendDiagnostics | null {
  if (error && typeof error === "object" && "kind" in error && error.kind === "archived" && "error" in error)
    return startupDiagnostics(error.error);
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "startup") return null;
  if (!("diagnostics" in error) || !error.diagnostics || typeof error.diagnostics !== "object") return null;
  return error.diagnostics as BackendDiagnostics;
}

/** Local recovery is independent of the startup diagnostic payload's presence. */
export function storageRecoveryAvailable(error: unknown, localAttempt: boolean, localAvailable: boolean): boolean {
  if (!error || !localAvailable) return false;
  if (typeof error !== "object") return localAttempt;
  if (!("kind" in error)) return localAttempt;
  if (error.kind === "backend_running" || ("reason" in error && error.reason === "backend_running")) return false;
  if (error.kind === "archived" && "error" in error) return storageRecoveryAvailable(error.error, localAttempt, localAvailable);
  return error.kind === "startup" || localAttempt;
}

export function storageRecoveryArchive(error: unknown): string | null {
  return error &&
    typeof error === "object" &&
    "kind" in error &&
    error.kind === "archived" &&
    "archive" in error &&
    typeof error.archive === "string"
    ? error.archive
    : null;
}
