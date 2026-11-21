/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native RPC failures mirror `Error` in `src-tauri/src/error.rs`.
 * Grpc.details always encodes generated harmos.model.v1.Failure, normalized by
 * the native boundary for product, library and plain-status failures alike. */

/* === Local Imports === */

import type { ConnectionError } from "./connection";

/* === Types === */

export type RpcError =
  | { kind: "session"; detail: ConnectionError }
  | { kind: "local"; detail: { message: string } }
  | { kind: "invalid_request" }
  | { kind: "incompatible_descriptor" }
  | { kind: "limit_exceeded" }
  | { kind: "busy" }
  | { kind: "unknown_stream" }
  | { kind: "invalid_ack" }
  | { kind: "cancelled" }
  | { kind: "grpc"; detail: { code: number; message: string; details: number[] } };
