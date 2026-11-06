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
import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { test, vi } from "vitest";

// Local Imports
import { createExecutionFileRefresh, createRuntimeProjectionRefreshCoordinator, createThrottledRefresh } from "@/app/live/runtime-refresh";
import type { RuntimeStateFrame } from "@/lib/api/native-observation";
import { mapNativeExecutionSummary, projectExecutionStatus } from "@/lib/api/native-planning";
import { invalidateWorkflowDocuments, workflowQueryKeys } from "@/features/cycle/server-state";

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
    if (calls === 1) await gate;
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

test("Unstarted Attempt Status", () => {
  const skipped = { HarmosVariant: { case: "Skipped" as const, value: {} } };
  assert.equal(projectExecutionStatus(skipped, null, false), "planned");
  assert.equal(projectExecutionStatus(skipped, null, true), "skipped");
  for (const outcome of ["Completed", "Failed", "Stopped", "Interrupted"]) {
    const cycle = { cycleId: "cycle", status: { HarmosVariant: { case: outcome, value: {} } } };
    const attempt = { executionId: "unstarted", caseId: "case", ordinal: 1n, status: skipped, sealed: false, archived: false };
    const projected = mapNativeExecutionSummary(attempt, [], cycle as never);
    assert.equal(projected.status, "planned");
    assert.equal(projected.sealed, false);
  }
});

test("Live File Refresh", () => {
  vi.useFakeTimers();
  const refresh = vi.fn();
  const files = createExecutionFileRefresh(refresh, 100);
  const frame = (phase: string, cursor = "1") =>
    ({
      cursor: { epoch: "epoch", stream: "runtime", position: BigInt(cursor) },
      operations: [{ operationId: "upload", phase, startedAt: { seconds: 1n, nanos: 0 } }],
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
    files.observe({ ...frame("Saving", "2"), resynchronized: true });
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
