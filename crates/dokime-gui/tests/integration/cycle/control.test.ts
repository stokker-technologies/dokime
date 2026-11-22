// @vitest-environment happy-dom
/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Exercise Planned Execution Controls and Progress Admission
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { CycleSummarySchema, ExecutionState, ExecutionSummarySchema } from "@generated/host/model";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { test, vi } from "vitest";

// Local Imports
import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { type ExecutionActions, ExecutionControl, useExecutionActions } from "@/features/cycle/ExecutionControl";
import { NodeMenu, type ExecutionMenu } from "@/features/cycle/NodeMenu";
import type { CycleExecutionRow } from "@dokime/document/features/cycle/cycle-workspace-model";
import { mapNativeExecutionSummary } from "@dokime/document/lib/api/native-planning-projection";

/* === Re-Exports === */

/* === Constants === */

const backend = vi.hoisted(() => ({ stream: vi.fn() }));

vi.mock("@/lib/rpc/transport", () => ({ openStream: backend.stream }));
vi.mock("@/app/native-session/context", () => {
  const session = { identity: { generation: "control", isCurrent: () => true }, own: () => () => {} };
  return { useNativeSessionOptional: () => session };
});

/* === Types === */

type ProgressReader = { cancel: () => Promise<void>; [Symbol.asyncIterator]: () => AsyncGenerator<never> };

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Planned Page Controls Follow Host Capability and Watch Only After Start", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  backend.stream.mockReset();
  let finish!: () => void;
  const finished = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const cancel = vi.fn(async () => {});
  backend.stream.mockImplementation(async (path, _input, output, _identity, request) => {
    assert.equal(path, "/dokime.host.v1.executions.Progress/Watch");
    assert.equal(request.name, "execution");
    return {
      cancel,
      async *[Symbol.asyncIterator]() {
        yield create(output, { value: { label: "Step", completed: 1n, total: 2n }, phase: "running" });
        await finished;
        yield create(output, { sealed: true });
      },
    };
  });
  const cycle = create(CycleSummarySchema, { cycleId: "cycle" });
  let execution = mapNativeExecutionSummary(
    create(ExecutionSummarySchema, { executionId: "execution", state: ExecutionState.OPEN, controls: { canStart: true } }),
    [],
    cycle,
  );
  const client = new QueryClient();
  const element = document.createElement("div");
  const root = createRoot(element);
  let actions: ExecutionActions | null = null;
  let cycleActive = true;
  function Probe() {
    actions = useExecutionActions({
      cycleActive,
      cycleId: "cycle",
      execution,
      onSelectExecution: () => {},
      prompts: [],
      row: { execution, case: { actions: null } } as CycleExecutionRow,
    });
    return actions ? createElement(ExecutionControl, { actions, ordinal: 1 }) : null;
  }
  const render = () => act(async () => root.render(createElement(QueryClientProvider, { client }, createElement(Probe))));
  try {
    // A Planned Page Offers the Host's Start Without Opening Progress.
    await render();
    assert.equal(element.querySelector<HTMLButtonElement>('[data-execution-action="Start"]')?.disabled, false);
    assert.equal(backend.stream.mock.calls.length, 0);

    // A Local Cycle Projection Cannot Override the Host's Start Capability.
    cycleActive = false;
    await render();
    assert.equal(element.querySelector<HTMLButtonElement>('[data-execution-action="Start"]')?.disabled, false);
    cycleActive = true;

    // A Refused Start Has Guidance and No Action.
    const controls = execution.controls;
    assert.ok(controls);
    execution = { ...execution, controls: { ...controls, canStart: false } };
    await render();
    assert.equal(element.querySelector('[data-execution-action="Start"]'), null);
    assert.ok((actions as ExecutionActions | null)?.state.detail);
    assert.equal(backend.stream.mock.calls.length, 0);

    // Admission Starts Observation Without Reopening the Page.
    execution = { ...execution, status: "running", controls: { ...controls, canStart: false, requestId: "execution", canStop: true } };
    await render();
    assert.equal(backend.stream.mock.calls.length, 1);
    assert.equal((actions as ExecutionActions | null)?.progress?.completed, 1);

    // Terminal Progress Never Reconnects.
    await act(async () => finish());
    await act(async () => vi.advanceTimersByTimeAsync(10_000));
    assert.equal(backend.stream.mock.calls.length, 1);
    assert.equal((actions as ExecutionActions | null)?.progress, null);
  } finally {
    finish();
    await act(async () => root.unmount());
    client.clear();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});

