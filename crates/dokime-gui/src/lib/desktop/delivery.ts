/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native delivery payloads for verified documents, mirroring the types beside
   the delivery commands in `src-tauri/src/app/desktop.rs`. */

/* === Local Imports === */

import type { RpcError } from "./rpc";

/* === Types === */

/**
 * The bounded document kinds supported by native delivery.
 *
 * Each kind owns a byte limit and optional extension policy in the native shell.
 * Keeping the kind in the typed contract prevents a caller from selecting a
 * larger limit by changing a filename suffix.
 *
 * `artifact` does not cross IPC as base64 JSON: it is
 * delivered through `desktopArtifactDeliver`'s verified server pull. The
 * support archive does: it is assembled in the webview from
 * `Backend.CollectDiagnostics` frames and so has no artifact, execution-file or
 * cycle id to pull it by, so `diagnostics_bundle` crosses inline under the
 * 32 MiB document bound. Its extension is required rather than
 * appended: a `diagnostics_bundle` filename must already name a `.zip`.
 * `json_document` is JSON assembled in the webview, such as an execution's
 * traceability; it crosses inline under the same bound, and the shell appends
 * `.json` when the name lacks it. `html_report` uses a verified native pull.
 */
export type DesktopSaveKind = "html_report" | "diagnostics_bundle" | "json_document" | "remote_key" | "public_certificate" | "artifact";

/** A verified document handed to the desktop shell for bounded native delivery. */
export type DesktopSaveDocument = {
  kind: DesktopSaveKind;
  filename: string;
  /** Standard base64 preserves the exact verified bytes across JSON IPC. */
  bytesBase64: string;
};

/**
 * Identifies one server-verified content item by its typed reference, never
 * by a client-authored filesystem path.
 */
export type ArtifactSource =
  | { kind: "artifact"; artifactId: string }
  | { kind: "execution_file"; executionId: string; relativePath: string }
  | { kind: "execution_evidence"; executionId: string }
  | { kind: "cycle_evidence"; cycleId: string }
  | { kind: "cycle_report"; cycleId: string }
  | { kind: "execution_report"; executionId: string };

export type Delivered = {
  path: string;
  origin: "current" | "sealed" | null;
};

/** Whether a verified pull is handed to the save dialog or the OS opener. */
export type DeliveryAction = "save" | "open";

/**
 * A request to stream one verified artifact or execution output to a native
 * destination without routing its bytes through the
 * webview.
 */
export type ArtifactDeliveryRequest = {
  source: ArtifactSource;
  kind: DesktopSaveKind;
  filename: string;
  action: DeliveryAction;
};

/** Where a native delivery failed, mirroring `DeliveryStage` in `src-tauri/src/app/content.rs`. */
export type DeliveryStage = "prepare" | "request" | "stream" | "write" | "rename" | "open";

/**
 * How `desktopArtifactDeliver` rejects: the stage that failed and either the
 * backend's own RPC failure or a local message when the backend was never
 * reached.
 */
export type ArtifactDeliveryError = {
  stage: DeliveryStage;
  failure: { kind: "rpc"; detail: RpcError } | { kind: "local"; detail: string };
};
