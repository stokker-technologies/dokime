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
import {
  formatStatValue,
  formatSummaryValue,
  nearestDrawnValue,
  type SeriesSummary,
  seriesLegendColumns,
  seriesLegendRows,
  statsSummaryCells,
  summaryInFlight,
  toggleTrace,
} from "@/lib/statistics";
import {
  HISTOGRAM_COUNT_LABEL,
  type HistogramColumn,
  type HistogramDeclaration,
  histogramAxes,
  histogramAxisTitle,
} from "@/plots/histogram-axes";
import { histogramBars } from "@/plots/histogram-bins";
import {
  NO_NUMERIC_SAMPLES,
  plotPlaceholderText,
  plotUnavailableText,
  WAITING_FOR_SAMPLES,
} from "@/plots/plot-placeholder";
import {
  DEFAULT_PLOT_MAX_POINTS,
  lttbIndices,
  minMaxIndices,
  RAW_RING_FACTOR,
  reduceIndices,
  retainedStart,
} from "@/plots/reduce";
import { heldViewport, PLOT_DRAG_OPTIONS } from "@/plots/viewport";

/* === Fixture === */

/* The pulse suite: 100 ms cadence, sin(0.5 i), about 12.6 samples per period. */
const PULSE_PERIOD_MS = 100;

/* A 400 x 200 plot area over x in [0, 100] and y in [0, 50], y growing upward. */
const AREA = { width: 400, height: 200 };

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
] as SeriesSummary[];

function pulse(count: number, startS = 0) {
  const x: number[] = [];
  const y: number[] = [];
  for (let index = 0; index < count; index += 1) {
    x.push(startS + (index * PULSE_PERIOD_MS) / 1000);
    y.push(Math.sin(0.5 * index));
  }
  return { x, y };
}

function assertAscending(indices: readonly number[]) {
  for (let index = 1; index < indices.length; index += 1) {
    assert.ok(indices[index] > indices[index - 1], `indices must ascend at ${index}`);
  }
}

function posToVal(position: number, axis: "x" | "y") {
  return axis === "x" ? (position / AREA.width) * 100 : 50 - (position / AREA.height) * 50;
}

function histogram(fields: Record<string, unknown>): HistogramDeclaration {
  return fields as HistogramDeclaration;
}

/* === Tests === */

