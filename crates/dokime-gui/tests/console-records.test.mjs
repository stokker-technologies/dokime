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
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const {
  RETAINED_BUFFERS,
  RETAINED_RECORDS,
  consoleBufferKey,
  createConsoleRecordsStore,
  mergeConsoleRecords,
} = await import(pathToFileURL(resolve(gui, "src/shell/console-buffer.ts")).href);

/* The keys the drawer resolves on the cycle page and on an execution page
   outside the cycle route, where the scope falls back to the instance. */
const CYCLE_PAGE = consoleBufferKey({ cycleId: "cycle-1", executionId: null, level: "info" });
const EXECUTION_PAGE = consoleBufferKey({ cycleId: null, executionId: null, level: "info" });

/* === Fixture === */

function record(sequence) {
  return {
    details: {},
    display: { icon: "clock", tone: "neutral" },
    level: "info",
    message: `event ${sequence}`,
    record_id: `trace-${sequence}`,
    record_type: "log",
    source: "dokime::test",
    timestamp: "",
  };
}

function page(from, to, epoch = "epoch-1") {
  const records = [];
  for (let sequence = from; sequence <= to; sequence += 1) records.push(record(sequence));
  return { dropped: 0n, epoch, nextSequence: BigInt(to), records };
}

function ids(buffer) {
  return buffer.records.map((item) => item.record_id);
}

function sessionStore() {
  const store = createConsoleRecordsStore();
  store.getState().bindSession("1");
  return store;
}

/* === Tests === */

test("records survive a move to another route and back", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, 3), 0n);
  store.getState().receive(EXECUTION_PAGE, page(1, 5), 0n);

  const back = store.getState().buffers.get(CYCLE_PAGE);
  assert.deepEqual(ids(back), ["trace-1", "trace-2", "trace-3"]);
  assert.equal(back.afterSequence, 3n, "the cycle stream resumes where it stopped");
  assert.equal(store.getState().buffers.get(EXECUTION_PAGE).records.length, 5);
});

test("the filter and the rows outlive the drawer's subscription", () => {
  const store = sessionStore();
  store.getState().setFilter("charger");
  store.getState().receive(CYCLE_PAGE, page(1, 2), 0n);
  store.getState().bindSession("1");
  assert.equal(store.getState().filter, "charger");
  assert.equal(store.getState().buffers.get(CYCLE_PAGE).records.length, 2);
});

test("a replayed page merges without duplicating rows", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, 4), 0n);
  store.getState().rewind(CYCLE_PAGE);
  store.getState().receive(CYCLE_PAGE, page(3, 6), 0n);
  assert.deepEqual(ids(store.getState().buffers.get(CYCLE_PAGE)), [
    "trace-1",
    "trace-2",
    "trace-3",
    "trace-4",
    "trace-5",
    "trace-6",
  ]);
});

test("a page from another epoch replaces the rows", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, 4), 0n);
  store.getState().receive(CYCLE_PAGE, page(1, 2, "epoch-2"), 0n);
  assert.deepEqual(ids(store.getState().buffers.get(CYCLE_PAGE)), ["trace-1", "trace-2"]);
});

test("a new native session starts with no rows", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, 2), 0n);
  store.getState().bindSession("2");
  assert.equal(store.getState().buffers.size, 0);
});

test("clearing empties the rows and keeps the cursor", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, 3), 0n);
  store.getState().clear(CYCLE_PAGE);
  const cleared = store.getState().buffers.get(CYCLE_PAGE);
  assert.equal(cleared.records.length, 0);
  assert.equal(cleared.afterSequence, 3n);
});

test("each scope's rows stay within the ring limit", () => {
  const store = sessionStore();
  store.getState().receive(CYCLE_PAGE, page(1, RETAINED_RECORDS), 0n);
  store.getState().receive(CYCLE_PAGE, page(RETAINED_RECORDS + 1, RETAINED_RECORDS + 10), 0n);
  const buffer = store.getState().buffers.get(CYCLE_PAGE);
  assert.equal(buffer.records.length, RETAINED_RECORDS);
  assert.equal(buffer.records.at(-1).record_id, `trace-${RETAINED_RECORDS + 10}`);
  assert.equal(buffer.records[0].record_id, "trace-11");
});

test("only the most recently read scopes are kept", () => {
  const store = sessionStore();
  for (let index = 0; index <= RETAINED_BUFFERS; index += 1) {
    const key = consoleBufferKey({ cycleId: `cycle-${index}`, executionId: null, level: "info" });
    store.getState().receive(key, page(1, 1), 0n);
  }
  const keys = [...store.getState().buffers.keys()];
  assert.equal(keys.length, RETAINED_BUFFERS);
  assert.ok(
    !keys.includes(consoleBufferKey({ cycleId: "cycle-0", executionId: null, level: "info" })),
  );
});

test("an out-of-order merge restores sequence order", () => {
  const merged = mergeConsoleRecords([record(2), record(5)], [record(1), record(3)]);
  assert.deepEqual(
    merged.map((item) => item.record_id),
    ["trace-1", "trace-2", "trace-3", "trace-5"],
  );
});
