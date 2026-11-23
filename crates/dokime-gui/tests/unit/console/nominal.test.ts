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
import { afterEach, beforeEach, test, vi } from "vitest";

// Local Imports
import type { HubRecord } from "@/lib/types";
import type { TraceLevel, TracePage } from "@/lib/api/native-logs";
import { fixedRowWindow } from "@/lib/row-window";
import {
  type ConsoleBuffer,
  consoleRecordsStore,
  createConsoleRecordsStore,
  mergeConsoleRecords,
  RETAINED_BUFFERS,
  RETAINED_RECORDS,
} from "@/shell/console-buffer";
import { CONSOLE_LIST_PADDING_PX, consoleFollowsTail, consoleTailPinned, FOLLOW_SLACK_PX } from "@/shell/console-follow";
import { observeConsoleRecords } from "@/shell/console-records";
import { consoleRecordScope, recordsAfter } from "@/shell/console-view-model";

/* === Re-Exports === */

/* === Constants === */

/* The instance trace keeps one ring per level. */
const INFO = "info";

const DEBUG = "debug";

/* A 200 px viewport over 100 rows of 22 px scrolls to 2,000 px at its bottom. */
const BOTTOM = 2_000;

const fixture = vi.hoisted(() => ({
  open: vi.fn(),
  notifyError: vi.fn(),
  current: true,
  session: { identity: { generation: "1", isCurrent: (): boolean => fixture.current } },
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/** A 200 px viewport over 100 rows of 22 px, scrolled to `scrollTop`. */
function at(scrollTop: number) {
  return { clientHeight: 200, scrollHeight: 2_200, scrollTop };
}

function record(sequence: number): HubRecord {
  return {
    details: {},
    display: { icon: "clock", tone: "neutral" },
    level: "info",
    message: `event ${sequence}`,
    record_id: `trace-${sequence}`,
    record_type: "log",
    source: "desktop::fixture",
    timestamp: "",
  } as unknown as HubRecord;
}

function page(from: number, to: number, epoch = "epoch-1"): TracePage {
  const records: HubRecord[] = [];
  for (let sequence = from; sequence <= to; sequence += 1) records.push(record(sequence));
  return { dropped: 0n, epoch, records };
}

function ids(buffer: ConsoleBuffer | undefined) {
  assert.ok(buffer);
  return buffer.records.map((item) => item.record_id);
}

function sessionStore() {
  const store = createConsoleRecordsStore();
  store.getState().bindSession("1");
  return store;
}

function buffer(store: ReturnType<typeof sessionStore>, key: string): ConsoleBuffer {
  const held = store.getState().buffers.get(key);
  assert.ok(held, key);
  return held;
}

function stream() {
  const queued: ({ page: TracePage } | { failure: unknown } | { end: true })[] = [];
  let wake: (() => void) | undefined;
  const push = (item: (typeof queued)[number]) => {
    queued.push(item);
    wake?.();
  };
  const cancel = vi.fn(async () => push({ end: true }));
  return {
    push,
    cancel,
    async *[Symbol.asyncIterator]() {
      while (true) {
        if (!queued.length)
          await new Promise<void>((resolve) => {
            wake = resolve;
          });
        const item = queued.shift();
        if (!item || "end" in item) {
          return;
        }
        if ("failure" in item) {
          throw item.failure;
        }
        yield item.page;
      }
    },
  };
}

function ConsoleFixture(level: TraceLevel = INFO) {
  return observeConsoleRecords(fixture.session.identity, level);
}

/* === Entry Point === */

vi.mock("@/lib/api/native-logs", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-logs")>()),
  openTraceStream: fixture.open,
}));
vi.mock("@/lib/notifications", () => ({ toast: { error: fixture.notifyError } }));

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal("window", globalThis);
  fixture.current = true;
  fixture.open.mockReset();
  fixture.notifyError.mockReset();
  consoleRecordsStore.getState().bindSession(null);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

test("Stream Reconnect", async () => {
  const first = stream();
  const next = stream();
  fixture.open
    .mockResolvedValueOnce(first)
    .mockRejectedValueOnce({ kind: "grpc", detail: { code: 14, message: "connection reset", details: [] } })
    .mockResolvedValueOnce(next);
  const unmount = ConsoleFixture();
  first.push({ page: page(1, 2) });
  first.push({ failure: { kind: "grpc", detail: { code: 4, message: "operation deadline", details: [] } } });
  await vi.advanceTimersByTimeAsync(0);
  const failed = buffer(consoleRecordsStore, INFO);
  assert.deepEqual(ids(failed), ["trace-1", "trace-2"]);
  assert.equal(fixture.notifyError.mock.calls[0][0].reason, "DEADLINE_EXCEEDED");
  assert.ok(failed.error?.includes("operation deadline"));
  await vi.advanceTimersToNextTimerAsync();
  assert.equal(fixture.open.mock.calls.length, 2);
  assert.equal(fixture.open.mock.calls[1][1], INFO);
  assert.equal(vi.getTimerCount(), 1);
  await vi.advanceTimersByTimeAsync(1_000);
  assert.equal(fixture.open.mock.calls.length, 2);
  await vi.advanceTimersToNextTimerAsync();
  next.push({ page: { dropped: 0n, epoch: "epoch-1", records: [] } });
  await vi.advanceTimersByTimeAsync(0);
  assert.equal(buffer(consoleRecordsStore, INFO).error, null);
  next.push({ page: page(2, 3) });
  await vi.advanceTimersByTimeAsync(0);
  assert.deepEqual(ids(buffer(consoleRecordsStore, INFO)), ["trace-1", "trace-2", "trace-3"]);
  assert.equal(buffer(consoleRecordsStore, INFO).error, null);
  unmount();
  await vi.advanceTimersByTimeAsync(0);
  assert.ok(next.cancel.mock.calls.length > 0);
  assert.equal(vi.getTimerCount(), 0);
});

test("History Recovery", async () => {
  consoleRecordsStore.getState().bindSession("1");
  consoleRecordsStore.getState().receive(INFO, { ...page(1, 2), dropped: 4n });
  const replay = stream();
  fixture.open
    .mockRejectedValueOnce({ kind: "grpc", detail: { code: 11, message: "expired cursor", details: [] } })
    .mockResolvedValueOnce(replay);
  const unmount = ConsoleFixture();
  await vi.advanceTimersByTimeAsync(0);
  assert.ok(buffer(consoleRecordsStore, INFO).error?.includes("Trace history expired"));
  await vi.advanceTimersToNextTimerAsync();
  replay.push({ page: page(1, 1, "epoch-2") });
  await vi.advanceTimersByTimeAsync(0);
  assert.deepEqual(ids(buffer(consoleRecordsStore, INFO)), ["trace-1"]);
  assert.equal(buffer(consoleRecordsStore, INFO).dropped, 0n);
  unmount();
  await vi.advanceTimersByTimeAsync(0);
});

test("Scope Cancellation", async () => {
  let resolve: ((value: ReturnType<typeof stream>) => void) | undefined;
  fixture.open.mockReturnValueOnce(
    new Promise((opened) => {
      resolve = opened;
    }),
  );
  const unmount = ConsoleFixture();
  unmount();
  const late = stream();
  resolve?.(late);
  await vi.advanceTimersByTimeAsync(0);
  assert.equal(late.cancel.mock.calls.length, 1);
  assert.equal(vi.getTimerCount(), 0);
  assert.equal(consoleRecordsStore.getState().buffers.size, 0);

  fixture.open.mockRejectedValueOnce({ kind: "grpc", detail: { code: 7, message: "trace permission", details: [] } });
  const close = ConsoleFixture(DEBUG);
  await vi.advanceTimersByTimeAsync(0);
  assert.ok(buffer(consoleRecordsStore, DEBUG).error?.includes("trace permission"));
  assert.equal(vi.getTimerCount(), 0);
  close();
});

test("Scope Widening", () => {
  // Cycle and Execution Read the Records Keyed Like the Document's Own Query
  assert.deepEqual(consoleRecordScope("execution", "cycle-1", "execution-1"), { cycleId: "cycle-1", executionId: "execution-1" });
  assert.deepEqual(consoleRecordScope("cycle", "cycle-1", "execution-1"), { cycleId: "cycle-1" });

  // A Scope the Route Cannot Fill Widens, Down to the Instance's Trace
  assert.deepEqual(consoleRecordScope("execution", "cycle-1", null), { cycleId: "cycle-1" });
  assert.equal(consoleRecordScope("cycle", null, null), null);
  assert.equal(consoleRecordScope("instance", "cycle-1", "execution-1"), null);
});

test("Tail Follow", () => {
  // The List Is Pinned at the Bottom and Within One Row of It
  assert.equal(consoleTailPinned(at(BOTTOM)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX - 1)), false);

  // Scrolling Up Releases the Tail
  assert.equal(consoleTailPinned(at(BOTTOM - 300)), false);

  // A List Too Short to Scroll Is Always on Its Tail
  assert.equal(consoleTailPinned({ clientHeight: 200, scrollHeight: 120, scrollTop: 0 }), true);

  // A Row Landing Before the Release Reaches the Drawer Leaves the Operator Where They Scrolled
  assert.equal(consoleFollowsTail(true, consoleTailPinned(at(BOTTOM - 300))), false);
  assert.equal(consoleFollowsTail(true, consoleTailPinned(at(BOTTOM))), true);
  assert.equal(consoleFollowsTail(false, true), false);

  // The First and Last Rows Are Drawn at the Padded Scroll Limits
  const top = fixedRowWindow(100, 22, 0, 200, 0, CONSOLE_LIST_PADDING_PX);
  assert.equal(top.before, CONSOLE_LIST_PADDING_PX);
  assert.deepEqual([top.first, top.last], [0, 10]);
  const contentHeight = top.before + (top.last - top.first) * 22 + top.after;
  assert.equal(contentHeight, 100 * 22 + 2 * CONSOLE_LIST_PADDING_PX);
  const end = fixedRowWindow(100, 22, contentHeight - 200, 200, 0, CONSOLE_LIST_PADDING_PX);
  assert.equal(end.last, 100);
  assert.equal(end.after, CONSOLE_LIST_PADDING_PX);
  assert.equal(end.before + (end.last - end.first) * 22 + end.after, contentHeight);

  // An Empty List Draws Nothing and Keeps Its Padding
  assert.deepEqual(fixedRowWindow(0, 22, 0, 200, 0, CONSOLE_LIST_PADDING_PX), {
    first: 0,
    last: 0,
    after: CONSOLE_LIST_PADDING_PX,
    before: CONSOLE_LIST_PADDING_PX,
  });
});