test("LTTB Reduction", () => {
  // LTTB Keeps The Ends, The Count, And Ascending Order
  const long = pulse(1_200);
  const kept = lttbIndices(long.x, long.y, 100);
  assert.equal(kept.length, 100);
  assert.equal(kept[0], 0);
  assert.equal(kept.at(-1), 1_199);
  assertAscending(kept);

  // LTTB Returns Every Index When The Budget Is Not Exceeded
  const short = pulse(10);
  assert.deepEqual(
    lttbIndices(short.x, short.y, 10),
    Array.from({ length: 10 }, (_, index) => index),
  );
  assert.deepEqual(
    lttbIndices(short.x, short.y, 2),
    Array.from({ length: 10 }, (_, index) => index),
  );

  // LTTB Picks The Sample That Spans The Largest Triangle
  /* Three buckets: the spike at index 1 beats its flat neighbours. */
  assert.deepEqual(lttbIndices([0, 1, 2, 3, 4], [0, 10, 0, 0, 0], 3), [0, 1, 4]);

  // LTTB Is Time-Aware: A Gap Does Not Collapse The Samples Around It
  /* Two dense runs separated by a long silence; a count-based pick treats
     them alike, a time-aware pick must keep both ends of the silence. */
  const first = pulse(300, 0);
  const second = pulse(300, 600);
  const gapX = [...first.x, ...second.x];
  const gapY = [...first.y, ...second.y];
  const gapKept = lttbIndices(gapX, gapY, 60);
  const keptX = gapKept.map((index) => gapX[index]);
  assert.ok(
    keptX.some((value) => value < 30),
    "keeps samples before the gap",
  );
  assert.ok(
    keptX.some((value) => value >= 600),
    "keeps samples after the gap",
  );
  assert.equal(gapKept.length, 60);

  // LTTB Never Picks A Sample Without A Finite Y
  const sparse = pulse(400);
  const gapped = sparse.y.map((value, index) => (index % 7 === 3 ? null : value));
  for (const index of lttbIndices(sparse.x, gapped, 50).slice(1, -1)) {
    assert.notEqual(gapped[index], null, `picked a null at ${index}`);
  }

  // Min/Max Keeps Each Bucket's Extrema In Index Order
  /* count 4 is two buckets of four samples. */
  assert.deepEqual(minMaxIndices([1, 5, 3, 0, 2, 9, 4, 4], 4), [1, 3, 4, 5]);

  // Reduce Indices Honours The Technique And The Raw Escape Hatch
  const { x, y } = pulse(500);
  assert.equal(reduceIndices(x, y, { budget: null, technique: "lttb" }), null);
  assert.equal(reduceIndices(x, y, { budget: 100, technique: "raw" }), null);
  assert.equal(reduceIndices(x, y, { budget: 500, technique: "lttb" }), null);
  assert.equal(reduceIndices(x, y, { budget: 100, technique: "lttb" })?.length, 100);
  const envelope = reduceIndices(x, y, { budget: 100, technique: "min_max" });
  assert.ok(envelope && envelope.length <= 100);

  // Auto Switches To An Envelope Only Above Four Times The Display Budget
  const autoX = Array.from({ length: 41 }, (_, index) => index);
  const autoY = autoX.map((value) => Math.sin(value));
  const budget = 10;
  const moderateX = autoX.slice(0, 40);
  const moderateY = autoY.slice(0, 40);
  assert.deepEqual(
    reduceIndices(moderateX, moderateY, { budget, technique: "auto" }),
    reduceIndices(moderateX, moderateY, { budget, technique: "lttb" }),
  );
  assert.deepEqual(
    reduceIndices(autoX, autoY, { budget, technique: "auto" }),
    reduceIndices(autoX, autoY, { budget, technique: "min_max" }),
  );

  // Auto Preserves Raw Points For An Unlimited Display
  const raw = [0, 1, 2];
  assert.equal(reduceIndices(raw, raw, { budget: null, technique: "auto" }), null);
});

test("Retention Window", () => {
  // The Window Applies Before The Raw Cap, So The Visible Span Does Not Drift
  /* 120 s at 10 Hz is 1,200 samples; an 800-point count-first cut would
     leave 80 s on screen. The raw cap is sized by the budget and only guards
     memory, so the window decides what stays. */
  const budget = 800;
  const { x, y } = pulse(1_200);
  const start = retainedStart(x, 120, budget * RAW_RING_FACTOR);
  assert.equal(start, 0, "every sample of the last 120 s is retained");

  const kept = reduceIndices(x.slice(start), y.slice(start), { budget, technique: "lttb" });
  assert.ok(kept);
  assert.equal(kept.length, budget);
  const span = x[kept[kept.length - 1]] - x[kept[0]];
  assert.ok(span >= 119.9, `visible span stays the window, got ${span.toFixed(1)} s`);

  // The Window Evicts Samples Older Than The Cutoff
  const last = x[x.length - 1];
  const cutoff = retainedStart(x, 60, DEFAULT_PLOT_MAX_POINTS * RAW_RING_FACTOR);
  assert.ok(x[cutoff] >= last - 60);
  assert.ok(cutoff === 0 || x[cutoff - 1] < last - 60);

  // The Raw Cap Bounds An Unbounded Window From The End
  assert.equal(retainedStart(x, null, 100), 1_100);
  assert.equal(retainedStart(x, null, 0), 0, "a zero cap keeps everything");
  assert.equal(retainedStart([], 10, 10), 0);
});

