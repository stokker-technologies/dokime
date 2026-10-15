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
import { test } from "node:test";
import {
  formatStatValue,
  formatSummaryValue,
  nearestDrawnValue,
  seriesLegendColumns,
  seriesLegendRows,
  summaryInFlight,
  toggleTrace,
} from "../src/lib/statistics.ts";

/* === Constants === */

const TRACES = [
  { key: "voltage", label: "Voltage", color: "#3b82f6" },
  { key: "current", label: "Current", color: "#ef4444" },
];

const SUMMARIES = [
  {
    series_id: "current",
    count: 5,
    min: 5,
    max: 9,
    mean: 7,
    median: 7,
    std: Math.SQRT2,
    rms: Math.sqrt(51),
    window: { from: 5, to: 9 },
  },
  {
    series_id: "voltage",
    count: 10,
    min: 0,
    max: 1234.5,
    mean: 4.5,
    median: 4.5,
    std: Math.sqrt(8.25),
    rms: Math.sqrt(28.5),
    window: { from: 0, to: 9 },
  },
];

/* === Tests === */

test("A live legend lists the trace, the cursor and the statistics with the unit in each header", () => {
  assert.deepEqual(
    seriesLegendColumns(true, "V").map((column) => column.header),
    [
      "Trace",
      "Cursor (V)",
      "Count",
      "Min (V)",
      "Max (V)",
      "Mean (V)",
      "Median (V)",
      "Std (V)",
      "Rms (V)",
    ],
  );
});

test("A report legend omits the cursor column", () => {
  assert.deepEqual(
    seriesLegendColumns(false, null).map((column) => column.id),
    ["trace", "count", "min", "max", "mean", "median", "std", "rms"],
  );
});

test("Rows follow trace order and read each trace's own backend summary", () => {
  const rows = seriesLegendRows({ hidden: new Set(), summaries: SUMMARIES, traces: TRACES });

  assert.deepEqual(
    rows.map((row) => [row.key, row.label, row.color, row.cells.count, row.cells.mean]),
    [
      ["voltage", "Voltage", "#3b82f6", "10", "4.50"],
      ["current", "Current", "#ef4444", "5", "7.00"],
    ],
  );
  assert.deepEqual(rows[0].cells, {
    count: "10",
    min: "0.00",
    max: "1235",
    mean: "4.50",
    median: "4.50",
    std: "2.87",
    rms: "5.34",
  });
  assert.equal(rows[1].cells.rms, "7.14");
});

test("A trace without a summary shows dashes, never browser-side statistics", () => {
  const [row] = seriesLegendRows({
    hidden: new Set(),
    summaries: [],
    traces: [TRACES[0]],
  });

  assert.deepEqual(Object.values(row.cells), ["-", "-", "-", "-", "-", "-", "-"]);
  assert.equal(formatSummaryValue(null), "-");
  assert.equal(formatSummaryValue(Number.NaN), "-");
  assert.equal(formatStatValue(undefined, "V"), "-");
  assert.equal(formatStatValue(2.5, "V"), "2.50 V");
});

test("Clicking a row toggles its trace and a hidden row keeps its numbers", () => {
  let hidden = new Set();
  hidden = toggleTrace(hidden, "current");
  assert.deepEqual([...hidden], ["current"]);

  const rows = seriesLegendRows({ hidden, summaries: SUMMARIES, traces: TRACES });
  assert.deepEqual(
    rows.map((row) => [row.key, row.hidden]),
    [
      ["voltage", false],
      ["current", true],
    ],
  );
  assert.equal(rows[1].cells.mean, "7.00");

  hidden = toggleTrace(hidden, "current");
  assert.equal(hidden.size, 0);
});

test("The cursor column is blank until the pointer has been over the plot", () => {
  const rows = seriesLegendRows({ hidden: new Set(), summaries: SUMMARIES, traces: TRACES });
  assert.deepEqual(
    rows.map((row) => row.cursor),
    ["", ""],
  );
});

test("The cursor reads each trace's nearest drawn point and skips its gaps", () => {
  const x = [0, 1, 2, 3, 4];
  const voltage = [10, 11, null, 13, 14];
  const current = [null, null, null, null, null];

  assert.equal(nearestDrawnValue(x, voltage, 0.4), 10);
  assert.equal(nearestDrawnValue(x, voltage, 0.6), 11);
  assert.equal(nearestDrawnValue(x, voltage, 2.2), 13);
  assert.equal(nearestDrawnValue(x, voltage, 99), 14);
  assert.equal(nearestDrawnValue(x, current, 1), null);
  assert.equal(nearestDrawnValue([3, 0, 2], [30, 0, 20], 0.9), 0);

  const cursorValues = [nearestDrawnValue(x, voltage, 3.1), nearestDrawnValue(x, current, 3.1)];
  const rows = seriesLegendRows({
    cursorValues,
    hidden: new Set(),
    summaries: SUMMARIES,
    traces: TRACES,
  });
  assert.deepEqual(
    rows.map((row) => row.cursor),
    ["13.00", ""],
  );
});

test("The table dims only while a summary request is in flight", () => {
  // A zoom or window change asks for new numbers: the old range's dim.
  assert.equal(summaryInFlight("requested"), true);
  // Its frame arrives, or the request fails: nothing is in flight.
  assert.equal(summaryInFlight("delivered"), false);
  assert.equal(summaryInFlight("failed"), false);
  // A run that ends closes its stream; its final numbers stay undimmed.
  let pending = summaryInFlight("requested");
  pending = summaryInFlight("delivered");
  pending = pending && summaryInFlight("ended");
  assert.equal(pending, false);
});
