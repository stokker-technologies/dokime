// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The Console Follows the Execution's Records
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { QueryClient, notifyManager } from "@tanstack/svelte-query";
import { render } from "@testing-library/svelte";
import type { EvidenceScope } from "@generated/host/model";
import { test, vi } from "vitest";

// Local Imports
import type { ConnectedSession } from "@/app/native-session/session";
import type { RecordTail } from "@/lib/api/native-evidence";
import type { Wire } from "@/lib/rpc/canonical";
import { scopeVisibleIn } from "@/lib/scope";
import type { HubRecord } from "@/lib/types";
import ConsoleDrawer from "@/shell/ConsoleDrawer.svelte";
import { uiStore } from "@/stores/ui-store";

/* === Re-Exports === */

/* === Constants === */

const backend = vi.hoisted(() => ({
  pages: [] as RecordTail[],
  tail: vi.fn(),
  trace: vi.fn(),
  watch: vi.fn(),
  wake: undefined as (() => void) | undefined,
}));

vi.mock("@/lib/api/native-evidence", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-evidence")>()),
  fetchNativeRecordTail: (...args: unknown[]) => backend.tail(...args),
  openNativeRecordStream: (...args: unknown[]) => backend.watch(...args),
}));
vi.mock("@/lib/api/native-logs", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-logs")>()),
  openTraceStream: (...args: unknown[]) => backend.trace(...args),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function record(id: string, level: HubRecord["level"], message: string, executionId?: string): HubRecord {
  return {
    details: {},
    display: { icon: "clock", tone: "neutral" },
    level,
    message,
    record_id: id,
    record_type: "dokime.log",
    scope: { cycleId: "cycle-1", executionId },
    source: "plugin",
    timestamp: "2026-11-23T07:39:06Z",
  } as HubRecord;
}

/** The Backend's Record Stream: Pages Arrive Only When the Test Appends Them. */
function stream() {
  let cancelled = false;
  return {
    async cancel() {
      cancelled = true;
      backend.wake?.();
    },
    async *[Symbol.asyncIterator]() {
      while (!cancelled) {
        const page = backend.pages.shift();
        if (page) {
          yield page;
        } else {
          await new Promise<void>((resolve) => {
            backend.wake = resolve;
          });
        }
      }
    },
  };
}

function rows(container: HTMLElement): string[] {
  return [...container.querySelectorAll("button.font-mono[aria-expanded]")].map((row) => row.lastElementChild?.textContent ?? "");
}

/* === Entry Point === */

test("Console Shows the Cycle's and Execution's Records Live at Trace", async () => {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "requestAnimationFrame", "cancelAnimationFrame"] });
  notifyManager.setScheduler(queueMicrotask);
  const held = [
    record("cycle-1:1", "info", "Cycle Opened"),
    record("execution-1:1", "debug", "Probe Armed", "execution-1"),
    record("execution-2:1", "info", "Other Case Logged", "execution-2"),
  ];
  backend.tail.mockImplementation(async (_identity, scope: Wire<EvidenceScope>) => ({
    cursor: null,
    records: held.filter((item) => scopeVisibleIn(item.scope, scope)),
  }));
  backend.watch.mockImplementation(async () => stream());
  const session = {
    identity: { generation: "console", isCurrent: () => true },
    own: () => () => {},
    queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }),
  } as unknown as ConnectedSession;
  uiStore.getState().setConsoleState({ open: true, level: "trace", scope: "cycle" });
  const view = render(ConsoleDrawer, { session, cycleId: "cycle-1", executionId: "execution-1" });
  await vi.advanceTimersByTimeAsync(0);

  // At Cycle Scope the Drawer Reads the Cycle's Records, Debug Included, Not the Backend's Trace
  assert.deepEqual(rows(view.container), ["Cycle Opened", "Probe Armed", "Other Case Logged"]);
  assert.deepEqual(backend.tail.mock.calls[0][1], { cycleId: "cycle-1" });
  assert.equal(backend.trace.mock.calls.length, 0);

  // A Record the Plugin Logs Now Appears Without a Reload
  const live = record("execution-1:2", "info", "Charging Started", "execution-1");
  held.push(live);
  backend.pages.push({ cursor: null, records: [live] });
  backend.wake?.();
  await vi.advanceTimersByTimeAsync(16);
  assert.equal(rows(view.container).at(-1), "Charging Started");
  assert.equal(backend.tail.mock.calls.length, 1, "the row arrived through the stream, not a re-read");

  // At Execution Scope Only the Selected Execution's Records Remain
  uiStore.getState().setConsoleState({ scope: "execution" });
  await vi.advanceTimersByTimeAsync(0);
  assert.deepEqual(backend.tail.mock.calls.at(-1)?.[1], { cycleId: "cycle-1", executionId: "execution-1" });
  assert.deepEqual(rows(view.container), ["Probe Armed", "Charging Started"]);

  view.unmount();
  uiStore.getState().resetUiStore();
  vi.useRealTimers();
});