test("Drag Viewport", () => {
  // Drag Options Zoom Either Axis Alone Or Both Together
  assert.equal(PLOT_DRAG_OPTIONS.x, true);
  assert.equal(PLOT_DRAG_OPTIONS.y, true);
  assert.ok(PLOT_DRAG_OPTIONS.uni > 0, "a uni threshold makes one-axis drags possible");
  assert.ok(PLOT_DRAG_OPTIONS.dist > 0, "a click is not a zoom");
  assert.equal(PLOT_DRAG_OPTIONS.setScale, true);

  // A Diagonal Box Holds Both Axes In Value Space
  assert.deepEqual(heldViewport({ left: 100, top: 50, width: 100, height: 50 }, AREA, posToVal), {
    x: { min: 25, max: 50 },
    y: { min: 25, max: 37.5 },
  });

  // A Horizontal Drag Spans The Full Height And Holds X Alone
  assert.deepEqual(heldViewport({ left: 40, top: 0, width: 80, height: 200 }, AREA, posToVal), {
    x: { min: 10, max: 30 },
    y: null,
  });

  // A Vertical Drag Spans The Full Width And Holds Y Alone
  assert.deepEqual(heldViewport({ left: 0, top: 20, width: 400, height: 40 }, AREA, posToVal), {
    x: null,
    y: { min: 35, max: 45 },
  });

  // An Empty Or Full-Area Box Holds Nothing
  assert.equal(heldViewport({ left: 0, top: 0, width: 0, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 10, top: 10, width: 20, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 0, top: 0, width: 400, height: 200 }, AREA, posToVal), null);

  // Ranges Are Ordered Whichever Way The Axis Runs
  const flipped = (position: number, axis: "x" | "y") => (axis === "x" ? 100 - position : position);
  assert.deepEqual(heldViewport({ left: 10, top: 10, width: 10, height: 10 }, AREA, flipped), {
    x: { min: 80, max: 90 },
    y: { min: 10, max: 20 },
  });
});

test("Histogram Axes", () => {
  // A Histogram Draws Its Declared Value On X And The Host's Count On Y
  const format = { kind: "fixed", decimals: 1 };
  const axes = histogramAxes(
    histogram({
      title: "Voltage Distribution",
      x: "voltage_v",
      unit: "V",
      histogram: { bins: 12 },
      x_format: format,
    }),
    { label: "Voltage", unit: "mV", format: null } as HistogramColumn,
  );
  assert.deepEqual(axes, {
    x: { label: "Voltage", unit: "V", format },
    y: { label: HISTOGRAM_COUNT_LABEL },
  });
  assert.equal(histogramAxisTitle(axes.x), "Voltage (V)");
  assert.equal(histogramAxisTitle(axes.y), "Count");

  // A Histogram Without A Column Spec Titles X By Its Declared Field
  const field = histogramAxes(
    histogram({
      title: "Time Until Ready",
      x: "time_until_ready_s",
      unit: null,
      histogram: { bins: 12, unit: "s" },
      x_format: null,
    }),
  );
  assert.equal(histogramAxisTitle(field.x), "time_until_ready_s (s)");
  assert.equal(field.x.format, null);

  // A Histogram Never Titles X By The Row Timestamp
  const timestamp = histogramAxes(
    histogram({
      title: "Current Distribution",
      x: "timestamp",
      unit: null,
      histogram: null,
      x_format: null,
    }),
    {
      label: null,
      unit: "A",
      format: { kind: "engineering", significant_digits: 3 },
    } as unknown as HistogramColumn,
  );
  assert.deepEqual(timestamp.x, {
    label: "Current Distribution",
    unit: "A",
    format: { kind: "engineering", significant_digits: 3 },
  });
});

