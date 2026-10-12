// Dokime — SPDX-License-Identifier: Apache-2.0

import type { ConnectionError } from "../desktop/connection";
import type { RpcError } from "../desktop/rpc";

const connectionErrorMessages: Record<
  Exclude<ConnectionError["kind"], "invalid_target" | "execution_rejected">,
  string
> = {
  credentials_unavailable: "No saved access token is available. Enter a token to connect.",
  authentication_required: "The backend requires a valid access token.",
  unauthenticated: "The backend requires a valid access token.",
  permission_denied: "This account does not have permission to use the backend.",
  transport: "The endpoint could not be reached or its TLS certificate could not be verified.",
  incompatible_api: "The backend API is incompatible with this desktop version.",
  identity_changed: "The backend identity does not match the expected identity.",
  backend_unavailable: "The backend could not be reached.",
  wait_timed_out: "The request wait deadline elapsed. Execution may still be running.",
  superseded: "The connection request was replaced.",
  not_connected: "The backend is not connected.",
  generation_exhausted: "The connection session has expired.",
};

function connectionMessage(error: ConnectionError): string {
  return "message" in error ? error.message : connectionErrorMessages[error.kind];
}

function rpcMessage(error: RpcError): string {
  switch (error.kind) {
    case "session":
      return connectionMessage(error.detail);
    case "grpc":
      return (
        error.detail.message.trim() ||
        `The native backend returned gRPC status ${error.detail.code}.`
      );
    case "invalid_request":
      return "The native RPC request was invalid.";
    case "incompatible_descriptor":
      return "The desktop interface bindings do not match this application.";
    case "limit_exceeded":
      return "The native RPC request exceeded its size limit.";
    case "busy":
      return "Another native RPC stream operation is already in progress.";
    case "unknown_stream":
      return "The native RPC stream is no longer available.";
    case "invalid_ack":
      return "The native RPC stream returned an invalid acknowledgement.";
    case "cancelled":
      return "The native RPC request was cancelled.";
  }
}

function isConnectionError(error: unknown): error is ConnectionError {
  if (!error || typeof error !== "object" || !("kind" in error)) return false;
  const kind = (error as { kind?: unknown }).kind;
  if (kind === "execution_rejected") {
    return (
      "code" in error &&
      typeof error.code === "string" &&
      "message" in error &&
      typeof error.message === "string" &&
      "issues" in error &&
      Array.isArray(error.issues) &&
      error.issues.every(
        (issue: unknown) =>
          !!issue &&
          typeof issue === "object" &&
          "path" in issue &&
          typeof issue.path === "string" &&
          "message" in issue &&
          typeof issue.message === "string" &&
          "expected" in issue &&
          (issue.expected === null || typeof issue.expected === "string"),
      )
    );
  }
  if (kind === "invalid_target") {
    return "message" in error && typeof error.message === "string";
  }
  return typeof kind === "string" && Object.hasOwn(connectionErrorMessages, kind);
}

function isRpcError(error: unknown): error is RpcError {
  if (!error || typeof error !== "object" || !("kind" in error)) return false;
  const kind = (error as { kind?: unknown }).kind;
  switch (kind) {
    case "session":
      return isConnectionError((error as { detail?: unknown }).detail);
    case "grpc": {
      const detail = (error as { detail?: unknown }).detail;
      return (
        !!detail &&
        typeof detail === "object" &&
        "code" in detail &&
        typeof detail.code === "number" &&
        "message" in detail &&
        typeof detail.message === "string" &&
        "details" in detail &&
        Array.isArray(detail.details)
      );
    }
    case "invalid_request":
    case "incompatible_descriptor":
    case "limit_exceeded":
    case "busy":
    case "unknown_stream":
    case "invalid_ack":
    case "cancelled":
      return true;
    default:
      return false;
  }
}

/** Keep native RPC failures usable by UI code that displays Error.message. */
export function normalizeRpcError(error: unknown): unknown {
  if (error instanceof Error || !isRpcError(error)) return error;
  const normalized = new Error(rpcMessage(error));
  normalized.name = "NativeRpcError";
  Object.assign(normalized, {
    kind: error.kind,
    ...("detail" in error ? { detail: error.detail } : {}),
  });
  return normalized;
}
