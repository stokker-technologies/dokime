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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

// Local Imports
import { LivePlotGroupGrid } from "@/components/document/PlotGroupGrid";
import { StatsSummaryTable } from "@/components/StatsSummaryTable";
import { plotWindowOptions } from "@/features/cycle/PlotWindowSlider";
import { formatUnitLabel } from "@/lib/format";
import {
  formatStatValue,
  formatSummaryValue,
  nearestDrawnPoint,
  type SeriesSummary,
  seriesLegendColumns,
  seriesLegendColumnWidth,
  seriesLegendRows,
  statsSummaryCells,
  summaryInFlight,
  toggleTrace,
} from "@/lib/statistics";
import type { DocumentViewPlot, HubRecord, PlotPayload, PlotPayloadStreamMessage, TelemetryColumnSpec } from "@/lib/types";
import { formatBooleanState, formatCursorX } from "@/plots/axis";
import { TimeSeriesPlot } from "@/plots/catalog/TimeSeriesPlot";
import { TrajectoryPlot } from "@/plots/catalog/TrajectoryPlot";
import { XyIntervalPlot } from "@/plots/catalog/XyIntervalPlot";
import { XyLinePlot } from "@/plots/catalog/XyLinePlot";
import { XyScatterPlot } from "@/plots/catalog/XyScatterPlot";
import { CartesianPlot } from "@/plots/CartesianPlot";
import { documentPlotSpec, groupRepeatsPlotTitle } from "@/plots/grouping";
import {
  HISTOGRAM_COUNT_LABEL,
  type HistogramColumn,
  type HistogramDeclaration,
  histogramAxes,
  histogramAxisTitle,
} from "@/plots/histogram-axes";
import { histogramBars } from "@/plots/histogram-bins";
import { plotRequestForDocumentPlot } from "@/plots/live-data";
import { PlotInteractionProvider } from "@/plots/interaction";
import { applyPendingNotice, applyPlotMessages, emptyLivePlotState, plotWaitingLabel } from "@/plots/live-frames";
import { NO_NUMERIC_SAMPLES, plotPlaceholderText, plotUnavailableText, STREAM_ERROR, WAITING_FOR_SAMPLES } from "@/plots/plot-placeholder";
import { promptMarkerLabel, promptMarkers } from "@/plots/prompt-markers";
import { m4Indices, windowIndices } from "@/plots/reduce";
import { attachPan, heldViewport, PLOT_DRAG_OPTIONS, pannedRange } from "@/plots/viewport";
import { readReportPayload } from "@/report/report-data";

/* === Re-Exports === */

/* === Constants === */

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

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

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
    epoch: "epoch",
    selection: "pulse-selection",
    payload,
    warnings: [],
  } as unknown as PlotPayloadStreamMessage;
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

/* === Entry Point === */

test("Configured Plot Windows Preserve Inheritance And All", () => {
  const options = plotWindowOptions([17, 42]);
  assert.deepEqual(
    options.map((option) => option.value),
    [undefined, 17, 42, null],
  );
  const structural = plotWindowOptions([]);
  assert.deepEqual(
    structural.map((option) => option.value),
    [undefined, null],
  );
});

test("Empty Snapshot", () => {
  // A Plot Without Records Opens On Its Revision-0 Snapshot Instead Of Waiting
  const empty = hostFrame([], [[]]);
  const state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 0, empty)]);
  assert.equal(state.status, "live");
  assert.equal(state.droppedBatches, 0);

  // A Replacement That Repeats The Revision Is Still Dropped
  assert.equal(applyPlotMessages(state, [streamMessage("replace", 0, empty)]).droppedBatches, 1);
});

