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
  nearestDrawnPoint,
  type SeriesSummary,
  seriesLegendColumns,
  seriesLegendRows,
  statsSummaryCells,
  summaryInFlight,
  toggleTrace,
} from "@/lib/statistics";
import type { PlotPayload, PlotPayloadStreamMessage } from "@/lib/types";
import {
  HISTOGRAM_COUNT_LABEL,
  type HistogramColumn,
  type HistogramDeclaration,
  histogramAxes,
  histogramAxisTitle,
} from "@/plots/histogram-axes";
import { histogramBars } from "@/plots/histogram-bins";
import { applyPlotMessages, emptyLivePlotState, type LivePlotState } from "@/plots/live-frames";
import { NO_NUMERIC_SAMPLES, plotPlaceholderText, plotUnavailableText, WAITING_FOR_SAMPLES } from "@/plots/plot-placeholder";
import { lttbIndices, minMaxIndices, NATIVE_PLOT_MAX_POINTS, reduceIndices } from "@/plots/reduce";
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

/* A host frame of aligned series on one x axis, as the stream decodes it. */
function hostFrame(
  x: number[],
  ys: (number | null)[][],
  fields: { maxPoints?: number; windowS?: number | null; keys?: string[] } = {},
): PlotPayload {
  return {
    stream: "pulse",
    x_key: "timestamp",
    generated_at: "2026-10-16T00:00:00Z",
    max_points: fields.maxPoints ?? 100,
    window_s: fields.windowS ?? null,
    series: ys.map((y, index) => ({
      key: fields.keys?.[index] ?? `trace-${index}`,
      label: `Trace ${index}`,
      points: x.map((value, at) => ({ x: value, y: y[at] })),
      raw_points: x.length,
      downsampled: false,
    })),
    cartesian: {
      mode: "aligned",
      series: ys.map((y) => ({ x: [...x], y: [...y] })),
      point_count: ys.reduce((count, y) => count + y.filter((value) => value !== null).length, 0),
      signature: "host",
    },
  } as unknown as PlotPayload;
}

function streamMessage(kind: PlotPayloadStreamMessage["kind"], revision: number, payload: PlotPayload): PlotPayloadStreamMessage {
  return {
    kind,
    revision,
    plot_id: "pulse",
    payload,
    cursor: null,
    warnings: [],
  } as unknown as PlotPayloadStreamMessage;
}

/* Streams `chunks` appends of `size` pulse samples after `state`, 10 Hz from `startS`. */
function appendPulses(
  state: LivePlotState,
  chunks: number,
  size: number,
  fields: { maxPoints?: number; windowS?: number | null; startS?: number; traces?: number },
  onFrame?: (state: LivePlotState) => void,
): LivePlotState {
  let next = state;
  for (let chunk = 0; chunk < chunks; chunk += 1) {
    const { x, y } = pulse(size, (fields.startS ?? 0) + (chunk * size * PULSE_PERIOD_MS) / 1000);
    const ys = Array.from({ length: fields.traces ?? 1 }, (_, trace) => y.map((v) => v + trace));
    const update = hostFrame(x, ys, fields);
    next = applyPlotMessages(next, [streamMessage("append", next.revision + 1, update)], "auto");
    onFrame?.(next);
  }
  return next;
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
  /* Buckets are equal spans of time, so the silence holds no picks. */
  const keptX = gapKept.map((index) => gapX[index]);
  assert.ok(
    keptX.some((value) => value < 30),
    "keeps samples before the gap",
  );
  assert.ok(
    keptX.some((value) => value >= 600),
    "keeps samples after the gap",
  );
  assert.ok(gapKept.length <= 60);
  assertAscending(gapKept);

  // LTTB Never Picks A Sample Without A Finite Y
  const sparse = pulse(400);
  const gapped = sparse.y.map((value, index) => (index % 7 === 3 ? null : value));
  for (const index of lttbIndices(sparse.x, gapped, 50).slice(1, -1)) {
    assert.notEqual(gapped[index], null, `picked a null at ${index}`);
  }

  // Min/Max Keeps Each Bucket's Extrema In Index Order
  /* count 4 is two equal spans of x holding four samples each. */
  const ramp = [0, 1, 2, 3, 4, 5, 6, 7];
  assert.deepEqual(minMaxIndices(ramp, [1, 5, 3, 0, 2, 9, 4, 4], 4), [1, 3, 4, 5]);

  // Reduce Indices Honours The Technique Within The Budget
  const { x, y } = pulse(500);
  assert.equal(reduceIndices(x, [y], 500, "lttb"), null);
  assert.equal(reduceIndices(x, [y], 100, "lttb")?.length, 100);
  const stride = reduceIndices(x, [y], 100, "raw");
  assert.ok(stride && stride.length <= 100 && stride[0] === 0 && stride.at(-1) === 499);
  const envelope = reduceIndices(x, [y], 100, "min_max");
  assert.ok(envelope && envelope.length <= 100);

  // Aligned Series Share One Union Of Picks
  const inverted = y.map((value) => -value);
  const shared = reduceIndices(x, [y, inverted], 100, "lttb");
  assert.ok(shared && shared.length >= 100 && shared.length <= 200);
  assertAscending(shared);

  // Auto Switches To An Envelope Only Above Four Times The Display Budget
  const autoX = Array.from({ length: 41 }, (_, index) => index);
  const autoY = autoX.map((value) => Math.sin(value));
  const budget = 10;
  const moderateX = autoX.slice(0, 40);
  const moderateY = autoY.slice(0, 40);
  assert.deepEqual(reduceIndices(moderateX, [moderateY], budget, "auto"), reduceIndices(moderateX, [moderateY], budget, "lttb"));
  assert.deepEqual(reduceIndices(autoX, [autoY], budget, "auto"), reduceIndices(autoX, [autoY], budget, "min_max"));
});

