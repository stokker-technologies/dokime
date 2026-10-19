/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The gate in front of the one destructive action in the application.
 *
 * Resetting local storage deletes every cycle, execution, record and file in
 * the owned backend's data directory. It is offered only to a development
 * build whose session owns a local backend, and it runs only once the
 * operator has typed the confirmation word exactly. The module imports types
 * only, so a node test can load it directly.
 */

/* === Local Imports === */

import type { BackendOwnership } from "@/lib/desktop/session";

/* === Types === */

export type StorageResetContext = {
  /** Whether the frontend is served by the development server. */
  development: boolean;
  /** Whether this window runs inside the desktop shell. */
  desktop: boolean;
  ownership: BackendOwnership | null | undefined;
  /** The data directory the backend reports, which the dialog names. */
  directory: string | null | undefined;
};

/* === Public Constants === */

export const STORAGE_RESET_CONFIRMATION = "RESET";

/* === Public Functions === */

/** Whether the Development card offers the reset at all. */
export function storageResetAvailable(context: StorageResetContext): boolean {
  return (
    context.development &&
    context.desktop &&
    context.ownership === "owned_local" &&
    typeof context.directory === "string" &&
    context.directory.length > 0
  );
}

/** Whether the typed text arms the reset: the exact word, nothing else. */
export function storageResetConfirmed(typed: string): boolean {
  return typed === STORAGE_RESET_CONFIRMATION;
}
