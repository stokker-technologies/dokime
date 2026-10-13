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
import * as rpc from "@/lib/rpc/generated/client";
import type { Wire } from "@/lib/rpc/canonical";
export type RuntimeStateFrame = Wire<
  Awaited<ReturnType<typeof rpc.observationWatchRuntime>> extends NativeStream<infer Frame>
    ? Frame
    : never
>;
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import { traceabilitySnapshotFromItems } from "@/lib/traceability";
import type {
  ExecutionResultSummary,
  HubLiveState,
  RecordScope,
  RunStatus,
  TraceabilitySnapshot,
} from "@/lib/types";
import type { DiagnosticsResponse, DiagnosticsSnapshot } from "@/lib/types/view-contracts";
import {
  enumTail,
  mapNativeSeverity,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nativeEvidenceScope,
  nativeValueToJson,
  nowIso,
  recordScopeFromNative,
  safeNumber,
  timestampIso,
} from "./native-values";

/* === Types === */

type RuntimeShellSnapshot = Wire<Awaited<ReturnType<typeof rpc.observationGetRuntimeShell>>>;
type ExecutionProgressFrame = Wire<
  Awaited<ReturnType<typeof rpc.observationWatchExecutionProgress>> extends NativeStream<
    infer Frame
  >
    ? Frame
    : never
>;
type NativeTraceabilitySnapshot = Wire<
  Awaited<ReturnType<typeof rpc.observationGetTraceability>>["snapshots"][number]
>;
type Cursor = NonNullable<Parameters<typeof rpc.observationGetTraceability>[1]["cursor"]>;
type EvidenceScope = Wire<
  NonNullable<Parameters<typeof rpc.observationGetTraceability>[1]["scope"]>
>;
type NativeDiagnosticsResponse = Wire<Awaited<ReturnType<typeof rpc.backendGetDiagnostics>>>;
type DiagnosticRuntime = NonNullable<NativeDiagnosticsResponse["runtime"]>;
type DiagnosticCheckStatus = NonNullable<DiagnosticRuntime["startupDoctor"]>["status"];
type StartupDoctorCheckKey = NonNullable<
  DiagnosticRuntime["startupDoctor"]
>["checks"][number]["key"];
type DiagnosticPathKey = DiagnosticRuntime["pathConventions"][number]["key"];
type DiagnosticPathScope = DiagnosticRuntime["pathConventions"][number]["scope"];
type DiagnosticFileFormat = DiagnosticRuntime["pathConventions"][number]["format"];
type InstalledPluginPhase = DiagnosticRuntime["installedPluginErrors"][number]["phase"];
type CycleStatus = RuntimeShellSnapshot["cycleSummaries"][number]["status"];

/* === Runtime shell === */

export async function fetchNativeRuntimeShell(
  session: SessionIdentity,
  cycleId: string,
): Promise<HubLiveState> {
  const cycleSnapshots = await fetchRuntimeShellCycles(session);
  return mapRuntimeShell(cycleSnapshots, cycleId);
}

async function fetchRuntimeShellCycles(session: SessionIdentity): Promise<RuntimeShellSnapshot[]> {
  const snapshots: RuntimeShellSnapshot[] = [];
  let afterCycleId = "";
  let cursor: RuntimeShellSnapshot["observedCursor"];
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const snapshot = await rpc.observationGetRuntimeShell(session, {
      limit: 100,
      afterCycleId,
      ...(cursor ? { cursor } : {}),
    });
    snapshots.push(snapshot);
    const nextAfterCycleId = snapshot.nextAfterCycleId ?? "";
    if (!nextAfterCycleId) return snapshots;
    if (nextAfterCycleId === afterCycleId) {
      throw new NativeApiLimitError(
        "Observation.GetRuntimeShell",
        "cycle pagination did not advance",
      );
    }
    afterCycleId = nextAfterCycleId;
    cursor = snapshot.observedCursor;
    if (!cursor) {
      throw new NativeApiLimitError("Observation.GetRuntimeShell", "cycle page omitted its cursor");
    }
  }
  throw new NativeApiLimitError(
    "Observation.GetRuntimeShell",
    `more than ${NATIVE_MAX_PAGES} cycle pages`,
  );
}

