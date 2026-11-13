/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Visibility of the Disabled Development Recovery Control. */

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