test("Window Resampling", () => {
  /* A 50 Hz sine sampled at 1 kHz for 60 s, drawn 800 CSS px wide. */
  const rate = 1_000;
  const x = Array.from({ length: 60 * rate }, (_, index) => index / rate);
  const sine = x.map((t) => Math.sin(2 * Math.PI * 50 * t));
  const columns = 800;

  // The Whole Run Is Cut To Four Samples Per Pixel Column, Ends Included
  const whole = windowIndices(x, [sine], null, columns);
  assert.ok(whole);
  assert.ok(whole.length <= 4 * columns + 2);
  assert.equal(whole[0], 0);
  assert.equal(whole.at(-1), x.length - 1);
  assertAscending(whole);

  // A 15 s Zoom Resamples The Window Itself And Keeps The Envelope Of Every Column
  const zoom = { min: 20, max: 35 };
  const zoomed = windowIndices(x, [sine], zoom, columns);
  assert.ok(zoomed);
  assertAscending(zoomed);
  const inside = zoomed.filter((index) => x[index] >= zoom.min && x[index] <= zoom.max);
  assert.ok(inside.length > 3 * columns, "the window gets the whole budget, not a slice of the run's");
  const span = (zoom.max - zoom.min) / columns;
  for (let column = 0; column < columns; column += 10) {
    const from = zoom.min + column * span;
    const to = from + span;
    const truth = sine.filter((_, index) => x[index] >= from && x[index] < to);
    const kept = inside.filter((index) => x[index] >= from && x[index] < to).map((index) => sine[index]);
    assert.equal(Math.max(...kept), Math.max(...truth), `column ${column} keeps its peak`);
    assert.equal(Math.min(...kept), Math.min(...truth), `column ${column} keeps its trough`);
  }
  // The Line Reaches Both Frame Edges And A Reset Still Fits The Whole Run
  assert.ok(zoomed.includes(Math.round(zoom.min * rate) - 1));
  assert.ok(zoomed.includes(Math.round(zoom.max * rate) + 1));
  assert.equal(zoomed[0], 0);
  assert.equal(zoomed.at(-1), x.length - 1);

  // A Window Within The Budget Draws Every Sample, A Short Run Is Never Reduced
  const second = windowIndices(x, [sine], { min: 10, max: 11 }, columns);
  assert.ok(second);
  for (let index = 10 * rate; index <= 11 * rate; index += 1) assert.ok(second.includes(index));
  assert.equal(windowIndices(x.slice(0, 3_000), [sine.slice(0, 3_000)], null, columns), null);

  // M4 Keeps First, Lowest, Highest And Last Per Bucket And Skips Gaps
  assert.deepEqual(m4Indices([0, 1, 2, 3, 4, 5], [3, 9, null, 1, 4, 5], 0, 6, 1), [0, 1, 3, 5]);
  assert.deepEqual(m4Indices([0, 1, 2], [null, null, null], 0, 3, 1), []);

  // Traces On One Axis Are Resampled Together
  const cosine = x.map((t) => Math.cos(2 * Math.PI * 50 * t));
  const fitted = windowIndices(x, [sine, cosine], null, columns, true);
  assert.ok(fitted && fitted.length <= columns);
  for (const trace of [sine, cosine]) {
    assert.equal(Math.min(...fitted.map((index) => trace[index])), Math.min(...trace));
    assert.equal(Math.max(...fitted.map((index) => trace[index])), Math.max(...trace));
  }
  const both = windowIndices(x, [sine, cosine], zoom, columns);
  assert.ok(both);
  assert.ok(zoomed.every((index) => both.includes(index)));

  // A Group Titled Like Its Only Plot Prints The Title Once
  const plot = { title: "AC Voltage" } as Parameters<typeof groupRepeatsPlotTitle>[1][number];
  assert.equal(groupRepeatsPlotTitle("AC Voltage", [plot]), true);
  assert.equal(groupRepeatsPlotTitle("Mains", [plot]), false);
  assert.equal(groupRepeatsPlotTitle("AC Voltage", [plot, plot]), false);
});

