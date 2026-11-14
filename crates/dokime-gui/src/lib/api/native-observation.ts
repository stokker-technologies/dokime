/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native runtime shell, diagnostics, and observation stream adapters. */

/* === Local Imports === */

import { defaultBrand } from "@/lib/brand";
import { required, type Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import { CycleStatus } from "@host/dokime/v1/cycles_pb.js";
import { ResultStatus } from "@host/dokime/v1/execution_pb.js";
import type { EvidenceScope as HostEvidenceScope } from "@host/dokime/v1/evidence_pb.js";
import { nativeValueToJson, nativeEvidenceScope, recordScopeFromNative } from "@/lib/host/values";
export type RuntimeStateFrame = Wire<
  Awaited<ReturnType<typeof rpc.backendOverviewWatch>> extends NativeStream<infer Frame> ? Frame : never
>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import { traceabilitySnapshotFromItems } from "@/lib/traceability";
import type { ExecutionResultSummary, HubLiveState, RecordScope, RunStatus, TraceabilitySnapshot } from "@/lib/types";
import { listNativeCycles } from "./native-planning";
import { NATIVE_MAX_PAGES, NATIVE_PAGE_SIZE, NativeApiLimitError, nowIso, safeNumber, timestampIso } from "./native-values";

/* === Types === */

type RuntimeShellSnapshot = Wire<NonNullable<Awaited<ReturnType<typeof rpc.backendOverviewGet>>["overview"]>>;
interface ExecutionProgressFrame {
  progress: { testExecutionId: string; label: string; phase: string; completed: bigint; total: bigint } | null;
  finished: boolean;
}
type NativeTraceabilitySnapshot = Wire<Awaited<ReturnType<typeof rpc.evidenceTraceabilityGet>>["snapshots"][number]>;
type TraceabilityCut = Wire<NonNullable<Awaited<ReturnType<typeof rpc.evidenceTraceabilityGet>>["resolvedAt"]>>;
type EvidenceScope = Wire<HostEvidenceScope>;

/* === Runtime shell === */

export async function fetchNativeRuntimeShell(session: SessionIdentity, cycleId: string): Promise<HubLiveState> {
  const cycleSnapshots = await fetchRuntimeShellCycles(session);
  return mapRuntimeShell(cycleSnapshots, cycleId);
}

async function fetchRuntimeShellCycles(session: SessionIdentity): Promise<RuntimeShellSnapshot[]> {
  const reply = await rpc.backendOverviewGet(session, {});
  const snapshot = required(reply.overview, "Read Backend Overview");
  const cycles = await listNativeCycles(session);
  return [{ ...snapshot, cycleSummaries: cycles }];
}

export function mapRuntimeShell(snapshots: readonly RuntimeShellSnapshot[], cycleId: string): HubLiveState {
  const first = snapshots[0];
  if (!first) throw new Error("Native runtime shell returned no snapshot.");
  const cycles = snapshots.flatMap((snapshot) => snapshot.cycleSummaries);
  // The caller names the run this shell describes. There is no fallback
  // choice: a shell that cannot say which cycle it belongs to belongs to none.
  const cycle = cycles.find((candidate) => candidate.cycleId === cycleId) ?? null;
  const title = cycle?.metadata?.label || first.hubMetadata?.label || defaultBrand.label;
  const updatedAt = timestampIso(cycle?.updatedAt) ?? timestampIso(cycle?.createdAt) ?? nowIso();
  const revision = safeNumber(first.revision, "Backend.Overview.Get revision");
  return {
    run: {
      session_id: first.sessionId,
      title,
      template_id: null,
      template_slug: "",
      label: title,
      metadata: {},
      status: mapRunStatus(cycle?.status),
      started_at: timestampIso(cycle?.startedAt),
      finished_at: cycle && isTerminalCycle(cycle.status) ? updatedAt : null,
      updated_at: updatedAt,
      run_dir: null,
      local_url: null,
      lan_url: null,
      rerun_command: null,
    },
    session_id: first.sessionId,
    title,
    revision,
    records: [],
    telemetry: {},
    pending_prompts: [],
    notes: [],
    health: [],
    result_summary: cycle?.result ? mapResult(cycle.result, { test_cycle_id: cycle.cycleId }) : null,
    traceability: null,
    artifacts: [],
    installed_plugins: [],
    updated_at: updatedAt,
    revisions: revisions(revision),
  };
}

type TraceabilityResponse = Awaited<ReturnType<typeof rpc.evidenceTraceabilityGet>>;

/** Read ordered traceability snapshots at one generation-bound, scoped read cut. */
export async function fetchNativeTraceability(session: SessionIdentity, scope?: RecordScope | null): Promise<TraceabilitySnapshot> {
  const operation = "Traceability.Get";
  const requestedScope = nativeEvidenceScope(scope);
  const items: NativeTraceabilitySnapshot[] = [];
  const tokens = new Set<string>();
  let pageToken = "";
  let at: TraceabilityCut | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    assertTraceabilitySession(session, operation);
    const response = await rpc.evidenceTraceabilityGet(session, {
      ...(requestedScope ? { scope: requestedScope } : {}),
      page: { pageSize: NATIVE_PAGE_SIZE, pageToken },
      ...(at ? { at } : {}),
    });
    assertTraceabilitySession(session, operation);

    validateTraceabilityResponse(response, at, requestedScope, items.at(-1), operation);
    at ??= response.resolvedAt;
    items.push(...response.snapshots);

    const nextPageToken = response.nextPageToken;
    if (!nextPageToken) return mapNativeTraceability(items, requestedScope, operation);
    if (!response.snapshots.length) {
      throw new NativeApiLimitError(operation, "incomplete page contained no snapshots");
    }
    if (tokens.has(nextPageToken)) {
      throw new NativeApiLimitError(operation, "traceability pagination did not advance");
    }
    tokens.add(nextPageToken);
    pageToken = nextPageToken;
  }

  throw new NativeApiLimitError(operation, `more than ${NATIVE_MAX_PAGES} traceability pages`);
}