test("Retention Window", () => {
  // A Windowed Plot Keeps Exactly Its Trailing Window While Appends Run
  /* 300 s at 10 Hz into a 60 s window: the front is evicted as the run moves. */
  const opening = pulse(10);
  const snapshot = hostFrame(opening.x, [opening.y], { windowS: 60 });
  let state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 1, snapshot)], "auto");
  state = appendPulses(state, 300, 10, { windowS: 60, startS: 1 });
  const x = state.payload?.cartesian?.series[0].x ?? [];
  const span = x[x.length - 1] - x[0];
  assert.ok(span <= 60 && span >= 59.8, `the window decides the span, got ${span.toFixed(1)} s`);
  assert.equal(state.payload?.cartesian?.point_count, x.length);
});

test("Empty Snapshot", () => {
  // A Plot Without Records Opens On Its Revision-0 Snapshot Instead Of Waiting
  const empty = hostFrame([], [[]]);
  const state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 0, empty)], "auto");
  assert.equal(state.status, "live");
  assert.equal(state.droppedBatches, 0);

  // An Append That Repeats The Revision Is Still Dropped
  assert.equal(applyPlotMessages(state, [streamMessage("append", 0, empty)], "auto").droppedBatches, 1);
});

test("Frame Retention", () => {
  // A Snapshot Keeps One Representation: The Cartesian Frame
  const start = pulse(50);
  const snapshot = hostFrame(start.x, [start.y, start.y], { maxPoints: 100 });
  let state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 1, snapshot)], "auto");
  assert.equal(state.payload?.cartesian?.signature, "host");
  assert.ok(state.payload?.series.every((series) => series.points.length === 0));

  // A Long Run Under The "All" Window Stays Within Twice Its Capacity
  /* 20,000 samples per trace; two aligned traces at a budget of 100 never
     hold more than 2 x 100 x 2 samples on their shared axis. */
  let largest = 0;
  const signatures = new Set<string>();
  state = appendPulses(state, 2_000, 10, { startS: 5, traces: 2 }, (frame) => {
    const cartesian = frame.payload?.cartesian;
    assert.ok(cartesian);
    largest = Math.max(largest, cartesian.series[0].x.length);
    assert.deepEqual(cartesian.series[1].x, cartesian.series[0].x, "aligned traces share x");
    assert.equal(cartesian.series[1].y.length, cartesian.series[0].y.length);
    signatures.add(cartesian.signature);
  });
  assert.ok(largest <= 400, `retained ${largest} samples on one axis`);
  assert.equal(signatures.size, 2_000, "every append changes the signature");

  // Compaction Keeps The Whole Run At An Even Density
  const x = state.payload?.cartesian?.series[0].x ?? [];
  const first = x[0];
  const last = x[x.length - 1];
  assert.equal(first, 0, "the All window keeps the first sample");
  assert.ok(Math.abs(last - 2_004.9) < 1e-9, `the latest sample is drawn, got ${last}`);
  const early = x.filter((value) => value < (first + last) / 2).length;
  /* Between compactions the newest raw samples crowd the recent half; what
     matters is that the older half is not thinned again by every compaction,
     which an index-bucketed reduction would leave with a handful of samples. */
  assert.ok(early >= 20, `the older half keeps ${early} of ${x.length} samples`);

  // A Stale Frame Is Counted As Dropped And Changes Nothing
  const before = state.payload;
  state = applyPlotMessages(state, [streamMessage("append", 1, snapshot)], "auto");
  assert.equal(state.droppedBatches, 1);
  assert.equal(state.payload, before);

  // A Replace Frame Is Authoritative
  const replaced = pulse(5, 3_000);
  const replace = hostFrame(replaced.x, [replaced.y, replaced.y], { maxPoints: 100 });
  state = applyPlotMessages(state, [streamMessage("replace", state.revision + 1, replace)], "auto");
  assert.deepEqual(state.payload?.cartesian?.series[0].x, replaced.x);
  assert.equal(state.payload?.cartesian?.point_count, 10);
  assert.equal(state.payload?.cartesian?.signature, "host");

  // A Plot That Keeps Every Sample Is Still Bounded By The Host Ceiling
  const raw = hostFrame([0], [[0]], { maxPoints: 0 });
  let unbudgeted = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 1, raw)], "auto");
  unbudgeted = appendPulses(unbudgeted, 130, 1_000, { maxPoints: 0, startS: 1 });
  const kept = unbudgeted.payload?.cartesian?.series[0].x.length ?? 0;
  assert.equal(unbudgeted.status, "live");
  assert.ok(kept <= NATIVE_PLOT_MAX_POINTS && kept > NATIVE_PLOT_MAX_POINTS / 4);

  // A Changed Series Set Starts The Frame Over From The Update
  const renamed = hostFrame([9_000], [[1]], { keys: ["other"] });
  state = applyPlotMessages(state, [streamMessage("append", state.revision + 1, renamed)], "auto");
  assert.deepEqual(state.payload?.cartesian?.series[0].x, [9_000]);
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
  // Live And Report Legends List The Trace, The Cursor And The Statistics With The Unit In Each Header
  assert.deepEqual(
    seriesLegendColumns("V").map((column) => column.header),
    ["Trace", "X Cursor", "Y Cursor (V)", "Count", "Min (V)", "Max (V)", "Mean (V)", "Median (V)", "Std (V)", "Rms (V)"],
  );
  assert.deepEqual(
    seriesLegendColumns(null).map((column) => column.id),
    ["trace", "xCursor", "yCursor", "count", "min", "max", "mean", "median", "std", "rms"],
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
    rows.map((row) => [row.xCursor, row.yCursor]),
    [
      ["", ""],
      ["", ""],
    ],
  );

  // The Cursor Reads Each Trace's Nearest Drawn Point And Skips Its Gaps
  const x = [0, 1, 2, 3, 4];
  const voltage = [10, 11, null, 13, 14];
  const current = [null, null, null, null, null];
  assert.deepEqual(nearestDrawnPoint(x, voltage, 0.4), { x: 0, y: 10 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 0.6), { x: 1, y: 11 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 2.2), { x: 3, y: 13 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 99), { x: 4, y: 14 });
  assert.equal(nearestDrawnPoint(x, current, 1), null);
  assert.deepEqual(nearestDrawnPoint([3, 0, 2], [30, 0, 20], 0.9), { x: 0, y: 0 });
  // Live Or Report, The Cursor Cells Follow The Plot Cursor
  const cursorPoints = [nearestDrawnPoint(x, voltage, 3.1), nearestDrawnPoint(x, current, 3.1)];
  const pointed = seriesLegendRows({
    cursorPoints,
    hidden: new Set(),
    summaries: SUMMARIES,
    traces: TRACES,
  });
  assert.deepEqual(
    pointed.map((row) => [row.xCursor, row.yCursor]),
    [
      ["3.00", "13.00"],
      ["", ""],
    ],
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