test("Record Buffers", () => {
  // Records Survive a Move to Another Level and Back
  const levels = sessionStore();
  levels.getState().receive(INFO, page(1, 3));
  levels.getState().receive(DEBUG, page(1, 5));
  assert.deepEqual(ids(buffer(levels, INFO)), ["trace-1", "trace-2", "trace-3"]);
  assert.equal(buffer(levels, DEBUG).records.length, 5);

  // The Filter and the Rows Outlive the Drawer's Subscription
  const drawer = sessionStore();
  drawer.getState().setFilter("charger");
  drawer.getState().receive(INFO, page(1, 2));
  drawer.getState().bindSession("1");
  assert.equal(drawer.getState().filter, "charger");
  assert.equal(buffer(drawer, INFO).records.length, 2);

  // A Replayed Page Merges Without Duplicating Rows
  const replay = sessionStore();
  replay.getState().receive(INFO, page(1, 4));
  replay.getState().receive(INFO, page(3, 6));
  assert.deepEqual(ids(buffer(replay, INFO)), ["trace-1", "trace-2", "trace-3", "trace-4", "trace-5", "trace-6"]);

  // A Page from Another Epoch Replaces the Rows
  const epochs = sessionStore();
  epochs.getState().receive(INFO, page(1, 4));
  epochs.getState().receive(INFO, page(1, 2, "epoch-2"));
  assert.deepEqual(ids(buffer(epochs, INFO)), ["trace-1", "trace-2"]);

  // A New Native Session Starts with No Rows
  const sessions = sessionStore();
  sessions.getState().receive(INFO, page(1, 2));
  sessions.getState().bindSession("2");
  assert.equal(sessions.getState().buffers.size, 0);

  // Clearing Hides the Rows Held, Also When a Replay Brings Them Back
  const held = [record(1), record(2), record(3)];
  assert.deepEqual(recordsAfter(held, "trace-2"), [record(3)]);
  assert.deepEqual(recordsAfter(held, undefined), held);
  assert.deepEqual(recordsAfter(held, "trace-0"), held, "a marker that left the ring hides nothing newer");

  // An Out-Of-Order Merge Restores Sequence Order
  const merged = mergeConsoleRecords([record(2), record(5)], [record(1), record(3)]);
  assert.deepEqual(
    merged.map((item) => item.record_id),
    ["trace-1", "trace-2", "trace-3", "trace-5"],
  );
});

test("Buffer Retention", () => {
  // Each Level's Rows Stay Within the Ring Limit
  const ring = sessionStore();
  ring.getState().receive(INFO, page(1, RETAINED_RECORDS));
  ring.getState().receive(INFO, page(RETAINED_RECORDS + 1, RETAINED_RECORDS + 10));
  const held = buffer(ring, INFO);
  assert.equal(held.records.length, RETAINED_RECORDS);
  assert.equal(held.records.at(-1)?.record_id, `trace-${RETAINED_RECORDS + 10}`);
  assert.equal(held.records[0].record_id, "trace-11");

  // Only the Most Recently Read Levels Are Kept
  const recent = sessionStore();
  const order = ["trace", "debug", "info", "warning", "error"].slice(0, RETAINED_BUFFERS + 1);
  for (const level of order) recent.getState().receive(level, page(1, 1));
  const keys = [...recent.getState().buffers.keys()];
  assert.equal(keys.length, RETAINED_BUFFERS);
  assert.ok(!keys.includes("trace"));
});
