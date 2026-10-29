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

import type { ConnectionSnapshot, ConnectionTarget, ConnectRequest } from "./connection";

export const DESKTOP_SESSION_STATUS_EVENT = "desktop://session-status";

export type BackendSelection =
  | { kind: "owned_local" }
  | { kind: "profile"; identifier: string }
  | { kind: "attach"; request: ConnectRequest };

export type BackendOwnership = "none" | "owned_local" | "attached";

export type DesktopSessionSnapshot = {
  /** Monotonic owner status sequence used to reconcile startup events with snapshots. */
  statusRevision: number;
  ownership: BackendOwnership;
  /** The startup phase an owned local backend last reported while starting
   * (`"storage"`, `"plugins"`, `"requests"`, …, free-form so an older GUI
   * tolerates a name it does not recognize). `null` once connected or
   * disconnected, and throughout an attach selection. */
  phase: string | null;
  connection: ConnectionSnapshot;
};

/** A failed embedded-server start and the shared application's diagnostic log. */
export type BackendDiagnostics = {
  cause: string;
  log: string | null;
};

/** Secret-free launch preferences supplied by the native process. */
export type DesktopLaunch = {
  target: ConnectionTarget | null;
  expectedBackendId: string | null;
  localBackendAvailable: boolean;
  error: string | null;
};
