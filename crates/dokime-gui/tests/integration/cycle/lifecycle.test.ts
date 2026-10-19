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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { CycleRunControls } from "@/features/cycle/CycleRunControls";
import {
  appendNativeCaseAttempt,
  sealNativeTestCycle,
  startNativeTestExecution,
  stopNativeTestExecution,
} from "@/lib/api/native-execution";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  phase: "active",
  status: "completed",
  append: vi.fn(),
  start: vi.fn(),
  stop: vi.fn(),
  seal: vi.fn(),
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
}));

vi.mock("@/features/cycle/cycle-queries", () => ({
  useCycleDetailQuery: () => ({ data: { plan: { test_cycle: { phase: fixture.phase, status: fixture.status } } } }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.phase = "active";
  fixture.status = "completed";
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
  for (const status of ["planned", "running", "paused", "completed", "failed", "stopped"]) {
    fixture.status = status;
    const html = renderToStaticMarkup(createElement(CycleRunControls, { cycleId: "cycle", onSeal: () => undefined }));
    assert.match(html, /Seal Cycle/);
    assert.equal((html.match(/<button/g) ?? []).length, 1);
    assert.doesNotMatch(html, /Start Cycle|Stop Cycle|Run Again|Pause Cycle|Resume Cycle/);
  }
  fixture.phase = "sealing";
  const sealing = renderToStaticMarkup(createElement(CycleRunControls, { cycleId: "cycle", onSeal: () => undefined }));
  assert.match(sealing, /Sealing — Saving Evidence/);
  assert.doesNotMatch(sealing, /<button/);
  fixture.phase = "sealed";
  const sealed = renderToStaticMarkup(createElement(CycleRunControls, { cycleId: "cycle", onSeal: () => undefined }));
  assert.match(sealed, /Sealed/);
  assert.doesNotMatch(sealed, /<button/);
});