function validateTraceabilityResponse(
  response: TraceabilityResponse,
  expectedCut: TraceabilityCut | undefined,
  requestedScope: EvidenceScope | undefined,
  previous: NativeTraceabilitySnapshot | undefined,
  operation: string,
): void {
  if (!response || !Array.isArray(response.snapshots) || typeof response.nextPageToken !== "string") {
    throw new NativeApiLimitError(operation, "response omitted its page fields");
  }
  const cut = response.resolvedAt;
  if (!cut || !Array.isArray(cut.positions) || !Array.isArray(cut.revisions) || cut.revisions.length) {
    throw new NativeApiLimitError(operation, "response omitted a usable read cut");
  }
  const histories = new Set<string>();
  for (const position of cut.positions) {
    const history = position.history;
    if (!history?.store?.trim() || !history.epoch?.trim() || typeof position.sequence !== "bigint" || position.sequence < 0n) {
      throw new NativeApiLimitError(operation, "read cut omitted a usable history position");
    }
    if (histories.has(history.epoch)) {
      throw new NativeApiLimitError(operation, "read cut repeated an execution history");
    }
    histories.add(history.epoch);
  }
  if (requestedScope?.executionId && (cut.positions.length !== 1 || !histories.has(requestedScope.executionId))) {
    throw new NativeApiLimitError(operation, "read cut did not retain the requested execution");
  }
  if (expectedCut && !sameTraceabilityCut(expectedCut, cut)) {
    throw new NativeApiLimitError(operation, "response read cut changed during pagination");
  }
  for (const snapshot of response.snapshots) {
    const position = snapshot.resolvedAt;
    const bound = cut.positions.find((item) => item.history?.epoch === position?.history?.epoch);
    if (
      !position ||
      !bound ||
      position.history?.store !== bound.history?.store ||
      typeof position.sequence !== "bigint" ||
      position.sequence < 1n ||
      position.sequence > bound.sequence ||
      !snapshot.revision?.length
    ) {
      throw new NativeApiLimitError(operation, "snapshot omitted its revision or fell outside the read cut");
    }
    if (!snapshot.scope || !matchesTraceabilityScope(requestedScope, snapshot.scope)) {
      throw new NativeApiLimitError(operation, "snapshot scope did not match the requested scope");
    }
    if (snapshot.scope.executionId !== position.history?.epoch) {
      throw new NativeApiLimitError(operation, "snapshot scope did not match its history");
    }
    if (previous?.resolvedAt) {
      const earlier = previous.resolvedAt;
      const epoch = required(position.history, "Read Traceability History").epoch;
      const earlierEpoch = required(earlier.history, "Read Previous Traceability History").epoch;
      if (epoch < earlierEpoch || (epoch === earlierEpoch && position.sequence <= earlier.sequence)) {
        throw new NativeApiLimitError(operation, "snapshots were not in strict history order");
      }
    }
    previous = snapshot;
  }
}

