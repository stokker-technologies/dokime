/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "vitest";
import type { HubRecord } from "@/lib/types";
import {
  type ConsoleBuffer,
  type ConsolePage,
  consoleBufferKey,
  createConsoleRecordsStore,
  mergeConsoleRecords,
  RETAINED_BUFFERS,
  RETAINED_RECORDS,
} from "@/shell/console-buffer";
import { consoleFollowsTail, consoleTailPinned, FOLLOW_SLACK_PX } from "@/shell/console-follow";

/* === Fixture === */

/* The keys the drawer resolves on the cycle page and on an execution page
   outside the cycle route, where the scope falls back to the instance. */
const CYCLE_PAGE = consoleBufferKey({ cycleId: "cycle-1", executionId: null, level: "info" });
const EXECUTION_PAGE = consoleBufferKey({ cycleId: null, executionId: null, level: "info" });

/* A 200 px viewport over 100 rows of 22 px scrolls to 2,000 px at its bottom. */
const BOTTOM = 2_000;

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
    source: "dokime::test",
    timestamp: "",
  } as unknown as HubRecord;
}

function page(from: number, to: number, epoch = "epoch-1"): ConsolePage {
  const records: HubRecord[] = [];
  for (let sequence = from; sequence <= to; sequence += 1) records.push(record(sequence));
  return { dropped: 0n, epoch, nextSequence: BigInt(to), records };
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

/* === Tests === */

test("Tail Follow", () => {
  // The List Is Pinned At The Bottom And Within One Row Of It
  assert.equal(consoleTailPinned(at(BOTTOM)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX - 1)), false);

  // Scrolling Up Releases The Tail
  assert.equal(consoleTailPinned(at(BOTTOM - 300)), false);

  // A List Too Short To Scroll Is Always On Its Tail
  assert.equal(consoleTailPinned({ clientHeight: 200, scrollHeight: 120, scrollTop: 0 }), true);

  // A Row Landing Before The Release Reaches The Drawer Leaves The Operator Where They Scrolled
  assert.equal(consoleFollowsTail(true, consoleTailPinned(at(BOTTOM - 300))), false);
  assert.equal(consoleFollowsTail(true, consoleTailPinned(at(BOTTOM))), true);
  assert.equal(consoleFollowsTail(false, true), false);
});

test("Record Buffers", () => {
  // Records Survive A Move To Another Route And Back
  const routes = sessionStore();
  routes.getState().receive(CYCLE_PAGE, page(1, 3), 0n);
  routes.getState().receive(EXECUTION_PAGE, page(1, 5), 0n);
  const back = buffer(routes, CYCLE_PAGE);
  assert.deepEqual(ids(back), ["trace-1", "trace-2", "trace-3"]);
  assert.equal(back.afterSequence, 3n, "the cycle stream resumes where it stopped");
  assert.equal(buffer(routes, EXECUTION_PAGE).records.length, 5);

  // The Filter And The Rows Outlive The Drawer's Subscription
  const drawer = sessionStore();
  drawer.getState().setFilter("charger");
  drawer.getState().receive(CYCLE_PAGE, page(1, 2), 0n);
  drawer.getState().bindSession("1");
  assert.equal(drawer.getState().filter, "charger");
  assert.equal(buffer(drawer, CYCLE_PAGE).records.length, 2);

  // A Replayed Page Merges Without Duplicating Rows
  const replay = sessionStore();
  replay.getState().receive(CYCLE_PAGE, page(1, 4), 0n);
  replay.getState().rewind(CYCLE_PAGE);
  replay.getState().receive(CYCLE_PAGE, page(3, 6), 0n);
  assert.deepEqual(ids(buffer(replay, CYCLE_PAGE)), ["trace-1", "trace-2", "trace-3", "trace-4", "trace-5", "trace-6"]);

  // A Page From Another Epoch Replaces The Rows
  const epochs = sessionStore();
  epochs.getState().receive(CYCLE_PAGE, page(1, 4), 0n);
  epochs.getState().receive(CYCLE_PAGE, page(1, 2, "epoch-2"), 0n);
  assert.deepEqual(ids(buffer(epochs, CYCLE_PAGE)), ["trace-1", "trace-2"]);

  // A New Native Session Starts With No Rows
  const sessions = sessionStore();
  sessions.getState().receive(CYCLE_PAGE, page(1, 2), 0n);
  sessions.getState().bindSession("2");
  assert.equal(sessions.getState().buffers.size, 0);

  // Clearing Empties The Rows And Keeps The Cursor
  const clearing = sessionStore();
  clearing.getState().receive(CYCLE_PAGE, page(1, 3), 0n);
  clearing.getState().clear(CYCLE_PAGE);
  const cleared = buffer(clearing, CYCLE_PAGE);
  assert.equal(cleared.records.length, 0);
  assert.equal(cleared.afterSequence, 3n);

  // An Out-Of-Order Merge Restores Sequence Order
  const merged = mergeConsoleRecords([record(2), record(5)], [record(1), record(3)]);
  assert.deepEqual(
    merged.map((item) => item.record_id),
    ["trace-1", "trace-2", "trace-3", "trace-5"],
  );
});

test("Buffer Retention", () => {
  // Each Scope's Rows Stay Within The Ring Limit
  const ring = sessionStore();
  ring.getState().receive(CYCLE_PAGE, page(1, RETAINED_RECORDS), 0n);
  ring.getState().receive(CYCLE_PAGE, page(RETAINED_RECORDS + 1, RETAINED_RECORDS + 10), 0n);
  const held = buffer(ring, CYCLE_PAGE);
  assert.equal(held.records.length, RETAINED_RECORDS);
  assert.equal(held.records.at(-1)?.record_id, `trace-${RETAINED_RECORDS + 10}`);
  assert.equal(held.records[0].record_id, "trace-11");

  // Only The Most Recently Read Scopes Are Kept
  const scopes = sessionStore();
  for (let index = 0; index <= RETAINED_BUFFERS; index += 1) {
    const key = consoleBufferKey({ cycleId: `cycle-${index}`, executionId: null, level: "info" });
    scopes.getState().receive(key, page(1, 1), 0n);
  }
  const keys = [...scopes.getState().buffers.keys()];
  assert.equal(keys.length, RETAINED_BUFFERS);
  assert.ok(!keys.includes(consoleBufferKey({ cycleId: "cycle-0", executionId: null, level: "info" })));
});
