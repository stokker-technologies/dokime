/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Project Canonical Native Failure Details Into Notifications
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { fromBinary } from "@bufbuild/protobuf";

// Local Imports
import type { Failure } from "@host/model";
import type { Wire } from "@/lib/rpc/canonical";
import type { RpcError } from "@/lib/desktop/rpc";
import type { NotificationFailure } from "@/lib/notification-details";
import { FailureSchema } from "./generated/errors";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function rpcFailureDetails(error: RpcError, reason: string): Partial<NotificationFailure> {
  if (error.kind === "session") {
    const detail = error.detail;
    return {
      reason: detail.kind === "execution_rejected" ? detail.code : detail.kind,
      problems:
        detail.kind === "execution_rejected"
          ? detail.issues.map((issue) => ({
              label: issue.path,
              message: issue.message,
              causes: issue.expected ? [`Expected: ${issue.expected}`] : undefined,
            }))
          : undefined,
    };
  }
  if (error.kind !== "grpc") return { reason };
  const failure: Partial<NotificationFailure> = { reason, status: reason, message: error.detail.message };
  if (!error.detail.details.length) return failure;
  try {
    return { ...failure, ...nativeFailure(fromBinary(FailureSchema, Uint8Array.from(error.detail.details)), reason) };
  } catch {
    return { ...failure, diagnostics: `Undecodable Desktop Details (Base64):\n${toBase64(Uint8Array.from(error.detail.details))}` };
  }
}

/** Project the same generated failure used in statuses and plugin entries. */
export function nativeFailure(rejection: Wire<Failure>, reason: string): Partial<NotificationFailure> {
  const chain: Wire<Failure>[] = [];
  for (let current: Wire<Failure> | undefined = rejection; current; current = current.cause) chain.push(current);
  return {
    message: rejection.message || undefined,
    reason: rejection.code || reason,
    action: chain.find((failure) => failure.action)?.action || undefined,
    plugin: chain.find((failure) => failure.plugin)?.plugin || undefined,
    failureOperation: rejection.operation || undefined,
    causes: [
      ...new Set(
        chain
          .slice(1)
          .map((failure) => failure.message)
          .filter((message) => message && message !== rejection.message),
      ),
    ],
    causeDetails: chain.slice(1).map(({ code, message, operation, plugin, action }) => ({ code, message, operation, plugin, action })),
    identifiers: Object.assign({}, ...chain.map((failure) => failure.details)),
    diagnostics: [...new Set(chain.map((failure) => failure.diagnostics).filter(Boolean))].join("\n\n") || undefined,
  };
}

/* === Private Functions === */

function toBase64(bytes: Uint8Array): string {
  return btoa(Array.from(bytes, (byte) => String.fromCharCode(byte)).join(""));
}

/* === Entry Point === */