function sameTraceabilityCut(left: TraceabilityCut, right: TraceabilityCut): boolean {
  return (
    left.positions.length === right.positions.length &&
    left.positions.every((position, index) => {
      const other = right.positions[index];
      return (
        position.history?.store === other.history?.store &&
        position.history?.epoch === other.history?.epoch &&
        position.sequence === other.sequence
      );
    })
  );
}

function matchesTraceabilityScope(wanted: EvidenceScope | undefined, actual: EvidenceScope): boolean {
  if (!wanted) return true;
  // The Host Validates the Runtime Epoch; Snapshot Scopes Carry Durable Attribution.
  const { origin, sessionId: _sessionId, ...fields } = wanted;
  const matchesFields = Object.entries(fields).every(([key, value]) => value == null || actual[key as keyof EvidenceScope] === value);
  return matchesFields && (!origin || JSON.stringify(origin.kind?.kind) === JSON.stringify(actual.origin?.kind?.kind));
}

function mapNativeTraceability(
  snapshots: NativeTraceabilitySnapshot[],
  requestedScope: EvidenceScope | undefined,
  operation: string,
): TraceabilitySnapshot {
  const items = snapshots.map((snapshot) => {
    if (!snapshot?.traceabilityId) {
      throw new NativeApiLimitError(operation, "snapshot omitted its traceability id");
    }
    if (!snapshot.createdAt) {
      throw new NativeApiLimitError(operation, "snapshot omitted its creation timestamp");
    }
    const createdAt = timestampIso(snapshot.createdAt);
    if (!createdAt) {
      throw new NativeApiLimitError(operation, "snapshot contained an invalid creation timestamp");
    }
    if (!Array.isArray(snapshot.entries)) {
      throw new NativeApiLimitError(operation, "snapshot omitted its entry collection");
    }
    const entries = snapshot.entries.map((entry) => {
      if (!entry?.metadata) {
        throw new NativeApiLimitError(operation, "traceability entry omitted its metadata");
      }
      if (!entry.value) {
        throw new NativeApiLimitError(operation, "traceability entry omitted its value");
      }
      let value: unknown;
      try {
        value = nativeValueToJson(entry.value);
      } catch (error) {
        throw new NativeApiLimitError(operation, `traceability entry contained an invalid value: ${String(error)}`);
      }
      if (value === undefined) {
        throw new NativeApiLimitError(operation, "traceability entry contained an unknown value");
      }
      return {
        metadata: {
          identifier: entry.metadata.identifier,
          label: entry.metadata.label,
          description: entry.metadata.description,
        },
        value,
        source: entry.source,
        unit: entry.unit,
      };
    });
    return {
      traceability_id: snapshot.traceabilityId,
      scope: snapshot.scope,
      metadata: snapshot.metadata
        ? {
            identifier: snapshot.metadata.identifier,
            label: snapshot.metadata.label,
            description: snapshot.metadata.description,
          }
        : null,
      entries,
      created_at: createdAt,
    };
  });
  const result = traceabilitySnapshotFromItems(items, (value) => recordScopeFromNative(value ?? undefined));
  if (requestedScope) {
    result.scope = recordScopeFromNative(requestedScope);
  }
  return result;
}