export function mapRuntimeShell(
  snapshots: readonly RuntimeShellSnapshot[],
  cycleId: string,
): HubLiveState {
  const first = snapshots[0];
  if (!first) throw new Error("Native runtime shell returned no snapshot.");
  const cycles = snapshots.flatMap((snapshot) => snapshot.cycleSummaries);
  // The caller names the run this shell describes. There is no fallback
  // choice: a shell that cannot say which cycle it belongs to belongs to none.
  const cycle = cycles.find((candidate) => candidate.cycleId === cycleId) ?? null;
  const title = cycle?.metadata?.label || first.hubMetadata?.label || defaultBrand.label;
  const updatedAt = timestampIso(cycle?.updatedAt) ?? timestampIso(cycle?.createdAt) ?? nowIso();
  const revision = safeNumber(first.revision, "Observation.RuntimeShell revision");
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
    result_summary: cycle?.result
      ? mapResult(cycle.result, { test_cycle_id: cycle.cycleId })
      : null,
    traceability: null,
    artifacts: [],
    installed_plugins: [],
    updated_at: updatedAt,
    revisions: revisions(revision),
  };
}

type NativeTraceabilityPage = Awaited<ReturnType<typeof rpc.observationGetTraceability>>;

/** Read the current traceability projection through exact, generation-bound pages. */
export async function fetchNativeTraceability(
  session: SessionIdentity,
  scope?: RecordScope | null,
): Promise<TraceabilitySnapshot> {
  const operation = "Observation.GetTraceability";
  const requestedScope = nativeEvidenceScope(scope);
  const items: NativeTraceabilitySnapshot[] = [];
  let afterTraceabilityId = "";
  let cursor: Cursor | undefined;
  let hostTraceabilityFingerprint: string | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    assertTraceabilitySession(session, operation);
    const response = await rpc.observationGetTraceability(session, {
      ...(requestedScope ? { scope: requestedScope } : {}),
      limit: NATIVE_PAGE_SIZE,
      afterTraceabilityId,
      ...(cursor ? { cursor } : {}),
      ...(hostTraceabilityFingerprint ? { hostTraceabilityFingerprint } : {}),
    });
    assertTraceabilitySession(session, operation);

    validateTraceabilityPage(
      response,
      requestedScope,
      cursor,
      hostTraceabilityFingerprint,
      operation,
    );
    cursor ??= response.cursor;
    hostTraceabilityFingerprint ??= response.hostTraceabilityFingerprint;
    items.push(...response.snapshots);

    const nextAfterTraceabilityId = response.nextAfterTraceabilityId;
    if (response.complete) {
      if (nextAfterTraceabilityId) {
        throw new NativeApiLimitError(operation, "complete page included a continuation marker");
      }
      return mapNativeTraceability(items, response.scope, requestedScope, operation);
    }
    if (!nextAfterTraceabilityId) {
      throw new NativeApiLimitError(operation, "incomplete page omitted its continuation marker");
    }
    if (!response.snapshots.length) {
      throw new NativeApiLimitError(operation, "incomplete page contained no snapshots");
    }
    if (nextAfterTraceabilityId === afterTraceabilityId) {
      throw new NativeApiLimitError(operation, "traceability pagination did not advance");
    }
    afterTraceabilityId = nextAfterTraceabilityId;
  }

  throw new NativeApiLimitError(operation, `more than ${NATIVE_MAX_PAGES} traceability pages`);
}

function validateTraceabilityPage(
  response: NativeTraceabilityPage,
  requestedScope: EvidenceScope | undefined,
  expectedCursor: Cursor | undefined,
  expectedHostTraceabilityFingerprint: string | undefined,
  operation: string,
): void {
  if (
    !response ||
    !Array.isArray(response.snapshots) ||
    typeof response.complete !== "boolean" ||
    typeof response.nextAfterTraceabilityId !== "string"
  ) {
    throw new NativeApiLimitError(operation, "response omitted its page fields");
  }
  if (
    !response.cursor ||
    typeof response.cursor.epoch !== "string" ||
    typeof response.cursor.stream !== "string" ||
    !response.cursor.epoch.trim() ||
    !response.cursor.stream.trim() ||
    typeof response.cursor.position !== "bigint"
  ) {
    throw new NativeApiLimitError(operation, "response omitted a usable cursor");
  }
  if (expectedCursor && !sameNativeCursor(expectedCursor, response.cursor)) {
    throw new NativeApiLimitError(operation, "response cursor changed during pagination");
  }
  if (
    typeof response.hostTraceabilityFingerprint !== "string" ||
    !response.hostTraceabilityFingerprint.trim()
  ) {
    throw new NativeApiLimitError(operation, "response omitted its host traceability fingerprint");
  }
  if (
    expectedHostTraceabilityFingerprint &&
    expectedHostTraceabilityFingerprint !== response.hostTraceabilityFingerprint
  ) {
    throw new NativeApiLimitError(
      operation,
      "response host traceability fingerprint changed during pagination",
    );
  }
  if (!sameNativeScope(requestedScope, response.scope)) {
    throw new NativeApiLimitError(operation, "response scope did not match the requested scope");
  }
}

