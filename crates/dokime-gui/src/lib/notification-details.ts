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
import { connectionError } from "@/app/native-session/errors";
import type { NotificationRecord } from "@/lib/notifications";

/* === Re-Exports === */

/* === Constants === */

const normalized = new WeakMap<object, NotificationFailure>();
const keyed = new Map<string, NotificationFailure>();

/* === Types === */

export type NotificationProblem = { label: string; message: string; causes?: string[] };

/** GUI projection; native adapters own decoding their canonical error contracts. */
export type NotificationFailure = {
  id: string;
  title: string;
  message: string;
  stack?: string;
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

/** Normalize once per error occurrence; explicit keys identify replayed backend events. */
export function failureOf(error: unknown, title: string, key?: string): NotificationFailure {
  if (error && typeof error === "object") {
    if (
      "id" in error &&
      typeof error.id === "string" &&
      "title" in error &&
      typeof error.title === "string" &&
      "message" in error &&
      typeof error.message === "string"
    )
      return error as NotificationFailure;
    const cached = normalized.get(error);
    if (cached) return cached;
  }
  const replay = key ? keyed.get(key) : undefined;
  if (replay) return replay;
  const fields = error && typeof error === "object" ? (error as Record<string, unknown>) : {};
  const details = fields.notificationFailure as Partial<NotificationFailure> | undefined;
  const field = (name: string) => (typeof fields[name] === "string" ? (fields[name] as string) : undefined);
  const causes: string[] = [];
  const seen = new Set<unknown>([error]);
  let cause = fields.cause;
  while (cause && !seen.has(cause)) {
    seen.add(cause);
    if (typeof cause === "string") causes.unshift(cause);
    else if (typeof cause === "object" && "message" in cause && typeof cause.message === "string") causes.unshift(cause.message);
    cause = typeof cause === "object" && "cause" in cause ? cause.cause : undefined;
  }
  const nativeDetail = fields.detail && typeof fields.detail === "object" ? (fields.detail as Record<string, unknown>) : undefined;
  const suppliedMessage = details?.message ?? field("message");
  const message =
    (suppliedMessage?.trim() ? suppliedMessage : undefined) ||
    (fields.kind === "grpc" && typeof nativeDetail?.message === "string" ? nativeDetail.message : undefined) ||
    (typeof error === "string" && error.trim()
      ? error
      : fields.kind === "conflict"
        ? "Settings changed elsewhere. Retry to review the latest settings before editing again."
        : fields.kind === "disconnected"
          ? "The backend is disconnected. Local preferences remain available."
          : "kind" in fields
            ? connectionError(error)
            : "The operation could not be completed.");
  const failure: NotificationFailure = {
    ...details,
    id: key ?? crypto.randomUUID(),
    title,
    message,
    reason: details?.reason ?? field("code") ?? field("kind") ?? (error instanceof Error ? error.name : undefined),
    causes: details?.causes ?? causes,
    problems: details?.problems ?? (fields.problems as NotificationProblem[] | undefined),
    operation: details?.operation ?? field("operation"),
    operationId: details?.operationId ?? field("operationId"),
    requestId: details?.requestId ?? field("requestId"),
    invocationId: details?.invocationId ?? field("invocationId"),
    diagnostics: details?.diagnostics ?? ([field("diagnostics"), field("backtrace")].filter(Boolean).join("\n\n") || undefined),
    stack: error instanceof Error ? error.stack : details?.stack,
  };
  if (error && typeof error === "object") normalized.set(error, failure);
  if (key) keyed.set(key, failure);
  return failure;
}

export function failureText(failure: NotificationFailure): string {
  return JSON.stringify(failure, null, 2);
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
  if (failure) lines.push(failureText(failure));
  if (failure?.stack) lines.push(`Client Stack:\n${failure.stack}`);
  return lines.filter((line) => line !== undefined && line !== "").join("\n");
}

/* === Private Functions === */

/* === Entry Point === */
