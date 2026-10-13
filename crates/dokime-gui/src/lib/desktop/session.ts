/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Desktop lifecycle ownership, mirroring `src-tauri/src/native_session/contract.rs`
   and the launch preferences in `src-tauri/src/native_session/launch.rs`. */

/* === Local Imports === */

import type { ConnectionSnapshot, ConnectionTarget, ConnectRequest } from "./connection";

/* === Types === */

export type BackendSelection =
  | { kind: "owned_local" }
  | { kind: "profile"; name: string }
  | { kind: "attach"; request: ConnectRequest };

export type BackendOwnership = "none" | "owned_local" | "attached";

export type DesktopSessionSnapshot = {
  ownership: BackendOwnership;
  /** The startup phase an owned local backend last reported while starting
   * (`"storage"`, `"plugins"`, `"requests"`, …, free-form so an older GUI
   * tolerates a name it does not recognize). `null` once connected or
   * disconnected, and throughout an attach selection. */
  phase: string | null;
  connection: ConnectionSnapshot;
};

/** What an owned local backend left behind when it failed to start, carried
 * as `diagnostics` on the `{ kind: "startup", message }` session error. Paths
 * are absolute; the tail and the log are redacted of credentials natively. */
export type BackendDiagnostics = {
  /** The resolved backend executable, so a wrong discovery is visible. */
  executable: string;
  exitCode: number | null;
  /** The platform's exit description, also set when no code exists (a signal). */
  exitStatus: string | null;
  /** The last stderr lines, bounded natively to 200 lines and 64 KiB. */
  tail: string;
  /** The complete stderr log of this launch, when a log directory resolved. */
  log: string | null;
};

/** Secret-free launch preferences supplied by the native process. */
export type DesktopLaunch = {
  target: ConnectionTarget | null;
  expectedBackendId: string | null;
  localBackendAvailable: boolean;
  error: string | null;
};