function mapNativeTraceability(
  snapshots: NativeTraceabilitySnapshot[],
  responseScope: EvidenceScope | undefined,
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
      const value = nativeValueToJson(entry.value);
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
  const result = traceabilitySnapshotFromItems(items, (value) =>
    recordScopeFromNative(value ?? undefined),
  );
  if (responseScope || requestedScope) {
    result.scope = recordScopeFromNative(responseScope ?? requestedScope);
  }
  return result;
}

function sameNativeCursor(left: Cursor | undefined, right: Cursor | undefined): boolean {
  return Boolean(
    left &&
      right &&
      left.epoch === right.epoch &&
      left.stream === right.stream &&
      left.position === right.position,
  );
}

function sameNativeScope(
  left: EvidenceScope | undefined,
  right: EvidenceScope | undefined,
): boolean {
  if (!left || !right) return left === right;
  return (
    (left.sessionId ?? null) === (right.sessionId ?? null) &&
    (left.cycleId ?? null) === (right.cycleId ?? null) &&
    (left.caseId ?? null) === (right.caseId ?? null) &&
    (left.executionId ?? null) === (right.executionId ?? null) &&
    (left.stepId ?? null) === (right.stepId ?? null) &&
    (left.criterionId ?? null) === (right.criterionId ?? null) &&
    (left.templateId ?? null) === (right.templateId ?? null) &&
    (left.suiteId ?? null) === (right.suiteId ?? null)
  );
}

