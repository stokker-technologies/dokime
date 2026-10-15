/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * What the status bar says about the backend's slow operations.
 *
 * The runtime stream carries every operation the backend is performing and
 * those that settled moments ago. One running operation shows its label,
 * phase and progress; several collapse to a count named after the one that
 * has run longest. With nothing running, the last settled operation shows
 * until the backend retires it, and nothing shows once it has. The module
 * imports nothing at runtime, so a node test can load it directly.
 */

/* === Types === */

export type BackendOperationStatus = "running" | "finished" | "failed";

export type BackendOperation = {
  id: string;
  label: string;
  phase: string;
  /** From zero to one; `null` while the backend cannot tell. */
  fraction: number | null;
  startedAtMs: number | null;
  status: BackendOperationStatus;
  failure: string | null;
};

export type OperationsSummary =
  | { kind: "idle" }
  | { kind: "single"; operation: BackendOperation }
  | { kind: "many"; count: number; longest: BackendOperation; operations: BackendOperation[] };

/** The operation fields a runtime state frame carries, as generated. */
export type OperationFrame = {
  operationId: string;
  label: string;
  phase: string;
  fraction?: number;
  startedAt?: { seconds: bigint | number; nanos: number };
  status?: { HarmosVariant: { case: string | undefined } };
  failure?: string;
};

/* === Public Functions === */

/** The operations of one runtime state frame, in the order the backend sent. */
export function operationsFromFrame(
  frames: readonly OperationFrame[] | undefined,
): BackendOperation[] {
  return (frames ?? []).map((frame) => ({
    failure: frame.failure ?? null,
    fraction: fractionOf(frame.fraction),
    id: frame.operationId,
    label: frame.label,
    phase: frame.phase,
    startedAtMs: frame.startedAt
      ? Number(frame.startedAt.seconds) * 1_000 + Math.floor(frame.startedAt.nanos / 1_000_000)
      : null,
    status: statusOf(frame.status?.HarmosVariant.case),
  }));
}

/** What the status bar shows for `operations`. */
export function summarizeOperations(operations: readonly BackendOperation[]): OperationsSummary {
  const running = operations.filter((operation) => operation.status === "running");
  if (running.length > 1) {
    return {
      count: running.length,
      longest: longestRunning(running),
      kind: "many",
      operations: [...running],
    };
  }
  const [only] = running;
  if (only) return { kind: "single", operation: only };
  const settled = operations.at(-1);
  return settled ? { kind: "single", operation: settled } : { kind: "idle" };
}

/** The operation's progress as a whole percentage, or `null` when unknown. */
export function operationPercent(operation: BackendOperation): number | null {
  return operation.fraction === null ? null : Math.round(operation.fraction * 100);
}

/** The phase an operator reads beside the label. */
export function operationPhaseLabel(operation: BackendOperation): string {
  switch (operation.status) {
    case "finished":
      return "Done";
    case "failed":
      return "Failed";
    default:
      return operation.phase || "Starting";
  }
}

/** The collapsed label for several running operations. */
export function operationsCountLabel(count: number): string {
  return `${count} Operations`;
}

/* === Private Functions === */

function fractionOf(value: number | undefined): number | null {
  if (value === undefined || !Number.isFinite(value)) return null;
  return Math.min(1, Math.max(0, value));
}

function statusOf(member: string | undefined): BackendOperationStatus {
  switch (member) {
    case "Finished":
      return "finished";
    case "Failed":
      return "failed";
    default:
      return "running";
  }
}

function longestRunning(running: readonly BackendOperation[]): BackendOperation {
  return running.reduce((longest, operation) =>
    (operation.startedAtMs ?? Number.POSITIVE_INFINITY) <
    (longest.startedAtMs ?? Number.POSITIVE_INFINITY)
      ? operation
      : longest,
  );
}
