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
import { QueryClient } from "@tanstack/svelte-query";
import { act, fireEvent, render } from "@testing-library/svelte";
import { test, vi } from "vitest";

// Local Imports
import { observeExecutionProgress } from "@/app/live/execution-progress";
import type { ConnectedSession } from "@/app/native-session/session";
import type { ExecutionActions } from "@/features/cycle/execution-actions";
import type { ExecutionMenu } from "@/features/cycle/node-menu";
import NodeMenu from "@/features/cycle/NodeMenu.svelte";
import ExecutionControlProbe from "./ExecutionControlProbe.svelte";
import { mapNativeExecutionSummary } from "@/lib/api/native-planning-projection";

/* === Re-Exports === */

/* === Constants === */

const backend = vi.hoisted(() => ({ stream: vi.fn() }));

vi.mock("@/lib/rpc/transport", () => ({ openStream: backend.stream }));
function connection(queryClient = new QueryClient()): ConnectedSession {
  return {
    identity: { generation: "control", isCurrent: () => true },
    own: () => () => {},
    queryClient,
  } as unknown as ConnectedSession;
}

/* === Types === */

type ProgressReader = { cancel: () => Promise<void>; [Symbol.asyncIterator]: () => AsyncGenerator<never> };

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Planned Page Controls Follow Host Capability and Watch Only After Start", async () => {
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
  const session = connection(client);
  let actions: ExecutionActions | null = null;
  let cycleActive = true;
  let observed!: () => void;
  const progressObserved = new Promise<void>((resolve) => {
    observed = resolve;
  });
  const onActions = (next: ExecutionActions | null) => {
    actions = next;
    if (next?.progress?.completed === 1) {
      observed();
    }
  };
  const view = render(ExecutionControlProbe, { session, cycleActive, execution, onActions });
  const element = view.container;
  const update = () => view.rerender({ session, cycleActive, execution, onActions });
  try {
    // A Planned Page Offers the Host's Start Without Opening Progress.
    assert.equal(element.querySelector<HTMLButtonElement>('[data-execution-action="Start"]')?.disabled, false);
    assert.equal(backend.stream.mock.calls.length, 0);

    // A Local Cycle Projection Cannot Override the Host's Start Capability.
    cycleActive = false;
    await update();
    assert.equal(element.querySelector<HTMLButtonElement>('[data-execution-action="Start"]')?.disabled, false);
    cycleActive = true;

    // A Refused Start Has Guidance and No Action.
    const controls = execution.controls;
    assert.ok(controls);
    execution = { ...execution, controls: { ...controls, canStart: false } };
    await update();
    assert.equal(element.querySelector('[data-execution-action="Start"]'), null);
    assert.ok((actions as ExecutionActions | null)?.state.detail);
    assert.equal(backend.stream.mock.calls.length, 0);

    // Admission Starts Observation Without Reopening the Page.
    execution = { ...execution, status: "running", controls: { ...controls, canStart: false, requestId: "execution", canStop: true } };
    await update();
    await progressObserved;
    assert.equal(backend.stream.mock.calls.length, 1);
    assert.equal((actions as ExecutionActions | null)?.progress?.completed, 1);

    // Terminal Progress Never Reconnects.
    await act(async () => finish());
    await act(async () => vi.advanceTimersByTimeAsync(10_000));
    assert.equal(backend.stream.mock.calls.length, 1);
    assert.equal((actions as ExecutionActions | null)?.progress, null);
  } finally {
    finish();
    view.unmount();
    client.clear();
    vi.useRealTimers();
  }
});

test.each([
  ["open", 3],
  ["read", 3],
  ["open", 9],
  ["read", 9],
] as const)("Progress %s Refusal %i Does Not Reconnect", async (stage, code) => {
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
  const stop = observeExecutionProgress(connection(), "execution", () => {});
  try {
    assert.equal(backend.stream.mock.calls.length, 1);
    await vi.advanceTimersByTimeAsync(10_000);
    assert.equal(backend.stream.mock.calls.length, 1);
  } finally {
    stop();
    vi.useRealTimers();
  }
});

test("Leaving an Execution Cancels a Progress Watch That Opens Later", async () => {
  backend.stream.mockReset();
  let complete!: (reader: ProgressReader) => void;
  const opening = new Promise<ProgressReader>((resolve) => {
    complete = resolve;
  });
  const cancel = vi.fn(async () => {});
  backend.stream.mockReturnValue(opening);
  const stop = observeExecutionProgress(connection(), "execution", () => {});
  stop();
  await act(async () => complete({ cancel, async *[Symbol.asyncIterator]() {} }));
  assert.equal(cancel.mock.calls.length, 1);
});

test("Entry Header Menu Invokes Start and Delete and Respects Refusals", async () => {
  const start = vi.fn();
  const remove = vi.fn();
  const menu: ExecutionMenu = {
    kind: "execution",
    archive: null,
    onStart: start,
    onDeleteCase: remove,
    onRerun: null,
    onSettings: null,
  };
  const view = render(NodeMenu, { menu });
  const open = () => fireEvent.pointerDown(view.getByRole("button", { name: "Execution Actions" }), { button: 0 });
  try {
    await open();
    await fireEvent.click(view.getByRole("menuitem", { name: "Start Execution" }));
    assert.equal(start.mock.calls.length, 1);
    await open();
    await fireEvent.click(view.getByRole("menuitem", { name: "Delete Case" }));
    assert.equal(remove.mock.calls.length, 1);
    await view.rerender({ menu: { ...menu, onStart: null, onDeleteCase: null } });
    await open();
    assert.equal(view.getByRole("menuitem", { name: "Start Execution" }).getAttribute("aria-disabled"), "true");
    assert.equal(view.getByRole("menuitem", { name: "Delete Case" }).getAttribute("aria-disabled"), "true");
  } finally {
    view.unmount();
  }
});