test("Prompt Markers", () => {
  const record = (record_type: string, timestamp: string, details: Record<string, unknown>, message?: string) =>
    ({
      details,
      display: { icon: "clock", tone: "neutral" },
      level: "info",
      message,
      record_id: `${record_type}-${timestamp}`,
      record_type,
      source: "runtime",
      timestamp,
    }) as unknown as HubRecord;
  const records = [
    record("prompt.response.submitted", "2026-10-20T20:04:30Z", { prompt_id: "p-1" }, "Prompt response submitted."),
    record("prompt.request.confirmation", "2026-10-20T20:04:12Z", { prompt_id: "p-1", title: "Connect The Load" }, "Connect The Load"),
    record("prompt.request.form", "2026-10-20T20:05:00Z", { prompt_id: "p-2", title: "Enter Serial" }),
    record("prompt.response.cancelled", "2026-10-20T20:05:10Z", { prompt_id: "p-2" }),
    record("prompt.request.choice", "not a time", { prompt_id: "p-3", title: "Pick" }),
    record("step.started", "2026-10-20T20:04:00Z", {}),
  ];

  // Requests Mark When A Prompt Was Asked And Submissions When It Was Answered, In Time Order
  const markers = promptMarkers(records);
  assert.deepEqual(
    markers.map((marker) => [marker.kind, marker.x, marker.title]),
    [
      ["asked", Date.parse("2026-10-20T20:04:12Z") / 1000, "Connect The Load"],
      ["answered", Date.parse("2026-10-20T20:04:30Z") / 1000, "Connect The Load"],
      ["asked", Date.parse("2026-10-20T20:05:00Z") / 1000, "Enter Serial"],
    ],
  );

  // The Hover Names The Moment And The Prompt's Title
  assert.equal(promptMarkerLabel(markers[0]), "Prompt asked: Connect The Load");
  assert.equal(promptMarkerLabel(markers[1]), "Prompt answered: Connect The Load");

  // An Answer Whose Request Is Out Of Scope Still Marks, Under A Generic Title
  assert.deepEqual(
    promptMarkers([records[0]]).map((marker) => marker.title),
    ["Prompt"],
  );
  assert.deepEqual(promptMarkers([]), []);
});

test("Waiting For Data", () => {
  const opened = { voltage: { ...emptyLivePlotState(), status: "loading" as const } };

  // The Watch's Acknowledgement Turns A Loading Plot Into A Quiet Wait, Never An Error
  const acknowledged = applyPendingNotice(opened, "voltage", 1_000);
  assert.equal(acknowledged.voltage.status, "waiting");
  assert.equal(acknowledged.voltage.error, null);
  assert.equal(plotWaitingLabel(acknowledged.voltage), "Waiting For Data \u00b7 0 s");

  // Each Keepalive Counts From The First Notice, However Long The First Read Takes
  const later = applyPendingNotice(acknowledged, "voltage", 46_000);
  assert.equal(later.voltage.status, "waiting");
  assert.equal(plotWaitingLabel(later.voltage), "Waiting For Data \u00b7 45 s");

  // A Notice For A Plot This Watch Does Not Hold Changes Nothing
  assert.equal(applyPendingNotice(later, "current", 50_000), later);

  // A Server Error Frame Still Reports, And A Later Notice Does Not Hide It
  const failed = {
    voltage: applyPlotMessages(later.voltage, [{ ...streamMessage("error", 0, hostFrame([], [])), warnings: ["plot query unavailable"] }]),
  };
  assert.equal(failed.voltage.status, "error");
  assert.equal(applyPendingNotice(failed, "voltage", 60_000), failed);

  // The First Frame Ends The Wait
  const served = applyPlotMessages(later.voltage, [streamMessage("snapshot", 0, hostFrame([0, 1], [[1, 2]]))]);
  assert.equal(served.status, "live");
  assert.equal(served.waitingSince, null);
  assert.equal(applyPendingNotice({ voltage: served }, "voltage", 70_000).voltage, served);
});

