/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Readable Serializable Notification Failure Context
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { NotificationRecord } from "@/lib/notifications";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NotificationProblem = { label: string; message: string; causes?: string[] };

/** GUI projection; native adapters own decoding their canonical error contracts. */
export type NotificationFailure = {
  message?: string;
  reason?: string;
  causes?: string[];
  problems?: NotificationProblem[];
  operation?: string;
  operationId?: string;
  requestId?: string;
  invocationId?: string;
  diagnostics?: string;
};

/* === Classes === */

/* === Public Functions === */

export function notificationMessage(error: unknown, fallback = "The operation could not be completed."): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") return error.message;
  return fallback;
}

export function notificationFailure(error: unknown): NotificationFailure | undefined {
  if (!error || typeof error !== "object") return undefined;
  if ("notificationFailure" in error) return error.notificationFailure as NotificationFailure;
  const causes: string[] = [];
  const seen = new Set<unknown>([error]);
  let cause: unknown = "cause" in error ? error.cause : undefined;
  while (cause && !seen.has(cause)) {
    seen.add(cause);
    if (cause instanceof Error) causes.unshift(cause.message);
    else if (typeof cause === "string") causes.unshift(cause);
    cause = typeof cause === "object" && "cause" in cause ? cause.cause : undefined;
  }
  const fields = error as Record<string, unknown>;
  const field = (key: string): string | undefined => (typeof fields[key] === "string" ? (fields[key] as string) : undefined);
  return {
    reason: field("code") ?? field("kind") ?? (error instanceof Error ? error.name : undefined),
    causes,
    operation: field("operation"),
    operationId: field("operationId"),
    requestId: field("requestId"),
    invocationId: field("invocationId"),
    diagnostics: [field("diagnostics"), field("backtrace")].filter(Boolean).join("\n\n") || undefined,
  };
}

export function notificationDetailsText(record: NotificationRecord, relative: string): string {
  const failure = record.failure;
  const lines = [
    record.title,
    record.description,
    `Source: ${record.source}`,
    `Time: ${new Date(record.at).toISOString()} (${relative})`,
    `Severity: ${record.tone}`,
  ];
  if (failure) {
    if (failure.message && failure.message !== record.description) lines.push(failure.message);
    for (const [label, value] of [
      ["Reason", failure.reason],
      ["Operation", failure.operation],
      ["Operation ID", failure.operationId],
      ["Request ID", failure.requestId],
      ["Invocation ID", failure.invocationId],
    ]) {
      if (value) lines.push(`${label}: ${value}`);
    }
    for (const cause of failure.causes ?? []) lines.push(`Cause: ${cause}`);
    for (const problem of failure.problems ?? []) {
      lines.push(`${problem.label}: ${problem.message}`);
      for (const cause of problem.causes ?? []) lines.push(`Cause: ${cause}`);
    }
  }
  for (const consequence of record.consequences ?? []) lines.push(`Consequence: ${consequence}`);
  if (record.missingSecret) lines.push(`Missing Secret: ${record.missingSecret}`);
  if (record.pluginSettings) lines.push(`Plugin Settings: ${record.pluginSettings}`);
  if (failure?.diagnostics) lines.push(`Technical Details:\n${failure.diagnostics}`);
  return lines.filter((line) => line !== undefined && line !== "").join("\n");
}

/* === Private Functions === */

/* === Entry Point === */