test("Series Legend", () => {
  // A Live Legend Lists The Trace, The Cursor And The Statistics With The Unit In Each Header
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

  // A Report Legend Omits The Cursor Column
  assert.deepEqual(
    seriesLegendColumns(false, null).map((column) => column.id),
    ["trace", "count", "min", "max", "mean", "median", "std", "rms"],
  );

  // Rows Follow Trace Order And Read Each Trace's Own Backend Summary
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

  // A Trace Without A Summary Shows Dashes, Never Browser-Side Statistics
  const [unsummarized] = seriesLegendRows({
    hidden: new Set(),
    summaries: [],
    traces: [TRACES[0]],
  });
  assert.deepEqual(Object.values(unsummarized.cells), ["-", "-", "-", "-", "-", "-", "-"]);
  assert.equal(formatSummaryValue(null), "-");
  assert.equal(formatSummaryValue(Number.NaN), "-");
  assert.equal(formatStatValue(undefined, "V"), "-");
  assert.equal(formatStatValue(2.5, "V"), "2.50 V");

  // Clicking A Row Toggles Its Trace And A Hidden Row Keeps Its Numbers
  let hidden = new Set<string>();
  hidden = toggleTrace(hidden, "current");
  assert.deepEqual([...hidden], ["current"]);
  const toggled = seriesLegendRows({ hidden, summaries: SUMMARIES, traces: TRACES });
  assert.deepEqual(
    toggled.map((row) => [row.key, row.hidden]),
    [
      ["voltage", false],
      ["current", true],
    ],
  );
  assert.equal(toggled[1].cells.mean, "7.00");
  hidden = toggleTrace(hidden, "current");
  assert.equal(hidden.size, 0);

  // The Cursor Column Is Blank Until The Pointer Has Been Over The Plot
  assert.deepEqual(
    rows.map((row) => row.cursor),
    ["", ""],
  );

  // The Cursor Reads Each Trace's Nearest Drawn Point And Skips Its Gaps
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
  const pointed = seriesLegendRows({
    cursorValues,
    hidden: new Set(),
    summaries: SUMMARIES,
    traces: TRACES,
  });
  assert.deepEqual(
    pointed.map((row) => row.cursor),
    ["13.00", ""],
  );

  // The Table Dims Only While A Summary Request Is In Flight
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

test("Backend Statistics", () => {
  // A Histogram Draws One Bar Per Backend Bin With The Backend's Counts
  const bars = histogramBars({
    bins: [
      { lower: 0, upper: 1.5, count: 3 },
      { lower: 1.5, upper: 3, count: 0 },
      { lower: 3, upper: 4.5, count: 7 },
    ],
    count: 10,
  });
  assert.deepEqual(bars, [
    { slot: "0", min: 0, max: 1.5, count: 3 },
    { slot: "1", min: 1.5, max: 3, count: 0 },
    { slot: "2", min: 3, max: 4.5, count: 7 },
  ]);

  // The Summary Grid Prints The Backend's Statistics And Count
  const cells = statsSummaryCells({
    label: "Voltage",
    unit: "V",
    statistics: { mean: 12.5, median: 12, std: 0.25, rms: 12.6, min: 11, max: 14 },
    count: 42,
  });
  assert.deepEqual(cells, [
    ["Mean", "12.50 V"],
    ["Median", "12.00 V"],
    ["Std", "0.25 V"],
    ["RMS", "12.60 V"],
    ["Min", "11.00 V"],
    ["Max", "14.00 V"],
    ["Count", "42"],
  ]);

  // Statistics The Backend Has Not Delivered Print Dashes, Never Browser Numbers
  const missing = statsSummaryCells({ label: "Voltage", unit: "V", statistics: null, count: 0 });
  assert.deepEqual(
    missing.map(([, value]) => value),
    ["-", "-", "-", "-", "-", "-", "0"],
  );
});

test("Plot Placeholders", () => {
  // A Plot Without Samples Waits With The Shared Label And Detail
  assert.deepEqual(plotPlaceholderText({}), {
    label: WAITING_FOR_SAMPLES,
    detail: NO_NUMERIC_SAMPLES,
  });
  assert.equal(WAITING_FOR_SAMPLES, "Waiting For Samples");

  // A Stated Stream Condition Replaces The Wait In The Placeholder
  const stated = plotPlaceholderText({
    stateLabel: "Reconnecting",
    stateDetail: "The last samples remain visible while the plot stream reopens.",
  });
  assert.deepEqual(stated, {
    label: "Reconnecting",
    detail: "The last samples remain visible while the plot stream reopens.",
  });

  // A Plot Kind May Word Its Own Detail But Keeps The Shared Label
  const worded = plotPlaceholderText({}, "No categorical samples are available.");
  assert.equal(worded.label, WAITING_FOR_SAMPLES);
  assert.equal(worded.detail, "No categorical samples are available.");

  // A Plot The Backend Could Not Produce Says So Instead Of Waiting
  assert.deepEqual(plotUnavailableText("Field voltage_v is not numeric."), {
    label: "Plot Unavailable",
    detail: "Field voltage_v is not numeric.",
  });
});