test("Slider Selects Storage Query Range", () => {
  const plot: DocumentViewPlot = {
    plot_id: "pulse",
    title: "Pulse",
    stream: "pulse",
    plot_type: "time_series",
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    x: "timestamp",
    y: ["voltage"],
    live_window_s: 60,
    display_points: 100,
  };
  const scope = { test_cycle_id: "cycle" };
  const declared = plotRequestForDocumentPlot(plot, { scope });
  const recent = plotRequestForDocumentPlot(plot, { scope, windowS: 10 });
  const all = plotRequestForDocumentPlot(plot, { scope, windowS: null });
  assert.equal(declared.windowS, 60);
  assert.equal(recent.windowS, 10);
  assert.equal(all.windowS, null);
  assert.equal(all.maxPoints, recent.maxPoints);
  assert.deepEqual(all.scope, scope);
  assert.equal(plot.live_window_s, 60);
});
test("Storage Query Snapshots", () => {
  const recent = hostFrame([100, 101], [[2, 3]], { windowS: 30 });
  recent.markers = [{ x: 101, label: "Recent", record_type: "recent", color: "blue" }];
  recent.context = { recent: true };
  let state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 0, recent)]);
  assert.deepEqual(state.payload?.cartesian?.series[0].x, [100, 101]);
  assert.ok(state.payload?.series.every((series) => series.points.length === 0));

  // A Broader Storage Query Can Restore Samples Absent From The Previous Frame.
  const historical = hostFrame([0, 50, 101], [[4, 5, 6]], { windowS: null });
  historical.markers = [{ x: 0, label: "Earlier", record_type: "earlier", color: "red" }];
  historical.context = { historical: true };
  state = applyPlotMessages(state, [streamMessage("replace", 1, historical)]);
  assert.deepEqual(state.payload?.cartesian?.series[0].x, [0, 50, 101]);
  assert.deepEqual(state.payload?.markers, historical.markers);
  assert.deepEqual(state.payload?.context, historical.context);
  assert.equal(state.payload?.cartesian?.point_count, 3);
  assert.equal(state.payload?.series[0].raw_points, 3);
  assert.deepEqual(recent.cartesian?.series[0].x, [100, 101]);

  // A Late Revision Cannot Replace The Selected Snapshot.
  const selected = state.payload;
  state = applyPlotMessages(state, [streamMessage("replace", 1, recent)]);
  assert.equal(state.payload, selected);
  assert.equal(state.droppedBatches, 1);
  const differentSelection = { ...streamMessage("replace", 2, recent), selection: "different-selection" };
  assert.equal(applyPlotMessages(state, [differentSelection]).payload, selected);

  // A Reopened Watch Begins A New Revision Origin Without Blending Old Data.
  const stale = { ...state, status: "reconnecting" as const };
  assert.equal(applyPendingNotice({ pulse: stale }, "pulse", 1_000).pulse.payload, selected);
  const reopened = hostFrame([200], [[8]]);
  state = applyPlotMessages(stale, [{ ...streamMessage("snapshot", 0, reopened), epoch: "reopened-epoch" }]);
  assert.equal(state.epoch, "reopened-epoch");
  assert.equal(state.selection, "pulse-selection");
  assert.equal(state.status, "live");
  assert.deepEqual(state.payload?.cartesian?.series[0].x, [200]);
  assert.equal(state.payload?.series[0].raw_points, 1);
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
  assert.match(histogramAxisTitle(axes.x), / \[V\]$/);
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
  assert.match(histogramAxisTitle(field.x), / \[s\]$/);
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

