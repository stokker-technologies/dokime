/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native RPC failures, mirroring `RpcError` in `src-tauri/src/rpc/mod.rs`. */

/* === Local Imports === */

import type { ConnectionError } from "./connection";

/* === Types === */

export type RpcError =
  | { kind: "session"; detail: ConnectionError }
  | { kind: "invalid_request" }
  | { kind: "incompatible_descriptor" }
  | { kind: "limit_exceeded" }
  | { kind: "busy" }
  | { kind: "unknown_stream" }
  | { kind: "invalid_ack" }
  | { kind: "cancelled" }
  | { kind: "grpc"; detail: { code: number; message: string; details: number[] } };
