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

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const { DEFAULT_PLOT_MAX_POINTS, RAW_RING_FACTOR, lttbIndices, minMaxIndices, reduceIndices, retainedStart } =
  await import(pathToFileURL(resolve(gui, "src/plots/reduce.ts")).href);

/* The pulse suite: 100 ms cadence, sin(0.5 i), about 12.6 samples per period. */
const PULSE_PERIOD_MS = 100;

/* === Fixture === */

function pulse(count, startS = 0) {
  const x = [];
  const y = [];
  for (let index = 0; index < count; index += 1) {
    x.push(startS + (index * PULSE_PERIOD_MS) / 1000);
    y.push(Math.sin(0.5 * index));
  }
  return { x, y };
}

function assertAscending(indices) {
  for (let index = 1; index < indices.length; index += 1) {
    assert.ok(indices[index] > indices[index - 1], `indices must ascend at ${index}`);
  }
}

/* === Tests === */

test("lttb keeps the ends, the count, and ascending order", () => {
  const { x, y } = pulse(1_200);
  const kept = lttbIndices(x, y, 100);
  assert.equal(kept.length, 100);
  assert.equal(kept[0], 0);
  assert.equal(kept.at(-1), 1_199);
  assertAscending(kept);
});

test("lttb returns every index when the budget is not exceeded", () => {
  const { x, y } = pulse(10);
  assert.deepEqual(
    lttbIndices(x, y, 10),
    Array.from({ length: 10 }, (_, index) => index),
  );
  assert.deepEqual(lttbIndices(x, y, 2), Array.from({ length: 10 }, (_, index) => index));
});

test("lttb picks the sample that spans the largest triangle", () => {
  /* Three buckets: the spike at index 1 beats its flat neighbours. */
  const x = [0, 1, 2, 3, 4];
  const y = [0, 10, 0, 0, 0];
  assert.deepEqual(lttbIndices(x, y, 3), [0, 1, 4]);
});

test("lttb is time-aware: a gap does not collapse the samples around it", () => {
  /* Two dense runs separated by a long silence; a count-based pick treats
     them alike, a time-aware pick must keep both ends of the silence. */
  const first = pulse(300, 0);
  const second = pulse(300, 600);
  const x = [...first.x, ...second.x];
  const y = [...first.y, ...second.y];
  const kept = lttbIndices(x, y, 60);
  const keptX = kept.map((index) => x[index]);
  assert.ok(keptX.some((value) => value < 30), "keeps samples before the gap");
  assert.ok(keptX.some((value) => value >= 600), "keeps samples after the gap");
  assert.equal(kept.length, 60);
});

test("lttb never picks a sample without a finite y", () => {
  const { x, y } = pulse(400);
  const gapped = y.map((value, index) => (index % 7 === 3 ? null : value));
  const kept = lttbIndices(x, gapped, 50);
  for (const index of kept.slice(1, -1)) {
    assert.notEqual(gapped[index], null, `picked a null at ${index}`);
  }
});

test("min/max keeps each bucket's extrema in index order", () => {
  const y = [1, 5, 3, 0, 2, 9, 4, 4];
  /* count 4 is two buckets of four samples. */
  assert.deepEqual(minMaxIndices(y, 4), [1, 3, 4, 5]);
});

test("reduceIndices honours the technique and the raw escape hatch", () => {
  const { x, y } = pulse(500);
  assert.equal(reduceIndices(x, y, { budget: null, technique: "lttb" }), null);
  assert.equal(reduceIndices(x, y, { budget: 100, technique: "raw" }), null);
  assert.equal(reduceIndices(x, y, { budget: 500, technique: "lttb" }), null);
  assert.equal(reduceIndices(x, y, { budget: 100, technique: "lttb" }).length, 100);
  assert.ok(reduceIndices(x, y, { budget: 100, technique: "min_max" }).length <= 100);
});

test("the window applies before the raw cap, so the visible span does not drift", () => {
  /* 120 s at 10 Hz is 1,200 samples; an 800-point count-first cut would
     leave 80 s on screen. The raw cap is sized by the budget and only guards
     memory, so the window decides what stays. */
  const budget = 800;
  const { x, y } = pulse(1_200);
  const start = retainedStart(x, 120, budget * RAW_RING_FACTOR);
  assert.equal(start, 0, "every sample of the last 120 s is retained");

  const kept = reduceIndices(x.slice(start), y.slice(start), { budget, technique: "lttb" });
  assert.equal(kept.length, budget);
  const span = x[kept.at(-1)] - x[kept[0]];
  assert.ok(span >= 119.9, `visible span stays the window, got ${span.toFixed(1)} s`);
});

test("the window evicts samples older than the cutoff", () => {
  const { x } = pulse(1_200);
  const start = retainedStart(x, 60, DEFAULT_PLOT_MAX_POINTS * RAW_RING_FACTOR);
  assert.ok(x[start] >= x.at(-1) - 60);
  assert.ok(start === 0 || x[start - 1] < x.at(-1) - 60);
});

test("the raw cap bounds an unbounded window from the end", () => {
  const { x } = pulse(1_200);
  assert.equal(retainedStart(x, null, 100), 1_100);
  assert.equal(retainedStart(x, null, 0), 0, "a zero cap keeps everything");
  assert.equal(retainedStart([], 10, 10), 0);
});