test("Boolean State Plots", () => {
  const payload = hostFrame([0, 1, 2, 3], [[0, 1, null, 0]], { keys: ["internet"] });
  payload.stream = "host.network";
  const descriptor = {
    plot_id: "connectivity",
    title: "Internet Connectivity",
    stream: "host.network",
    plot_type: "time_series",
    x: "timestamp",
    y: ["internet"],
  } as DocumentViewPlot;
  const columns: TelemetryColumnSpec[] = [{ name: "internet", value_type: "boolean", role: "state", required: false }];
  const telemetry = {
    "host.network": {
      stream: "host.network",
      spec: {
        stream: "host.network",
        columns,
        allow_extra_fields: false,
      },
      summary: {},
      samples: [],
      max_samples: 0,
      total_samples: 4,
      dropped_samples: 0,
      updated_at: payload.generated_at,
    },
  };
  const plot = documentPlotSpec({ plot: descriptor, payload, telemetry, title: descriptor.title, position: 0 });
  assert.equal(plot.booleanAxis, true);
  assert.deepEqual(payload.cartesian?.series[0].y.map(formatBooleanState), ["False", "True", "--", "False"]);
  assert.equal(formatBooleanState(0.5), "--");
  for (const zoom of [false, true]) {
    const html = renderToStaticMarkup(createElement(PlotInteractionProvider, { zoom }, createElement(TimeSeriesPlot, { plot })));
    assert.match(html, /plot-follow/);
    assert.doesNotMatch(html, /<table\b/);
  }
  columns[0].value_type = "number";
  assert.equal(documentPlotSpec({ plot: descriptor, payload, telemetry, title: descriptor.title, position: 0 }).booleanAxis, false);
});

test("Plot Statistics", () => {
  const payload = hostFrame([0, 1, 2], [[2, 5, 11]], { keys: ["distance"] });
  payload.summaries = [{ ...SUMMARIES[0], series_id: "distance" }];
  const descriptor = {
    plot_id: "distance",
    title: "Distance",
    stream: "pulse",
    plot_type: "time_series",
    x: "timestamp",
    y: ["distance"],
  } as DocumentViewPlot;
  const plot = documentPlotSpec({ plot: descriptor, payload, title: descriptor.title, position: 0 });
  const interval = {
    ...plot,
    analysis: {
      kind: "sigma_interval" as const,
      series: [{ key: "distance", raw_points: 3, bins: [{ x: 1, count: 3, mean: 6, lower: 2, upper: 11 }] }],
    },
  };
  for (const zoom of [false, true]) {
    const render = (child: ReturnType<typeof createElement>) =>
      renderToStaticMarkup(createElement(PlotInteractionProvider, { zoom }, child));
    const time = render(createElement(TimeSeriesPlot, { plot }));
    assert.match(time, /<thead\b/);
    for (const Component of [XyScatterPlot, XyLinePlot, TrajectoryPlot]) {
      const xy = render(createElement(Component, { plot: { ...plot, timeAxis: false } }));
      assert.match(xy, /plot-follow/);
      assert.doesNotMatch(xy, /<table\b/);
    }
    const xy = render(createElement(XyIntervalPlot, { plot: interval }));
    assert.match(xy, /plot-follow/);
    assert.doesNotMatch(xy, /<table\b/);
  }
});

