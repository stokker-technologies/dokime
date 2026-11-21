/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { CycleSummarySchema, ExecutionOutcomeSchema, ExecutionState, ExecutionSummarySchema, Termination } from "@host/model";
import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { test, vi } from "vitest";

// Local Imports
import {
  createEvidenceRefresh,
  createRuntimeProjectionRefreshCoordinator,
  createThrottledRefresh,
  runtimeStateOf,
  settledOf,
} from "@/app/live/runtime-refresh";
import { invalidateWorkflowDocuments, workflowQueryKeys } from "@/features/cycle/server-state";
import { executionControlState } from "@/features/cycle/control-model";
import type { RuntimeStateFrame } from "@/lib/api/native-observation";
import { mapNativeExecutionSummary, projectExecutionStatus } from "@dokime/document/lib/api/native-planning-projection";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Runtime Refresh Preserves a Trailing Observation", async () => {
  let finish!: () => void;
  const gate = new Promise<void>((resolve) => {
    finish = resolve;
  });
  let calls = 0;
  const coordinator = createRuntimeProjectionRefreshCoordinator(async () => {
    calls += 1;
    if (calls === 1) {
      await gate;
    }
  });
  const first = coordinator.request();
  assert.equal(coordinator.request(), first);
  await Promise.resolve();
  assert.equal(calls, 1);
  assert.equal(coordinator.request(), first);
  assert.equal(calls, 1);
  finish();
  await first;
  assert.equal(calls, 2);

  await coordinator.request({ ensureTrailing: true });
  assert.equal(calls, 4);
  coordinator.dispose();
  await coordinator.request();
  assert.equal(calls, 4);
});

test("Runtime Refresh Can Retry After Failure", async () => {
  const refresh = vi.fn().mockRejectedValueOnce(new Error("Disconnected")).mockResolvedValue(undefined);
  const coordinator = createRuntimeProjectionRefreshCoordinator(refresh);
  await assert.rejects(coordinator.request(), /Disconnected/);
  await coordinator.request();
  assert.equal(refresh.mock.calls.length, 2);
  coordinator.dispose();
});

test("Document Refresh Is Demand Driven and Bounded", () => {
  vi.useFakeTimers();
  const refresh = vi.fn();
  const coordinator = createThrottledRefresh(refresh, 5_000);
  try {
    vi.advanceTimersByTime(30_000);
    assert.equal(refresh.mock.calls.length, 0);
    coordinator.request();
    vi.advanceTimersByTime(4_999);
    coordinator.request();
    assert.equal(refresh.mock.calls.length, 0);
    vi.advanceTimersByTime(1);
    assert.equal(refresh.mock.calls.length, 1);
    vi.advanceTimersByTime(30_000);
    assert.equal(refresh.mock.calls.length, 1);
    coordinator.request();
    coordinator.dispose();
    vi.advanceTimersByTime(5_000);
    assert.equal(refresh.mock.calls.length, 1);
  } finally {
    coordinator.dispose();
    vi.useRealTimers();
  }
});

test("A Settled Run Refreshes Its Documents at Once", () => {
  vi.useFakeTimers();
  const refresh = vi.fn();
  const coordinator = createThrottledRefresh(refresh, 5_000);
  const frame = (running: bigint, passed: number, telemetry: bigint, sealed: boolean) =>
    ({
      overview: {
        cycleSummaries: [create(CycleSummarySchema, { cycleId: "cycle", runningCount: running, result: { passed }, sealed })],
        evidenceCounts: { telemetry },
      },
    }) as unknown as RuntimeStateFrame;
  try {
    assert.equal(settledOf(frame(2n, 0, 10n, false)), settledOf(frame(1n, 1, 20n, false)));
    assert.notEqual(settledOf(frame(1n, 1, 20n, false)), settledOf(frame(0n, 2, 20n, false)));
    assert.notEqual(settledOf(frame(0n, 2, 20n, false)), settledOf(frame(0n, 2, 20n, true)));
    coordinator.request();
    coordinator.now();
    assert.equal(refresh.mock.calls.length, 1);
    vi.advanceTimersByTime(5_000);
    assert.equal(refresh.mock.calls.length, 1);
  } finally {
    coordinator.dispose();
    vi.useRealTimers();
  }
});

test("Durable Execution State Is Independent of Its Cycle", () => {
  const outcome = create(ExecutionOutcomeSchema, { attribution: { case: "notEvaluated", value: { termination: Termination.COMPLETED } } });
  assert.equal(projectExecutionStatus(ExecutionState.OPEN), "planned");
  assert.equal(projectExecutionStatus(ExecutionState.SEALED, outcome), "skipped");
  const attempt = create(ExecutionSummarySchema, { executionId: "unstarted", caseId: "case", ordinal: 1n, state: ExecutionState.OPEN });
  const cycle = create(CycleSummarySchema, { cycleId: "cycle", sealed: true });
  const projected = mapNativeExecutionSummary(attempt, [], cycle);
  assert.equal(projected.status, "planned");
  assert.equal(projected.sealed, false);
});

