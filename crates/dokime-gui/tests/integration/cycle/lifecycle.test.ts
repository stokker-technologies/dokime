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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { ControlCard } from "@/features/cycle/ControlCard";
import { cycleControlState } from "@/features/cycle/control-model";
import { invalidateRuntimeDocuments, invalidateRuntimeProjectionQueries, workflowQueryKeys } from "@/features/cycle/server-state";
import {
  appendNativeCaseAttempt,
  fetchNativeTestCycleSteps,
  sealNativeTestCycle,
  startNativeTestExecution,
  stopNativeTestExecution,
} from "@/lib/api/native-execution";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  append: vi.fn(),
  start: vi.fn(),
  stop: vi.fn(),
  seal: vi.fn(),
  detail: vi.fn(),
  parameters: vi.fn(),
  steps: vi.fn(),
}));

const session = { generation: "lifecycle", isCurrent: () => true };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function accepted() {
  return { outcome: { HarmosVariant: { case: "Accepted", value: { field0: { commandId: "execution", jobId: null } } } } };
}

/* === Entry Point === */

vi.mock("@/lib/rpc/generated/client", () => ({
  executionAppendExecutions: fixture.append,
  executionStartExecution: fixture.start,
  executionStopExecution: fixture.stop,
  executionSealCycle: fixture.seal,
  executionGetExecution: fixture.detail,
  executionListExecutionParameters: fixture.parameters,
  executionListExecutionSteps: fixture.steps,
}));

beforeEach(() => {
  vi.clearAllMocks();
});

test("Execution Detail Uses Embedded Criteria", async () => {
  fixture.detail.mockResolvedValue({ cycleId: "cycle", summary: { executionId: "execution", caseId: "case" } });
  fixture.parameters.mockResolvedValue({ parameters: [] });
  const steps = Array.from({ length: 20 }, (_, ordinal) => ({
    stepId: `step-${ordinal}`,
    ordinal: BigInt(ordinal),
    measurementCount: 0n,
    criterionCount: 1n,
    criteria: [{ criterionId: `criterion-${ordinal}`, ordinal: 0n }],
  }));
  fixture.steps.mockResolvedValue({ steps });

  const records = await fetchNativeTestCycleSteps(session, { cycleId: "cycle", executionId: "execution" });
  assert.equal(records.length, 40);
  assert.equal(records.filter((record) => record.record_type === "criterion").length, 20);
  assert.equal(records[39].step_id, "criterion-19");
  assert.equal(fixture.detail.mock.calls.length, 1);
  assert.equal(fixture.parameters.mock.calls.length, 2);
  assert.equal(fixture.steps.mock.calls.length, 1);
});

test("Append Without Start", async () => {
  fixture.append.mockResolvedValue({});
  const first = await appendNativeCaseAttempt(session, "cycle", "case");
  const second = await appendNativeCaseAttempt(session, "cycle", "case");
  assert.notEqual(first, second);
  assert.match(first, /^[0-9a-f-]{36}$/);
  assert.deepEqual(fixture.append.mock.calls, [
    [session, { cycleId: "cycle", executions: [{ executionId: first, caseId: "case" }] }],
    [session, { cycleId: "cycle", executions: [{ executionId: second, caseId: "case" }] }],
  ]);
  assert.equal(fixture.start.mock.calls.length, 0);
});

test("Start Reuses Identity", async () => {
  fixture.start.mockResolvedValue(accepted());
  await startNativeTestExecution(session, "execution");
  await startNativeTestExecution(session, "execution");
  const request = { executionId: "execution", mode: { HarmosVariant: { case: "Automatic", value: {} } } };
  assert.deepEqual(fixture.start.mock.calls, [
    [session, request],
    [session, request],
  ]);
});

test("Start Admission Refusal", async () => {
  fixture.start.mockResolvedValue({
    outcome: {
      HarmosVariant: {
        case: "Rejected",
        value: {
          field0: {
            code: "cycle_sealed",
            message: "Cycle Is Sealed",
            issues: [{ path: "execution_id", message: "Admission Closed" }],
          },
        },
      },
    },
  });
  await assert.rejects(startNativeTestExecution(session, "execution"), /cycle_sealed: Cycle Is Sealed.*execution_id: Admission Closed/);
});

