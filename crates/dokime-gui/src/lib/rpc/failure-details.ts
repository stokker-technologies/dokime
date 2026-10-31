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
import type { RpcError } from "@/lib/desktop/rpc";
import type { NotificationFailure } from "@/lib/notification-details";
import { CapturedRecordErrorSchema, RejectionSchema, StatusSchema } from "./generated/errors";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function rpcFailureDetails(error: RpcError, reason: string): NotificationFailure {
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
  const failure: NotificationFailure = { reason };
  if (error.kind !== "grpc" || !error.detail.details.length) return failure;
  try {
    const status = fromBinary(StatusSchema, Uint8Array.from(error.detail.details));
    for (const detail of status.details) {
      if (detail.typeUrl === "type.googleapis.com/dokime.v1.Rejection") {
        const rejection = fromBinary(RejectionSchema, detail.value);
        failure.message = rejection.message;
        failure.reason = rejection.code || reason;
        failure.problems = rejection.issues.map((issue) => ({
          label: issue.path,
          message: issue.message,
          causes: issue.expected ? [`Expected: ${issue.expected}`] : undefined,
        }));
      } else if (detail.typeUrl === "type.googleapis.com/dokime.v1.CapturedRecordError") {
        const captured = fromBinary(CapturedRecordErrorSchema, detail.value);
        failure.message = captured.message;
        failure.reason = captured.code || failure.reason;
        failure.causes = captured.causeChain.map((cause) => (cause.code ? `${cause.code}: ${cause.message}` : cause.message)).reverse();
        const diagnostics = [
          captured.source && `Source: ${captured.source}`,
          ...captured.causeChain.filter((cause) => cause.source).map((cause) => `Cause Source: ${cause.source}`),
          captured.stacktraceAttachment && `Stacktrace Attachment: ${captured.stacktraceAttachment}`,
          captured.traceId && `Trace ID: ${captured.traceId}`,
          captured.correlationId && `Correlation ID: ${captured.correlationId}`,
          captured.diagnostics,
        ]
          .filter(Boolean)
          .join("\n");
        if (diagnostics) failure.diagnostics = [failure.diagnostics, diagnostics].filter(Boolean).join("\n\n");
      } else {
        failure.diagnostics = [failure.diagnostics, `${detail.typeUrl}\n${toBase64(detail.value)}`].filter(Boolean).join("\n\n");
      }
    }
  } catch {
    failure.diagnostics = `Unrecognized Backend Details (Base64):\n${toBase64(Uint8Array.from(error.detail.details))}`;
  }
  return failure;
}

/* === Private Functions === */

function toBase64(bytes: Uint8Array): string {
  return btoa(Array.from(bytes, (byte) => String.fromCharCode(byte)).join(""));
}

/* === Entry Point === */