function assertTraceabilitySession(session: SessionIdentity, operation: string): void {
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

export interface NativeObservationSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

export async function openNativeRuntimeStream(
  session: NativeObservationSession,
  at?: Parameters<typeof rpc.backendOverviewWatch>[1]["at"],
): Promise<NativeStream<RuntimeStateFrame>> {
  const reader = await rpc.backendOverviewWatch(session, { at });
  let settled = false;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = (): Promise<void> => {
    cancelPromise ??= reader.cancel();
    return cancelPromise;
  };
  const release = session.own(() => {
    if (!settled) void cancelReader().catch(() => undefined);
  });
  let released = false;
  const releaseOnce = () => {
    if (!released) {
      released = true;
      release();
    }
  };
  const forward = (async function* () {
    let exhausted = false;
    try {
      for await (const frame of reader) {
        if (frame.resyncRequired) throw new Error(`Watch Backend Overview: ${frame.resyncRequired.message}; Refresh the Overview`);
        yield frame;
      }
      exhausted = true;
    } finally {
      if (!exhausted) await cancelReader().catch(() => undefined);
      settled = true;
      releaseOnce();
    }
  })();
  return {
    cancel: async () => {
      if (settled) return;
      await cancelReader();
      settled = true;
      releaseOnce();
    },
    [Symbol.asyncIterator]: () => forward,
  };
}

export async function openNativeExecutionProgressStream(
  session: NativeObservationSession,
  testExecutionId: string,
): Promise<NativeStream<ExecutionProgressFrame>> {
  const reader = await rpc.executionsProgressWatch(session, { name: testExecutionId });
  let settled = false;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = (): Promise<void> => {
    cancelPromise ??= reader.cancel();
    return cancelPromise;
  };
  const release = session.own(() => {
    if (!settled) void cancelReader().catch(() => undefined);
  });
  let released = false;
  const releaseOnce = () => {
    if (!released) {
      released = true;
      release();
    }
  };
  const forward = (async function* () {
    let exhausted = false;
    try {
      for await (const frame of reader)
        yield {
          progress:
            frame.value && !frame.finished
              ? {
                  testExecutionId,
                  label: frame.value.label,
                  phase: frame.phase,
                  completed: frame.value.completed,
                  total: frame.value.total,
                }
              : null,
          finished: frame.finished,
        };
      exhausted = true;
    } finally {
      if (!exhausted) await cancelReader().catch(() => undefined);
      settled = true;
      releaseOnce();
    }
  })();
  return {
    cancel: async () => {
      if (settled) return;
      await cancelReader();
      settled = true;
      releaseOnce();
    },
    [Symbol.asyncIterator]: () => forward,
  };
}

/* === Private mapping === */

function mapRunStatus(status?: CycleStatus): RunStatus {
  switch (status) {
    case CycleStatus.RUNNING:
      return "running";
    case CycleStatus.PAUSED:
      return "paused";
    case CycleStatus.STOPPING:
      return "stopping";
    case CycleStatus.COMPLETED:
      return "completed";
    case CycleStatus.FAILED:
      return "failed";
    case CycleStatus.STOPPED:
      return "stopped";
    default:
      return "idle";
  }
}

function isTerminalCycle(status: CycleStatus): boolean {
  return [CycleStatus.COMPLETED, CycleStatus.FAILED, CycleStatus.BLOCKED, CycleStatus.STOPPED].includes(status);
}

function mapResult(
  result: NonNullable<RuntimeShellSnapshot["cycleSummaries"][number]["result"]>,
  scope: RecordScope,
): ExecutionResultSummary {
  return {
    outcome:
      result.status === ResultStatus.PASSED
        ? "completed"
        : result.status === ResultStatus.FAILED
          ? "failed"
          : result.status === ResultStatus.BLOCKED
            ? "unknown"
            : "unknown",
    headline: result.status === ResultStatus.PASSED ? "passed" : "failed",
    warnings: [],
    evidence: [
      {
        passed: result.passed,
        failed: result.failed,
        blocked: result.blocked,
        skipped: result.skipped,
      },
    ],
    updated_at: nowIso(),
    scope,
  };
}

function revisions(revision: number): HubLiveState["revisions"] {
  return {
    run: revision,
    prompts: revision,
    cycles: revision,
    document: revision,
    records: revision,
    artifacts: revision,
    executionFiles: revision,
    notes: revision,
    health: revision,
    result_summary: revision,
    traceability: revision,
    contracts: revision,
    telemetry: { __all__: revision },
  };
}