test("Series Legend", () => {
  // Live And Report Legends List The Trace, The Cursor And The Statistics With The Unit In Each Header
  const plain = seriesLegendColumns(null);
  for (const unit of ["V", "M\u03a9", "Mbps"]) {
    const columns = seriesLegendColumns(unit);
    assert.deepEqual(
      columns.map((column) => column.id),
      plain.map((column) => column.id),
    );
    columns.forEach((column, index) => {
      const suffix = column.header.slice(plain[index].header.length);
      assert.equal(suffix, ["trace", "xCursor", "count"].includes(column.id) ? "" : ` [${unit}]`);
      assert.doesNotMatch(column.header, /[\r\n]/);
    });
    const html = renderToStaticMarkup(
      createElement(StatsSummaryTable, {
        legend: { columns, rows: [], pending: false },
      }),
    );
    const headings = html.match(/<thead\b[\s\S]*?<\/thead>/)?.[0] ?? "";
    assert.match(headings, /class="[^"]*truncate[^"]*" title="[^"<>]* \[/);
    assert.doesNotMatch(headings, /whitespace-normal|overflow-wrap:anywhere/);
  }
  assert.equal(formatUnitLabel("fixture", "  "), "fixture");
  assert.equal(formatUnitLabel("fixture", " ms "), "fixture [ms]");
  // Widths Depend On The Column Alone, So Every Table On A Page Lines Up Whatever Its Unit
  const widths = (unit: string | null) => seriesLegendColumns(unit).map((column) => seriesLegendColumnWidth(column.id));
  assert.deepEqual(widths("dBm"), widths(null));
  assert.deepEqual(widths("M\u03a9"), widths("V"));
  assert.deepEqual(widths(null).slice(1), ["w-52", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32"]);

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

  // On A Time Axis The X Cursor Reads As The Local Date And Clock, To What One Pixel Resolves
  const evening = new Date(2026, 9, 20, 22, 4, 12, 345).getTime() / 1000;
  const timeAxis = { timeAxis: true, xFormat: null, xElapsedUnit: null, origin: {}, widthPx: 600 };
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 3_600 }), "10/20/26 10:04:12 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 15 }), "10/20/26 10:04:12.35 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 1 }), "10/20/26 10:04:12.345 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 120 }), "10/20/26 10:04:12.3 PM");
  assert.equal(formatCursorX(new Date(2026, 9, 21, 0, 0, 5).getTime() / 1000, { ...timeAxis, span: 3_600 }), "10/21/26 12:00:05 AM");
  // Elapsed And Numeric Axes Keep Their Own Readouts, Never A Date
  assert.equal(formatCursorX(12.5, { ...timeAxis, xElapsedUnit: "s", span: 60 }), "12.5 s");
  assert.equal(formatCursorX(3, { ...timeAxis, timeAxis: false, span: 6_000 }), "3");
  const formatted = seriesLegendRows({
    cursorPoints: [{ x: evening, y: 1 }, null],
    formatX: (value) => formatCursorX(value, { ...timeAxis, span: 15 }),
    hidden: new Set(),
    summaries: SUMMARIES,
    traces: TRACES,
  });
  assert.equal(formatted[0].xCursor, "10/20/26 10:04:12.35 PM");

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
    ["Mean [V]", "12.50"],
    ["Median [V]", "12.00"],
    ["Std [V]", "0.25"],
    ["RMS [V]", "12.60"],
    ["Min [V]", "11.00"],
    ["Max [V]", "14.00"],
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
  // It Reads As A Failure, Not A Wait: The Error Tone Tints The Full Plot Area
  assert.deepEqual(plotUnavailableText("Field voltage_v is not numeric."), {
    label: "Plot Unavailable",
    detail: "Field voltage_v is not numeric.",
    tone: "error",
  });

  // A Stream Error With Nothing Drawn Takes The Same Tone; Waiting And Reconnecting Stay Quiet
  assert.equal(plotPlaceholderText({ stateLabel: STREAM_ERROR, stateDetail: "plot query unavailable" }).tone, "error");
  assert.equal(stated.tone, undefined);
  assert.equal(plotPlaceholderText({}).tone, undefined);
});

test("Stream Errors Replace Retained Charts", () => {
  const descriptor: DocumentViewPlot = {
    plot_id: "pulse",
    title: "Pulse",
    stream: "pulse",
    plot_type: "time_series",
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    x: "timestamp",
    y: ["trace-0"],
  };
  const payload = hostFrame([0, 1], [[2, 3]]);
  const ready = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 1, payload)]);
  const failed = applyPlotMessages(ready, [{ ...streamMessage("error", 1, payload), warnings: ["Voltage channel disconnected"] }]);
  assert.equal(failed.payload, ready.payload, "recovery retains the last frame in memory");
  const html = renderToStaticMarkup(
    createElement(LivePlotGroupGrid, {
      documentPlots: [descriptor],
      liveStates: { pulse: failed },
      title: "Signals",
    }),
  );
  assert.match(html, /role="alert"/);
  assert.match(html, /Voltage channel disconnected/);
  assert.doesNotMatch(html, /Trace Statistics/);
});

