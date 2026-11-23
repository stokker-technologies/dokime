// Dokime — SPDX-License-Identifier: Apache-2.0
export type ConnectionState = "connected" | "reconnecting" | "disconnected" | "unknown";

export type BackendStatus = {
  /** Operator-facing backend name, or null while the identity read is pending. */
  label: string | null;
  connection: ConnectionState;
  /** Connection failure reported by the runtime subscription. */
  detail: string | null;
};