test("Not-Run Executions Retain Their Sealed State and Outcome", () => {
  const outcome = create(ExecutionOutcomeSchema, { attribution: { case: "notEvaluated", value: { termination: Termination.NOT_RUN } } });
  const attempt = create(ExecutionSummarySchema, {
    executionId: "unstarted",
    caseId: "case",
    ordinal: 1n,
    state: ExecutionState.SEALED,
    outcome,
    controls: { canStart: false },
  });
  const cycle = create(CycleSummarySchema, { cycleId: "cycle", sealed: true });
  const execution = mapNativeExecutionSummary(attempt, [], cycle);
  assert.equal(execution.status, "not_run");
  assert.equal(execution.sealed, true);
  assert.equal(execution.controls?.canStart, false);
  assert.equal(
    executionControlState({ ...execution, controls: execution.controls, progress: null, promptWaiting: false, stopping: false }).phase,
    "sealed",
  );
});

test("Planned Execution Retains Host Start Capability", () => {
  for (const canStart of [true, false]) {
    const attempt = create(ExecutionSummarySchema, {
      executionId: "unstarted",
      caseId: "case",
      ordinal: 1n,
      state: ExecutionState.OPEN,
      controls: { canStart },
    });
    const cycle = create(CycleSummarySchema, { cycleId: "cycle" });
    const execution = mapNativeExecutionSummary(attempt, [], cycle);
    assert.equal(execution.controls?.canStart, canStart);
    const state = executionControlState({
      ...execution,
      controls: execution.controls,
      progress: null,
      promptWaiting: false,
      stopping: false,
    });
    assert.equal(state.actions.includes("Start"), canStart);
    assert.equal(state.detail !== null, !canStart);
  }
});

test("Runtime State Is Independent of Evidence Growth", () => {
  const frame = (samples: bigint, state: number, epoch = "epoch") =>
    ({
      overview: {
        observedCursor: {
          positions: [],
          revisions: [{ name: `runtime/${epoch}`, revision: Uint8Array.of(state) }],
        },
        evidenceCounts: { telemetry: samples },
      },
      resolvedAt: { positions: [{ history: { store: "runtime", epoch }, sequence: samples }], revisions: [] },
    }) as unknown as RuntimeStateFrame;
  assert.equal(runtimeStateOf(frame(1n, 1)), runtimeStateOf(frame(100n, 1)));
  assert.notEqual(runtimeStateOf(frame(100n, 1)), runtimeStateOf(frame(100n, 2)));
  assert.notEqual(runtimeStateOf(frame(100n, 2)), runtimeStateOf(frame(100n, 2, "restarted")));
  assert.equal(runtimeStateOf({ overview: undefined }), null);
});

test("Live Evidence Refresh", () => {
  vi.useFakeTimers();
  const refresh = vi.fn();
  const files = createEvidenceRefresh(refresh, 100);
  const frame = (phase: string, cursor = "1") =>
    ({
      resolvedAt: {
        positions: [{ history: { store: "runtime", epoch: "epoch" }, sequence: BigInt(cursor) }],
        revisions: [{ name: "overview", revision: new TextEncoder().encode(phase) }],
      },
    }) as unknown as RuntimeStateFrame;
  try {
    files.observe(frame("Receiving 3 B"));
    vi.advanceTimersByTime(100);
    assert.equal(refresh.mock.calls.length, 1);
    files.observe(frame("Receiving 3 B"));
    vi.advanceTimersByTime(1_000);
    assert.equal(refresh.mock.calls.length, 1);
    files.observe(frame("Receiving 4 B"));
    files.observe(frame("Receiving 5 B"));
    vi.advanceTimersByTime(100);
    assert.equal(refresh.mock.calls.length, 2);
    files.observe(frame("Saving"));
    vi.advanceTimersByTime(100);
    assert.equal(refresh.mock.calls.length, 3);
    files.observe(frame("Saving", "2"));
    vi.advanceTimersByTime(100);
    assert.equal(refresh.mock.calls.length, 4);
    files.observe({ ...frame("Saving", "2"), opening: true });
    vi.advanceTimersByTime(100);
    assert.equal(refresh.mock.calls.length, 5);
    files.observe(frame("Finished", "2"));
    files.dispose();
    vi.advanceTimersByTime(1_000);
    assert.equal(refresh.mock.calls.length, 5);
  } finally {
    files.dispose();
    vi.useRealTimers();
  }
});

test("Saved Capture Refreshes Active Execution Files", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  let files: { path: string }[] = [];
  const observer = new QueryObserver(client, {
    queryKey: workflowQueryKeys.executionFiles("running"),
    queryFn: async () => files,
    staleTime: Infinity,
  });
  const unsubscribe = observer.subscribe(() => undefined);
  try {
    await observer.refetch();
    assert.deepEqual(observer.getCurrentResult().data, []);
    files = [{ path: "media/capture.png" }];
    await invalidateWorkflowDocuments(client);
    assert.deepEqual(observer.getCurrentResult().data, files);
  } finally {
    unsubscribe();
    client.clear();
  }
});