test.each([
  ["open", 3],
  ["read", 3],
  ["open", 9],
  ["read", 9],
] as const)("Progress %s Refusal %i Does Not Reconnect", async (stage, code) => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  backend.stream.mockReset();
  const refusal = { kind: "grpc", detail: { code, message: "not_admitted", details: [] } };
  if (stage === "open") {
    backend.stream.mockRejectedValue(refusal);
  } else {
    backend.stream.mockResolvedValue({
      cancel: async () => {},
      [Symbol.asyncIterator]: () => ({
        next: async () => {
          throw refusal;
        },
      }),
    });
  }
  const root = createRoot(document.createElement("div"));
  function Probe() {
    useExecutionProgress("execution");
    return null;
  }
  try {
    await act(async () => root.render(createElement(Probe)));
    assert.equal(backend.stream.mock.calls.length, 1);
    await act(async () => vi.advanceTimersByTimeAsync(10_000));
    assert.equal(backend.stream.mock.calls.length, 1);
  } finally {
    await act(async () => root.unmount());
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});

test("Leaving an Execution Cancels a Progress Watch That Opens Later", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  backend.stream.mockReset();
  let complete!: (reader: ProgressReader) => void;
  const opening = new Promise<ProgressReader>((resolve) => {
    complete = resolve;
  });
  const cancel = vi.fn(async () => {});
  backend.stream.mockReturnValue(opening);
  const root = createRoot(document.createElement("div"));
  let executionId: string | null = "execution";
  function Probe() {
    useExecutionProgress(executionId);
    return null;
  }
  try {
    await act(async () => root.render(createElement(Probe)));
    executionId = null;
    await act(async () => root.render(createElement(Probe)));
    await act(async () => complete({ cancel, async *[Symbol.asyncIterator]() {} }));
    assert.equal(cancel.mock.calls.length, 1);
  } finally {
    await act(async () => root.unmount());
    vi.unstubAllGlobals();
  }
});

test("Entry Header Menu Invokes Start and Delete and Respects Refusals", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const element = document.createElement("div");
  document.body.append(element);
  const root = createRoot(element);
  const start = vi.fn();
  const remove = vi.fn();
  const menu: ExecutionMenu = {
    kind: "execution",
    archive: null,
    onStart: start,
    onDeleteCase: remove,
    onRerun: null,
    onSettings: null,
    onOpenReport: null,
  };
  const open = async () => {
    const trigger = element.querySelector("button");
    assert.ok(trigger);
    await act(async () =>
      trigger.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, button: 0, pointerType: "mouse", ctrlKey: false })),
    );
  };
  try {
    await act(async () => root.render(createElement(NodeMenu, { menu })));
    await open();
    let items = [...document.querySelectorAll<HTMLElement>('[role="menuitem"]')];
    const startItem = items.find((item) => item.textContent === "Start Execution");
    assert.ok(startItem);
    await act(async () => startItem.click());
    assert.equal(start.mock.calls.length, 1);
    await open();
    items = [...document.querySelectorAll<HTMLElement>('[role="menuitem"]')];
    const deleteItem = items.find((item) => item.textContent === "Delete Case");
    assert.ok(deleteItem);
    await act(async () => deleteItem.click());
    assert.equal(remove.mock.calls.length, 1);
    await act(async () => root.render(createElement(NodeMenu, { menu: { ...menu, onStart: null, onDeleteCase: null } })));
    await open();
    items = [...document.querySelectorAll<HTMLElement>('[role="menuitem"]')];
    assert.equal(items.find((item) => item.textContent === "Start Execution")?.getAttribute("aria-disabled"), "true");
    assert.equal(items.find((item) => item.textContent === "Delete Case")?.getAttribute("aria-disabled"), "true");
  } finally {
    await act(async () => root.unmount());
    element.remove();
    vi.unstubAllGlobals();
  }
});
