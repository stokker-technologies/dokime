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
  action?: string;
  plugin?: string;
  status?: string;
  identifiers?: Record<string, string>;
  reason?: string;
  causes?: string[];
  causeDetails?: { code: string; message: string; operation: string; plugin: string; action: string }[];
  problems?: NotificationProblem[];
  operation?: string;
  failureOperation?: string;
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
    action: details?.action ?? field("action"),
    plugin: details?.plugin ?? field("plugin"),
    reason: details?.reason ?? field("code") ?? field("kind") ?? (error instanceof Error ? error.name : undefined),
    causes: details?.causes ?? causes,
    problems: details?.problems ?? (fields.problems as NotificationProblem[] | undefined),
    operation: details?.operation ?? field("operation"),
    operationId: details?.operationId ?? field("operationId"),
    requestId: details?.requestId ?? field("requestId"),
    invocationId: details?.invocationId ?? field("invocationId"),
    diagnostics: details?.diagnostics ?? ([field("diagnostics"), field("backtrace")].filter(Boolean).join("\n\n") || undefined),
  };
  if (error && typeof error === "object") normalized.set(error, failure);
  if (key) keyed.set(key, failure);
  return failure;
}

export function isWaitingFailure(failure: NotificationFailure): boolean {
  return failure.reason === "settings_not_ready";
}

export function failureText(failure: NotificationFailure): string {
  return [
    failure.title,
    failure.message,
    ...[
      ["Operation", failure.operation],
      ["Failure Operation", failure.failureOperation],
      ["Code", failure.reason],
      ["Status", failure.status],
      ["Action", failure.action],
      ["Plugin", failure.plugin],
      ["Operation ID", failure.operationId],
      ["Request ID", failure.requestId],
      ["Invocation ID", failure.invocationId],
    ]
      .filter(([, value]) => value)
      .map(([label, value]) => `${label}: ${value}`),
    ...(failure.causeDetails?.length
      ? failure.causeDetails
          .map((cause) =>
            [
              cause.message,
              ...Object.entries(cause)
                .filter(([key, value]) => key !== "message" && value)
                .map(([key, value]) => `${key[0].toUpperCase()}${key.slice(1)}: ${value}`),
            ].join("; "),
          )
          .map((cause) => `Cause: ${cause}`)
      : (failure.causes ?? []).map((cause) => `Cause: ${cause}`)),
    ...(failure.problems ?? []).flatMap((problem) => [
      `${problem.label}: ${problem.message}`,
      ...(problem.causes ?? []).map((cause) => `Cause: ${cause}`),
    ]),
    ...Object.entries(failure.identifiers ?? {}).map(([key, value]) => `${key}: ${value}`),
    failure.diagnostics,
  ]
    .filter(Boolean)
    .join("\n");
}

export function notificationDetailsText(record: NotificationRecord, relative: string): string {
  if (record.failure) return failureText(record.failure);
  return [
    record.title,
    record.description,
    `Source: ${record.source}`,
    `Time: ${new Date(record.at).toISOString()} (${relative})`,
    `Severity: ${record.tone}`,
    ...(record.consequences ?? []).map((consequence) => `Consequence: ${consequence}`),
    record.missingSecret ? `Missing Secret: ${record.missingSecret}` : undefined,
    record.pluginSettings ? `Plugin Settings: ${record.pluginSettings}` : undefined,
  ]
    .filter(Boolean)
    .join("\n");
}

/* === Private Functions === */

/* === Entry Point === */