test("Viewport Panning", () => {
  assert.deepEqual(pannedRange({ min: 0, max: 100 }, -0.25, false), { min: -25, max: 75 });
  const logarithmic = pannedRange({ min: 1, max: 100 }, 0.5, true);
  assert.ok(logarithmic);
  assert.ok(Math.abs(logarithmic.min - 10) <= Number.EPSILON * 10 * 8);
  assert.ok(Math.abs(logarithmic.max - 1000) <= Number.EPSILON * 1000 * 8);
  assert.equal(pannedRange({ min: 0, max: 100 }, 0.5, true), null);
  assert.equal(pannedRange({ min: 1, max: 100 }, Infinity, false), null);
});

test("Pan Listener Ownership", () => {
  const view = new EventTarget();
  const over = Object.assign(new EventTarget(), { clientWidth: 100, clientHeight: 100, ownerDocument: { defaultView: view } });
  const scales = { x: { min: 0, max: 100 }, y: { min: 0, max: 10 } };
  const setScale = vi.fn((axis: "x" | "y", range: { min: number; max: number }) => {
    scales[axis] = range;
  });
  const onPan = vi.fn();
  const setCursor = vi.fn();
  const plot = {
    over,
    scales,
    setScale,
    cursor: { left: 75, top: 70 },
    setCursor,
    batch: (effect: () => void) => effect(),
  } as unknown as Parameters<typeof attachPan>[0];
  const detach = attachPan(plot, { logX: false, logY: false, panY: true, onPan });
  const mouse = (type: string, values: Partial<MouseEvent>) => Object.assign(new Event(type, { cancelable: true }), values);
  const ordinary = mouse("mousedown", { button: 0, shiftKey: false, clientX: 50, clientY: 50 });
  over.dispatchEvent(ordinary);
  assert.equal(ordinary.defaultPrevented, false);
  const shifted = mouse("mousedown", { button: 0, shiftKey: true, clientX: 50, clientY: 50 });
  over.dispatchEvent(shifted);
  assert.equal(shifted.defaultPrevented, true);
  view.dispatchEvent(mouse("mousemove", { buttons: 1, clientX: 75, clientY: 70 }));
  assert.deepEqual(scales, { x: { min: -25, max: 75 }, y: { min: 2, max: 12 } });
  assert.deepEqual(onPan.mock.calls[0][0], { x: scales.x, y: scales.y });
  assert.deepEqual(setCursor.mock.calls[0], [{ left: 75, top: 70 }]);
  view.dispatchEvent(new Event("mouseup"));
  view.dispatchEvent(mouse("mousemove", { buttons: 1, clientX: 90, clientY: 90 }));
  assert.equal(onPan.mock.calls.length, 1);
  over.dispatchEvent(mouse("mousedown", { button: 1, shiftKey: false, clientX: 10, clientY: 50 }));
  view.dispatchEvent(mouse("mousemove", { buttons: 4, clientX: 0, clientY: 50 }));
  assert.deepEqual(scales.x, { min: -15, max: 85 });
  view.dispatchEvent(new Event("blur"));
  assert.equal(onPan.mock.calls.length, 2);
  detach();
  const removed = mouse("mousedown", { button: 1, clientX: 10, clientY: 50 });
  over.dispatchEvent(removed);
  assert.equal(removed.defaultPrevented, false);
  view.dispatchEvent(mouse("mousemove", { buttons: 4, clientX: 0, clientY: 50 }));
  assert.equal(onPan.mock.calls.length, 2);
});

