/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Desktop connection wire types, mirroring `src-tauri/src/connection/contract.rs`.
   All protobuf 64-bit integers cross IPC as decimal strings. */

/* === Types === */

/** Attaching to a caller-owned listener never starts a runtime. */
export type ConnectionTarget =
  | { kind: "local"; endpoint: string }
  | { kind: "remote"; endpoint: string; caCertificatePem: string | null; endpoints?: string[] };

/** Input-only secrets. Secrets never appear in a connection snapshot. */
export type ConnectionSecrets =
  | { kind: "none" }
  | { kind: "saved" }
  | { kind: "profile"; credential: string }
  | { kind: "bearer"; token: string; remember: boolean };

export type ConnectRequest = {
  target: ConnectionTarget;
  secrets: ConnectionSecrets;
  expectedBackendId: string | null;
};

export type ConnectionPhase = "disconnected" | "connecting" | "connected" | "failed";

export type BackendLimits = {
  maxRequestBytes: string;
  maxResponseBytes: string;
  maxContentBytes: string | null;
  maxEvidenceBytes: string | null;
  maxChunkBytes: number | null;
  maxPageItems: number | null;
  maxStreamBufferItems: number | null;
};

export type BackendSnapshot = {
  backendId: string;
  epoch: string;
  displayName: string;
  productVersion: string;
  target: string;
  apiVersion: string;
  descriptorSha256: string;
  methods: string[];
  limits: BackendLimits;
};

export type ConnectionSnapshot = {
  /** Monotonic session generation, encoded exactly rather than as a JS number. */
  generation: string;
  phase: ConnectionPhase;
  endpoint: string | null;
  remote: boolean;
  backend: BackendSnapshot | null;
  error: ConnectionError | null;
};

export type ConnectionError =
  | { kind: "registry"; message: string }
  | { kind: "invalid_target"; message: string }
  | { kind: "secrets_unavailable" }
  | { kind: "authentication_required" }
  | { kind: "unauthenticated" }
  | { kind: "permission_denied" }
  | { kind: "transport" }
  | { kind: "incompatible_api" }
  | { kind: "identity_changed" }
  | { kind: "backend_unavailable" }
  | {
      kind: "execution_rejected";
      code: string;
      message: string;
      issues: { path: string; message: string; expected: string | null }[];
    }
  | { kind: "wait_timed_out" }
  | { kind: "superseded" }
  | { kind: "not_connected" }
  | { kind: "generation_exhausted" };
