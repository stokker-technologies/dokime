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

/** Secret-free launch preferences supplied by the native process. */
export type DesktopLaunch = {
  target: ConnectionTarget | null;
  expectedBackendId: string | null;
  localBackendAvailable: boolean;
  error: string | null;
};