test("Declared Occupancy Context", () => {
  const plot: DocumentViewPlot = {
    plot_id: "occupancy",
    title: "Occupancy",
    stream: "fixture.scan",
    plot_type: "wifi_channel_occupancy",
    style: "points",
    x_scale: "linear",
    y_scale: "linear",
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    x: "timestamp",
    y: ["network_count"],
  };
  const telemetry = {
    "fixture.scan": {
      stream: "fixture.scan",
      summary: {},
      samples: [{ active_wifi_channel: 36, active_wifi_band: "5 GHz", active_wifi_ssid: "scan" }],
      max_samples: 1,
      total_samples: 1,
      dropped_samples: 0,
      updated_at: "2026-10-29T00:00:00Z",
    },
    "host.wifi.link": {
      stream: "host.wifi.link",
      summary: {},
      samples: [{ wifi_channel: 6, wifi_ssid: "link" }],
      max_samples: 1,
      total_samples: 1,
      dropped_samples: 0,
      updated_at: "2026-10-29T00:00:00Z",
    },
  };
  const spec = documentPlotSpec({ plot, telemetry, payload: null, title: "Occupancy", position: 0 });
  assert.deepEqual(spec.context, { active_wifi_channel: 36, active_wifi_band: "5 GHz", active_wifi_ssid: "scan" });
  delete (telemetry as Partial<typeof telemetry>)["fixture.scan"];
  const absent = documentPlotSpec({ plot, telemetry, payload: null, title: "Occupancy", position: 0 });
  assert.deepEqual(absent.context, {});
});

test("Bounded Report Envelopes", () => {
  for (const values of [
    [[3]],
    [[3, -7]],
    [
      [3, -7, 11],
      [9, null, 4],
    ],
  ]) {
    const x = values[0].map((_, index) => 1_798_761_600 + index);
    const payload = hostFrame(x, values);
    const series = payload.series.map((series) => ({ ...series, unit: "dBm", raw_points: 20_000, downsampled: true }));
    const html = renderToStaticMarkup(
      createElement(CartesianPlot, {
        title: "Signal",
        series,
        cartesian: payload.cartesian,
        unit: "dBm",
        contextSummary: "fixture-network / radio-2 / 6",
      }),
    );
    assert.ok(html.includes("plot-follow"));
    assert.ok(html.includes("[dBm]"));
    assert.ok(html.includes("fixture-network / radio-2 / 6"));
    const missingFrame = renderToStaticMarkup(createElement(CartesianPlot, { title: "Signal", series }));
    assert.ok(!missingFrame.includes("plot-follow"));
    assert.deepEqual(windowIndices(x, values, null, 640, true), null);
  }
});

test("Report Metadata Subtitles", () => {
  const captured = hostFrame([1, 2], [[-61, -59]]);
  captured.context = {
    wifi_ssid: { type: "text", value: "fixture-network" },
    wifi_radio: { type: "text", value: "radio-2" },
    wifi_channel: { type: "integer", value: "6" },
  };
  const textContent = JSON.stringify({ plots: { signal: captured }, case_runs: [{ plots: { signal: captured } }] });
  vi.stubGlobal("document", { getElementById: () => ({ textContent }) });
  let payload: PlotPayload;
  try {
    const report = readReportPayload();
    payload = report.case_runs[0].plots.signal;
    assert.deepEqual(payload.context, { wifi_ssid: "fixture-network", wifi_radio: "radio-2", wifi_channel: 6 });
    assert.deepEqual(report.plots.signal.context, payload.context);
  } finally {
    vi.unstubAllGlobals();
  }
  const plot = {
    plot_id: "fixture.rssi",
    stream: "host.wifi.link",
    title: "Wi-Fi RSSI",
    plot_type: "time_series",
    y: ["rssi"],
  } as DocumentViewPlot;
  const spec = documentPlotSpec({ plot, payload, plotPalette: [], position: 0, title: plot.title });
  assert.ok(spec.contextSummary?.includes("fixture-network"));
  assert.ok(spec.contextSummary?.includes("radio-2"));
  const html = renderToStaticMarkup(
    createElement(CartesianPlot, {
      title: spec.title,
      series: spec.series,
      cartesian: spec.cartesian,
      contextSummary: spec.contextSummary,
    }),
  );
  assert.ok(html.includes("plot-follow"));
  assert.ok(html.includes("fixture-network"));
});