test("Durable Stop Seal", async () => {
  let finishStop!: () => void;
  fixture.stop.mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        finishStop = resolve;
      }),
  );
  let stopped = false;
  const stop = stopNativeTestExecution(session, "execution").then(() => {
    stopped = true;
  });
  await Promise.resolve();
  assert.equal(stopped, false);
  finishStop();
  await stop;
  assert.equal(stopped, true);
  assert.deepEqual(fixture.stop.mock.calls, [[session, { executionId: "execution" }]]);

  let failSeal!: (error: Error) => void;
  fixture.seal.mockImplementation(
    () =>
      new Promise<void>((_resolve, reject) => {
        failSeal = reject;
      }),
  );
  const seal = sealNativeTestCycle(session, "cycle");
  const rejected = assert.rejects(seal, /Persistence Failed/);
  failSeal(new Error("Persistence Failed"));
  await rejected;
  assert.deepEqual(fixture.seal.mock.calls, [[session, { cycleId: "cycle" }]]);
});

test("Container Seal Controls", () => {
  // An Active Cycle Offers Seal Cycle Alone, Whatever Its Members Did
  for (const running of [null, { ordinal: 2, step: "Energize Charger", stepId: "energize" }]) {
    const state = cycleControlState({ phase: "active", running });
    assert.deepEqual(state.actions, ["Seal Cycle"]);
    const html = renderToStaticMarkup(
      createElement(ControlCard, {
        actions: createElement("button", { type: "button" }, state.actions[0]),
        label: "Cycle Controls",
        state: state.line,
      }),
    );
    assert.match(html, /Seal Cycle/);
    assert.equal((html.match(/<button/g) ?? []).length, 1);
    assert.doesNotMatch(html, /Start Cycle|Stop Cycle|Run Again|Pause Cycle|Resume Cycle/);
  }
  assert.equal(cycleControlState({ phase: "active", running: null }).line, "Idle · no execution running");
  assert.equal(
    cycleControlState({ phase: "active", running: { ordinal: 2, step: "Energize Charger", stepId: "energize" } }).line,
    "Execution 2 · Energize Charger",
  );

  // Sealing And Sealed Offer Nothing
  assert.deepEqual(cycleControlState({ phase: "sealing", running: null }), {
    actions: [],
    detail: null,
    line: "Sealing · saving evidence",
  });
  assert.deepEqual(cycleControlState({ phase: "sealed", running: null }), { actions: [], detail: null, line: "Sealed · evidence saved" });
});

test("Stream Refreshes Files And Preserves Sealed Detail", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
  const calls = { files: 0, document: 0, sealed: 0, active: 0 };
  const keys = {
    files: workflowQueryKeys.executionFiles("active"),
    document: workflowQueryKeys.executionDocument({ cycleId: "cycle", executionId: "active" }),
    sealed: workflowQueryKeys.executionDetail("sealed"),
    active: workflowQueryKeys.executionDetail("active"),
  };
  const observers = Object.entries(keys).map(([name, queryKey]) => {
    const kind = name as keyof typeof calls;
    return new QueryObserver(client, {
      queryKey,
      queryFn: async () => ({ revision: ++calls[kind], sealed: kind === "sealed" }),
    });
  });
  const release = observers.map((observer) => observer.subscribe(() => {}));
  try {
    await Promise.all(observers.map((observer) => observer.refetch()));
    assert.deepEqual(calls, { files: 1, document: 1, sealed: 1, active: 1 });

    await invalidateRuntimeDocuments(client);
    assert.deepEqual(calls, { files: 2, document: 2, sealed: 1, active: 1 });
    await invalidateRuntimeProjectionQueries(client);
    assert.deepEqual(calls, { files: 2, document: 2, sealed: 1, active: 2 });
  } finally {
    for (const unsubscribe of release) unsubscribe();
    client.clear();
  }
});