function assertTraceabilitySession(session: SessionIdentity, operation: string): void {
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

export interface NativeObservationSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

export async function openNativeRuntimeStream(
  session: NativeObservationSession,
  cursor?: Parameters<typeof rpc.observationWatchRuntime>[1]["cursor"],
): Promise<NativeStream<RuntimeStateFrame>> {
  const reader = await rpc.observationWatchRuntime(session, { cursor });
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
      for await (const frame of reader) yield frame;
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
  const reader = await rpc.observationWatchExecutionProgress(session, { testExecutionId });
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
      for await (const frame of reader) yield frame;
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

/* === Diagnostics === */

export async function fetchNativeDiagnostics(
  session: SessionIdentity,
): Promise<DiagnosticsResponse> {
  const response: NativeDiagnosticsResponse = await rpc.backendGetDiagnostics(session, {});
  if (!response.server || !response.runtime) throw new Error("Native diagnostics are incomplete.");
  return mapDiagnostics(response);
}

function mapDiagnostics(response: NativeDiagnosticsResponse): DiagnosticsResponse {
  const server = response.server;
  const runtime = response.runtime;
  if (!server || !runtime) throw new Error("Native diagnostics are incomplete.");
  const capturedAt = timestampIso(runtime.capturedAt) ?? nowIso();
  const node = runtime.identity?.node;
  if (!node?.nodeId) throw new Error("Native diagnostics have no runtime node identity.");
  const createdAt = timestampIso(node.createdAt) ?? capturedAt;
  const updatedAt = timestampIso(node.updatedAt) ?? capturedAt;
  const snapshot: DiagnosticsSnapshot = {
    captured_at: capturedAt,
    session_id: runtime.sessionId,
    identity: {
      node: {
        node_id: node.nodeId,
        hostname_label: node.hostnameLabel ?? null,
        created_at: createdAt,
        updated_at: updatedAt,
      },
      actor: runtime.identity?.actor
        ? {
            actor_id: runtime.identity.actor.actorId,
            display_name: runtime.identity.actor.displayName,
          }
        : null,
    },
    revision: safeNumber(runtime.revision, "Backend.Diagnostics revision"),
    shutting_down: runtime.shuttingDown,
    storage_root: runtime.storageRoot,
    hub_metadata: {
      identifier: runtime.hubMetadata?.identifier ?? "hub",
      label: runtime.hubMetadata?.label ?? defaultBrand.label,
      description: runtime.hubMetadata?.description ?? null,
    },
    startup_doctor: {
      checked_at: timestampIso(runtime.startupDoctor?.checkedAt) ?? capturedAt,
      status: mapDiagnosticStatus(runtime.startupDoctor?.status),
      checks: (runtime.startupDoctor?.checks ?? []).map((check) => ({
        key: mapStartupKey(check.key),
        status: mapDiagnosticStatus(check.status),
        message: check.message,
        detail: check.detail ?? null,
      })),
    },
    path_conventions: runtime.pathConventions.map((item) => ({
      key: mapPathKey(item.key),
      scope: mapPathScope(item.scope),
      format: mapPathFormat(item.format),
      relative_path: item.relativePath,
      absolute_path: item.absolutePath ?? null,
    })),
    counts: {
      test_suites: safeNumber(runtime.counts?.testSuites ?? 0n, "diagnostic suite count"),
      cycles: safeNumber(runtime.counts?.cycles ?? 0n, "diagnostic cycle count"),
      installed_plugin_errors: safeNumber(
        runtime.counts?.installedPluginErrors ?? 0n,
        "diagnostic artifact error count",
      ),
      state_cycles: safeNumber(runtime.counts?.stateCycles ?? 0n, "diagnostic state cycle count"),
      state_executions_total: safeNumber(
        runtime.counts?.stateExecutionsTotal ?? 0n,
        "diagnostic execution count",
      ),
      state_record_count: safeNumber(
        runtime.counts?.stateRecordCount ?? 0n,
        "diagnostic record count",
      ),
      telemetry_cursor_entries: safeNumber(
        runtime.counts?.telemetryCursorEntries ?? 0n,
        "diagnostic telemetry cursor count",
      ),
    },
    installed_plugin_errors: runtime.installedPluginErrors.map((item) => ({
      plugin_id: item.pluginId,
      contribution_id: item.contributionId ?? null,
      phase: mapArtifactPhase(item.phase),
      level: mapNativeSeverity(item.level),
      message: item.message,
      code: item.code ?? null,
      source: item.source ?? null,
      cause_chain: item.causeChain.map((cause) => ({
        message: cause.message,
        code: cause.code ?? null,
        source: cause.source ?? null,
      })),
      stacktrace_attachment: item.stacktraceAttachment ?? null,
      trace_id: item.traceId ?? null,
      scope: recordScopeFromNative(item.scope),
      session_id: item.sessionId ?? null,
      test_cycle_id: item.testCycleId ?? null,
      test_execution_id: item.testExecutionId ?? null,
      recorded_at: timestampIso(item.recordedAt) ?? capturedAt,
    })),
  };
  return {
    runtime: snapshot,
    server: {
      bind_addr: "native",
      body_limit_bytes: 0,
      auth: "native",
      allowed_origins: [],
      heartbeat_interval_ms: 0,
      version: server.version,
      process: {
        pid: server.process?.pid ?? 0,
        cpu_percent: server.process?.cpuPercent ?? null,
        memory_bytes:
          server.process?.memoryBytes === undefined
            ? null
            : safeNumber(server.process.memoryBytes, "diagnostic memory bytes"),
        virtual_memory_bytes:
          server.process?.virtualMemoryBytes === undefined
            ? null
            : safeNumber(server.process.virtualMemoryBytes, "diagnostic virtual memory bytes"),
      },
    },
  };
}

/* === Private mapping === */

function mapRunStatus(status?: CycleStatus): RunStatus {
  switch (status?.HarmosVariant.case) {
    case "Running":
      return "running";
    case "Paused":
      return "paused";
    case "Stopping":
      return "stopping";
    case "Completed":
      return "completed";
    case "Failed":
      return "failed";
    case "Stopped":
      return "stopped";
    default:
      return "idle";
  }
}

function isTerminalCycle(status: CycleStatus): boolean {
  return ["Completed", "Failed", "Blocked", "Stopped", "Cleared"].includes(
    status?.HarmosVariant.case ?? "",
  );
}

function mapResult(
  result: NonNullable<RuntimeShellSnapshot["cycleSummaries"][number]["result"]>,
  scope: RecordScope,
): ExecutionResultSummary {
  return {
    outcome:
      result.status?.HarmosVariant.case === "Passed"
        ? "completed"
        : result.status?.HarmosVariant.case === "Failed"
          ? "failed"
          : result.status?.HarmosVariant.case === "Blocked"
            ? "unknown"
            : "unknown",
    headline: result.status?.HarmosVariant.case === "Passed" ? "passed" : "failed",
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

function mapDiagnosticStatus(value?: DiagnosticCheckStatus): "ok" | "warning" | "failed" {
  return enumTail(undefined, value, "ok");
}

function mapStartupKey(value: StartupDoctorCheckKey): "storage_root" | "shutdown_state" {
  return enumTail(undefined, value, "storage_root");
}

function mapPathKey(
  value: DiagnosticPathKey,
):
  | "hub_log"
  | "startup_doctor"
  | "diagnostics_snapshot"
  | "collect_bundle"
  | "cycle_log_pattern"
  | "execution_log_pattern" {
  return enumTail(undefined, value, "hub_log");
}

function mapPathScope(value: DiagnosticPathScope): "root" | "cycle_pattern" | "execution_pattern" {
  return enumTail(undefined, value, "root");
}

function mapPathFormat(value: DiagnosticFileFormat): "json" | "jsonl" | "zip" {
  return enumTail(undefined, value, "json");
}

function mapArtifactPhase(
  value: InstalledPluginPhase,
):
  | "entrypoint"
  | "action"
  | "sidecar"
  | "provider"
  | "initialize"
  | "run"
  | "finalize"
  | "interrupt" {
  return enumTail(undefined, value, "entrypoint");
}
