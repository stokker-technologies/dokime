// @vitest-environment happy-dom
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
import { create } from "@bufbuild/protobuf";
import { PlotCartesianMode, PlotStreamKind, SnapshotsWatchResponseSchema } from "@host/dokime/client/v1/plots/snapshots_pb.js";
import { PlotKind, Reduction, XyInterval } from "@host/dokime/v1/plots_pb.js";
import { LineStyle } from "@host/dokime/v1/plots_pb.js";

// Third Party Imports
import { act, type ComponentProps, createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import type uPlot from "uplot";
import { onTestFinished, test, vi } from "vitest";
import { Connection } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
// Local Imports
import { LivePlotGroupGrid } from "@/components/document/PlotGroupGrid";
import { StatsSummaryTable } from "@/components/StatsSummaryTable";
import { PlotWindowControl, plotWindowOptions, usePlotWindow } from "@/features/cycle/PlotWindowSlider";
import {
  mapNativePlotWatchPage,
  toNativePlotRequest,
  type NativePlotPayloadEnvelope,
  type NativePlotStreamMessage,
  type NativePlotStreamUpdate,
} from "@/lib/api/native-plots";
import { jsonToNativeValue } from "@/lib/host/values";
import * as appearance from "@/lib/appearance";
import { ApplicationThemeProvider } from "@/lib/application-theme";
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
import type { DocumentViewPlot, HubRecord, PlotPayload, PlotPayloadStreamMessage, TelemetryColumnSpec, TelemetryStream } from "@/lib/types";
import { formatBooleanState, formatCursorX } from "@/plots/axis";
import { CartesianPlot, cursorPointOptions, drawnData, facetedSeriesPaths, seriesPointMarks } from "@/plots/CartesianPlot";
import { TimeSeriesPlot } from "@/plots/catalog/TimeSeriesPlot";
import { BarPlot } from "@/plots/catalog/BarPlot";
import { PiePlot } from "@/plots/catalog/PiePlot";
import { ContourPlot } from "@/plots/catalog/ContourPlot";
import { GridChartPlot } from "@/plots/catalog/GridChartPlot";
import { TrajectoryPlot } from "@/plots/catalog/TrajectoryPlot";
import { XyIntervalPlot } from "@/plots/catalog/XyIntervalPlot";
import { XyPlot } from "@/plots/catalog/XyPlot";
import { instantTime, orderedInstants, PlotPlayback, usePlotPlayback } from "@/plots/playback";
import { documentPlotSpec, groupRepeatsPlotTitle } from "@/plots/grouping";
import {
  HISTOGRAM_COUNT_LABEL,
  type HistogramColumn,
  type HistogramDeclaration,
  histogramAxes,
  histogramAxisTitle,
} from "@/plots/histogram-axes";
import { histogramBars } from "@/plots/histogram-bins";
import { NativeApiLimitError } from "@/lib/api/native-values";
import { PlotInteractionProvider } from "@/plots/interaction";
import { plotRequestForDocumentPlot, useLivePlotPayloads } from "@/plots/live-data";
import { applyPendingNotice, applyPlotMessages, emptyLivePlotState, plotWaitingLabel } from "@/plots/live-frames";
import { drawMarkerKinds, markerKinds, markerTooltip, nearestMarker } from "@/plots/markers";
import { PlotSettingsButton } from "@/plots/PlotSettingsDialog";
import { NO_NUMERIC_SAMPLES, plotPlaceholderText, plotUnavailableText, STREAM_ERROR, WAITING_FOR_SAMPLES } from "@/plots/plot-placeholder";
import { PlotSettingsEditorContext } from "@/plots/plot-settings";
import { promptMarkerLabel, promptMarkers } from "@/plots/prompt-markers";
import { m4Indices, windowIndices } from "@/plots/reduce";
import { usePlotPalette } from "@/plots/use-plot-palette";
import { attachPan, heldViewport, PLOT_DRAG_OPTIONS, pannedRange } from "@/plots/viewport";
import { readReportPayload } from "@/report/report-data";
import { applyReportAppearance } from "@/report/report-appearance";
import { applyThemeVariables } from "@/lib/theme";
import { useSummaryWindowStore } from "@/stores/summary-window-store";

/* === Re-Exports === */

/* === Constants === */

/* A 400 x 200 plot area over x in [0, 100] and y in [0, 50], y growing upward. */
const AREA = { width: 400, height: 200 };

const refreshFixture = vi.hoisted(() => ({ read: vi.fn(), watch: vi.fn(), notify: vi.fn() }));

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
    series: ys.map((_, index) => ({
      key: fields.keys?.[index] ?? `trace-${index}`,
      label: `Trace ${index}`,
      points: [],
      raw_points: x.length,
      downsampled: false,
    })),
    cartesian: {
      mode: "aligned",
      series: ys.map((y) => ({ x, y: [...y] })),
      point_count: ys.reduce((count, y) => count + y.filter((value) => value !== null).length, 0),
      signature: "host",
    },
  } as unknown as PlotPayload;
}

/** Reads a uPlot cursor point option for one series, whether it is a value or a callback. */
function callPointOption<T>(option: T | ((self: uPlot, seriesIdx: number) => T) | undefined, seriesIdx: number): T | undefined {
  return typeof option === "function" ? (option as (self: uPlot, seriesIdx: number) => T)({} as uPlot, seriesIdx) : option;
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

function plotStream() {
  const messages: NativePlotStreamMessage[] = [];
  let wake: (() => void) | undefined;
  let ended = false;
  return {
    push(...updates: NativePlotStreamUpdate[]) {
      messages.push({ updates });
      wake?.();
    },
    cancel: vi.fn(async () => {
      ended = true;
      wake?.();
    }),
    async *[Symbol.asyncIterator]() {
      while (!ended) {
        if (!messages.length)
          await new Promise<void>((resolve) => {
            wake = resolve;
          });
        if (ended) return;
        const message = messages.shift();
        if (message) yield message;
      }
    },
  };
}

async function mountPlotView(ids = ["pulse", "current"]) {
  vi.useFakeTimers({ now: Date.parse("2026-11-01T12:00:00Z") });
  const animationFrame = vi
    .spyOn(globalThis.window, "requestAnimationFrame")
    .mockImplementation((callback) => globalThis.setTimeout(() => callback(0), 16) as unknown as number);
  const cancelAnimationFrame = vi
    .spyOn(globalThis.window, "cancelAnimationFrame")
    .mockImplementation((handle) => globalThis.clearTimeout(handle));
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  refreshFixture.read.mockReset();
  refreshFixture.watch.mockReset();
  refreshFixture.notify.mockReset();
  const streams: ReturnType<typeof plotStream>[] = [];
  refreshFixture.watch.mockImplementation(async () => {
    const stream = plotStream();
    streams.push(stream);
    return stream;
  });
  const plots = ids.map(
    (id): DocumentViewPlot => ({
      plot_id: id,
      title: id,
      stream: "pulse",
      plot_type: "time_series",
      style: "line",
      x_scale: "linear",
      y_scale: "linear",
      fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
      x: "timestamp",
      y: [id],
      display_points: 100,
    }),
  );
  const scope = { test_cycle_id: "cycle" };
  useSummaryWindowStore.setState({ visible: Object.fromEntries(ids.map((id) => [id, true])) });
  const session = { identity: { generation: "1", isCurrent: () => true }, own: () => () => {} } as unknown as ConnectedSession;
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  let window: ReturnType<typeof usePlotWindow>;
  let states: ReturnType<typeof useLivePlotPayloads>;
  const commits: number[][] = [];
  let nextPlots = plots;
  let nextScope = scope;
  function View() {
    window = usePlotWindow();
    states = useLivePlotPayloads({
      enabled: true,
      plots: nextPlots,
      scope: nextScope,
      windowS: window.option.value,
      refresh: window.refresh,
    });
    commits.push(Object.values(states).map((plot) => plot.revision));
    return createElement(PlotWindowControl, { live: true, elapsedUnit: null, window });
  }
  const render = async () => {
    await act(async () => {
      root.render(createElement(Connection.Provider, { value: session }, createElement(View)));
    });
  };
  onTestFinished(async () => {
    await act(async () => {
      root.unmount();
    });
    host.remove();
    useSummaryWindowStore.setState({ windows: {}, widths: {}, heights: {}, visible: {} });
    animationFrame.mockRestore();
    cancelAnimationFrame.mockRestore();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
  await render();
  return {
    plots,
    streams,
    host,
    window: () => window,
    states: () => states,
    commits,
    button: () => host.querySelector("button") as HTMLButtonElement,
    choose: async (index: number) => {
      await act(async () => {
        window.choose(index);
      });
    },
    click: async () => {
      await act(async () => {
        (host.querySelector("button") as HTMLButtonElement).click();
      });
    },
    advance: async (ms = 20) => {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(ms);
      });
    },
    runtimeUpdate: async (updated = plots.map((plot) => ({ ...plot, display_points: 200 }))) => {
      nextPlots = updated;
      nextScope = { ...scope };
      await render();
    },
  };
}

function snapshotEnvelopes(plots: DocumentViewPlot[]): NativePlotPayloadEnvelope[] {
  return plots.map((plot) => ({ ...streamMessage("snapshot", 1, hostFrame([0, 1], [[2, 3]])), plot_id: plot.plot_id }));
}

/* === Entry Point === */

vi.mock("@/lib/api/native-plots", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-plots")>()),
  fetchNativePlotPayloads: refreshFixture.read,
  openNativePlotStream: refreshFixture.watch,
}));
vi.mock("@/lib/notifications", () => ({ toast: { error: refreshFixture.notify } }));
vi.mock("@/lib/application-settings", async (original) => {
  const settings = await original<typeof import("@/lib/application-settings")>();
  return { ...settings, useReportDefaults: () => settings.BUILT_IN_REPORT_DEFAULTS };
});

test("Window Presets", () => {
  const options = plotWindowOptions([17, 42, 3_600]);
  assert.deepEqual(
    options.map((option) => option.value),
    [17, 42, 3_600, null],
  );
  assert.equal(options.at(-2)?.label, "1 h");
  const structural = plotWindowOptions([]);
  assert.deepEqual(
    structural.map((option) => option.value),
    [null],
  );
});

test("Window Selection", () => {
  const selections: { index: number; value: number | null }[] = [];
  function View({ live, selected }: { live: boolean; selected?: number }) {
    const window = usePlotWindow();
    if (selected !== undefined && window.index !== selected) window.choose(selected);
    selections.push({ index: window.index, value: window.option.value });
    return createElement(PlotWindowControl, { elapsedUnit: null, live, window });
  }
  const live = renderToStaticMarkup(createElement(View, { live: true }));
  assert.match(live, /role="slider"/);
  assert.deepEqual(selections.at(-1), { index: 2, value: 300 });
  const selected = renderToStaticMarkup(createElement(View, { live: true, selected: 6 }));
  assert.match(selected, /role="slider"/);
  assert.equal(selections.at(-1)?.index, 6);
  assert.equal(selections.at(-1)?.value, null);
  assert.equal(renderToStaticMarkup(createElement(View, { live: false, selected: 2 })), "");
});

test("A Complete Watch Revision Commits Every Chart in One Animation Frame", async () => {
  const ids = Array.from({ length: 8 }, (_, index) => `plot.${index}`);
  const view = await mountPlotView(ids);
  const initial = snapshotEnvelopes(view.plots);
  await act(async () => {
    view.streams[0].push(...initial.map((frame) => ({ ...frame, kind: "snapshot" as const })));
  });
  assert.ok(Object.values(view.states()).every((plot) => plot.revision === 0));
  await view.advance();
  assert.ok(Object.values(view.states()).every((plot) => plot.revision === 1));
  view.commits.length = 0;
  await act(async () => {
    view.streams[0].push(...initial.map((frame) => ({ ...frame, revision: 2, kind: "replace" as const })));
    view.streams[0].push(...ids.map((plot_id) => ({ kind: "pending" as const, plot_id })));
  });
  assert.ok(Object.values(view.states()).every((plot) => plot.revision === 1));
  await view.advance();
  assert.ok(Object.values(view.states()).every((plot) => plot.revision === 2));
  assert.ok(view.commits.length > 0);
  assert.ok(view.commits.every((revisions) => new Set(revisions).size === 1));
});

test("Manual All Refresh", async () => {
  const view = await mountPlotView();
  assert.deepEqual(
    refreshFixture.watch.mock.calls[0][1].map((request: { windowS: number }) => request.windowS),
    [300, 300],
  );
  let complete: (envelopes: NativePlotPayloadEnvelope[]) => void = () => {};
  refreshFixture.read.mockImplementation(
    () =>
      new Promise<NativePlotPayloadEnvelope[]>((resolve) => {
        complete = resolve;
      }),
  );
  await view.choose(6);
  assert.equal(refreshFixture.read.mock.calls.length, 1);
  assert.equal(refreshFixture.read.mock.calls[0][1].length, 2);
  assert.ok(refreshFixture.read.mock.calls[0][1].every((request: { windowS: number | null }) => request.windowS === null));
  assert.equal(view.button().disabled, true);
  await view.click();
  assert.equal(refreshFixture.read.mock.calls.length, 1);
  await act(async () => complete(snapshotEnvelopes(view.plots)));
  await view.advance();
  assert.ok(Object.values(view.states()).every((state) => state.status === "live"));
  assert.equal(view.button().disabled, false);
  const firstTime = view.window().refresh.updatedAt;
  const retained = view.states().pulse.payload;
  await act(async () => useSummaryWindowStore.getState().setSummaryWindow("pulse", { from: 0, to: 1 }));
  assert.equal(refreshFixture.read.mock.calls.length, 2);
  assert.deepEqual(refreshFixture.read.mock.calls[1][1][0].viewport, { from: 0, to: 1 });
  assert.equal(view.states().pulse.payload, retained);
  await act(async () => complete(snapshotEnvelopes(view.plots)));
  await view.advance();
  await act(async () => useSummaryWindowStore.getState().setPanelWidth("pulse", 900));
  assert.equal(refreshFixture.read.mock.calls.length, 3);
  assert.equal(refreshFixture.read.mock.calls[2][1][0].panelWidth, 900);
  await act(async () => complete(snapshotEnvelopes(view.plots)));
  await view.advance();
  const resizedTime = view.window().refresh.updatedAt;
  await view.runtimeUpdate([]);
  await view.runtimeUpdate();
  await view.advance(60_000);
  assert.equal(refreshFixture.read.mock.calls.length, 3);
  assert.equal(view.window().refresh.updatedAt, resizedTime);
  await view.click();
  assert.equal(refreshFixture.read.mock.calls.length, 4);
  assert.ok(refreshFixture.read.mock.calls[3][1].every((request: { maxPoints: number }) => request.maxPoints === 200));
  await act(async () => complete(snapshotEnvelopes(view.plots)));
  await view.advance();
  assert.ok((view.window().refresh.updatedAt ?? 0) > (firstTime ?? 0));
});

test("Timed Refresh", async () => {
  const view = await mountPlotView();
  assert.equal(view.button().disabled, true);
  await act(async () => {
    view.streams[0].push({ kind: "pending", plot_id: "pulse" });
  });
  assert.equal(view.button().disabled, true);
  assert.equal(view.window().refresh.updatedAt, null);
  const initial = snapshotEnvelopes(view.plots);
  await act(async () => {
    view.streams[0].push({ ...initial[0], kind: "snapshot" });
  });
  assert.equal(view.button().disabled, true);
  await act(async () => {
    view.streams[0].push({ ...initial[1], kind: "snapshot" });
  });
  await view.advance();
  assert.equal(view.button().disabled, false);
  const firstTime = view.window().refresh.updatedAt;
  await view.advance(1_000);
  await act(async () => {
    view.streams[0].push(
      ...initial.map((frame) => ({ ...frame, revision: 2, kind: "replace" as const, payload: hostFrame([2, 3], [[4, 5]]) })),
    );
  });
  await view.advance();
  assert.equal(view.states().pulse.payload?.cartesian?.series[0].x[0], 2);
  assert.ok(Object.values(view.states()).every((plot) => plot.revision === 2));
  assert.ok((view.window().refresh.updatedAt ?? 0) > (firstTime ?? 0));
  await view.click();
  assert.equal(view.button().disabled, true);
  assert.equal(refreshFixture.watch.mock.calls.length, 2);
  assert.equal(refreshFixture.read.mock.calls.length, 0);
  assert.equal(view.streams[0].cancel.mock.calls.length, 1);
  await act(async () => {
    for (const frame of initial) view.streams[1].push({ ...frame, kind: "snapshot" });
  });
  await view.advance();
  assert.equal(view.button().disabled, false);
  await view.choose(0);
  assert.ok(refreshFixture.watch.mock.calls[2][1].every((request: { windowS: number }) => request.windowS === 60));
  await act(async () => {
    view.streams[2].push({ ...initial[0], kind: "error", warnings: ["Storage unavailable"] });
  });
  assert.equal(view.button().disabled, false);
  assert.equal(refreshFixture.notify.mock.calls.length, 1);
  // Repeated Ticks Retain The Failure Without Adding Notifications.
  await act(async () => {
    view.streams[2].push({ ...initial[0], kind: "error", warnings: ["Storage unavailable"] });
  });
  await view.advance();
  assert.equal(refreshFixture.notify.mock.calls.length, 1);
  assert.equal(view.states().pulse.status, "error");
  // A Successful Revision Allows A Later Failure To Be Reported Again.
  await act(async () => {
    view.streams[2].push({ ...initial[0], kind: "snapshot" });
  });
  await view.advance();
  await act(async () => {
    view.streams[2].push({ ...initial[0], kind: "error", warnings: ["Storage unavailable"] });
  });
  await view.advance();
  assert.equal(refreshFixture.notify.mock.calls.length, 2);
});

test("A Large Document Uses Independent Bounded Watches", async () => {
  const view = await mountPlotView(Array.from({ length: 65 }, (_, index) => `plot-${index}`));
  assert.deepEqual(
    refreshFixture.watch.mock.calls.map((call) => call[1].length),
    [32, 32, 1],
  );
  const initial = snapshotEnvelopes(view.plots);
  await act(async () => {
    for (let index = 0; index < view.streams.length; index++) {
      view.streams[index].push(...initial.slice(index * 32, (index + 1) * 32).map((frame) => ({ ...frame, kind: "snapshot" as const })));
    }
  });
  await view.advance();
  assert.equal(Object.values(view.states()).filter((state) => state.status === "live").length, 65);
  assert.equal(view.button().disabled, false);
  const retained = view.states()["plot-64"].payload;
  await act(async () => useSummaryWindowStore.getState().setSummaryWindow("plot-0", { from: 0, to: 1 }));
  assert.equal(refreshFixture.watch.mock.calls.length, 4);
  assert.equal(view.streams[0].cancel.mock.calls.length, 1);
  assert.equal(view.streams[1].cancel.mock.calls.length, 0);
  assert.equal(view.streams[2].cancel.mock.calls.length, 0);
  assert.equal(view.states()["plot-64"].payload, retained);
  await act(async () => view.streams[2].push({ ...initial[64], kind: "replace", revision: 2, payload: hostFrame([2, 3], [[4, 5]]) }));
  await view.advance();
  assert.equal(view.states()["plot-64"].revision, 2);
});

test("Refresh Failure", async () => {
  const view = await mountPlotView();
  const snapshots = snapshotEnvelopes(view.plots);
  refreshFixture.read.mockImplementation(async (_session, requests) =>
    snapshots.filter((envelope) => requests.some((request: { plotId: string }) => request.plotId === envelope.plot_id)),
  );
  await view.choose(6);
  const firstTime = view.window().refresh.updatedAt;
  await view.runtimeUpdate();
  refreshFixture.read.mockRejectedValueOnce(new NativeApiLimitError("Plots.Snapshots.List", "plot response exceeds transport limit"));
  await view.advance(1_000);
  await view.click();
  await view.advance();
  assert.equal(view.button().disabled, false);
  assert.equal(view.window().refresh.updatedAt, firstTime);
  assert.equal(refreshFixture.notify.mock.calls.length, 1);
  assert.equal(view.states()[view.plots[0].plot_id].status, "error");
  assert.equal(view.states()[view.plots[1].plot_id].status, "error");
  await view.advance(60_000);
  assert.equal(refreshFixture.read.mock.calls.length, 2);
  await view.click();
  assert.equal(refreshFixture.read.mock.calls.length, 3);
  assert.ok((view.window().refresh.updatedAt ?? 0) > (firstTime ?? 0));
});

test("Refresh Cancellation", async () => {
  const view = await mountPlotView();
  const pending: (() => void)[] = [];
  refreshFixture.read.mockImplementation(
    (_session, requests) =>
      new Promise<NativePlotPayloadEnvelope[]>((resolve) => {
        pending.push(() =>
          resolve(
            snapshotEnvelopes(view.plots).filter((envelope) =>
              requests.some((request: { plotId: string }) => request.plotId === envelope.plot_id),
            ),
          ),
        );
      }),
  );
  await view.choose(6);
  const signal = refreshFixture.read.mock.calls[0][2].signal as AbortSignal;
  await view.choose(2);
  assert.equal(signal.aborted, true);
  await act(async () => {
    for (const complete of pending) complete();
  });
  assert.equal(view.window().refresh.updatedAt, null);
  assert.equal(view.button().disabled, true);
  refreshFixture.read.mockImplementation(async (_session, requests) =>
    snapshotEnvelopes(view.plots).filter((envelope) => requests.some((request: { plotId: string }) => request.plotId === envelope.plot_id)),
  );
  await view.choose(6);
  assert.equal(refreshFixture.read.mock.calls.length, 2);
  assert.equal(view.button().disabled, false);
});

test("Empty Snapshot", () => {
  // A Plot Without Records Opens on Its Revision-0 Snapshot Instead of Waiting
  const empty = hostFrame([], [[]]);
  const state = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 0, empty)]);
  assert.equal(state.status, "live");
  assert.equal(state.droppedBatches, 0);

  // A Replacement That Repeats the Revision Is Still Dropped
  assert.equal(applyPlotMessages(state, [streamMessage("replace", 0, empty)]).droppedBatches, 1);
});

test("Window Resampling", () => {
  /* A 50 Hz sine sampled at 1 kHz for 60 s, drawn 800 CSS px wide. */
  const rate = 1_000;
  const x = Array.from({ length: 60 * rate }, (_, index) => index / rate);
  const sine = x.map((t) => Math.sin(2 * Math.PI * 50 * t));
  const columns = 800;

  // The Whole Run Is Cut to Four Samples per Pixel Column, Ends Included
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
  // The Line Reaches Both Frame Edges and a Reset Still Fits the Whole Run
  assert.ok(zoomed.includes(Math.round(zoom.min * rate) - 1));
  assert.ok(zoomed.includes(Math.round(zoom.max * rate) + 1));
  assert.equal(zoomed[0], 0);
  assert.equal(zoomed.at(-1), x.length - 1);

  // A Window Within the Budget Draws Every Sample, a Short Run Is Never Reduced
  const second = windowIndices(x, [sine], { min: 10, max: 11 }, columns);
  assert.ok(second);
  for (let index = 10 * rate; index <= 11 * rate; index += 1) assert.ok(second.includes(index));
  assert.equal(windowIndices(x.slice(0, 3_000), [sine.slice(0, 3_000)], null, columns), null);

  // M4 Keeps First, Lowest, Highest and Last per Bucket and Skips Gaps
  assert.deepEqual(m4Indices([0, 1, 2, 3, 4, 5], [3, 9, null, 1, 4, 5], 0, 6, 1), [0, 1, 3, 5]);
  assert.deepEqual(m4Indices([0, 1, 2], [null, null, null], 0, 3, 1), []);

  // Traces on One Axis Are Resampled Together
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

  // A Group Titled Like Its Only Plot Prints the Title Once
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
    record("prompt.request.confirmation", "2026-10-20T20:04:12Z", { prompt_id: "p-1", title: "Connect the Load" }, "Connect the Load"),
    record("prompt.request.form", "2026-10-20T20:05:00Z", { prompt_id: "p-2", title: "Enter Serial" }),
    record("prompt.response.cancelled", "2026-10-20T20:05:10Z", { prompt_id: "p-2" }),
    record("prompt.request.choice", "not a time", { prompt_id: "p-3", title: "Pick" }),
    record("step.started", "2026-10-20T20:04:00Z", {}),
  ];

  // Requests Mark When a Prompt Was Asked and Submissions When It Was Answered, in Time Order
  const markers = promptMarkers(records);
  assert.deepEqual(
    markers.map((marker) => [marker.kind, marker.x, marker.title]),
    [
      ["asked", Date.parse("2026-10-20T20:04:12Z") / 1000, "Connect the Load"],
      ["answered", Date.parse("2026-10-20T20:04:30Z") / 1000, "Connect the Load"],
      ["asked", Date.parse("2026-10-20T20:05:00Z") / 1000, "Enter Serial"],
    ],
  );

  // The Hover Names the Moment and the Prompt's Title
  assert.equal(promptMarkerLabel(markers[0]), "Prompt Asked: Connect the Load");
  assert.equal(promptMarkerLabel(markers[1]), "Prompt Answered: Connect the Load");

  // An Answer Whose Request Is Out of Scope Still Marks, Under a Generic Title
  assert.deepEqual(
    promptMarkers([records[0]]).map((marker) => marker.title),
    ["Prompt"],
  );
  assert.deepEqual(promptMarkers([]), []);
});

test("Waiting for Data", () => {
  const opened = { voltage: { ...emptyLivePlotState(), status: "loading" as const } };

  // The Watch's Acknowledgement Turns a Loading Plot into a Quiet Wait, Never an Error
  const acknowledged = applyPendingNotice(opened, "voltage", 1_000);
  assert.equal(acknowledged.voltage.status, "waiting");
  assert.equal(acknowledged.voltage.error, null);
  assert.equal(plotWaitingLabel(acknowledged.voltage), "Waiting for Data \u00b7 0 s");

  // Each Keepalive Counts from the First Notice, However Long the First Read Takes
  const later = applyPendingNotice(acknowledged, "voltage", 46_000);
  assert.equal(later.voltage.status, "waiting");
  assert.equal(plotWaitingLabel(later.voltage), "Waiting for Data \u00b7 45 s");

  // A Notice for a Plot This Watch Does Not Hold Changes Nothing
  assert.equal(applyPendingNotice(later, "current", 50_000), later);

  // A Server Error Frame Still Reports, and a Later Notice Does Not Hide It
  const failed = {
    voltage: applyPlotMessages(later.voltage, [{ ...streamMessage("error", 0, hostFrame([], [])), warnings: ["plot query unavailable"] }]),
  };
  assert.equal(failed.voltage.status, "error");
  assert.equal(applyPendingNotice(failed, "voltage", 60_000), failed);

  // The First Frame Ends the Wait
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
    display_points: 100,
  };
  const scope = { test_cycle_id: "cycle" };
  const plots = [plot, { ...plot, plot_id: "current", y: ["current"] }];
  for (const definition of plots) {
    const initial = plotRequestForDocumentPlot(definition, { scope });
    assert.equal(initial.windowS, null);
    for (const option of plotWindowOptions([60, 120])) {
      const request = plotRequestForDocumentPlot(definition, { scope, windowS: option.value });
      assert.equal(request.windowS, option.value);
      assert.equal(request.maxPoints, initial.maxPoints);
      assert.deepEqual(request.scope, scope);
    }
  }
  const unbounded = { ...plot, display_points: 0 };
  assert.ok((plotRequestForDocumentPlot(unbounded, { scope }).maxPoints ?? 0) > 0);
  assert.equal(plotRequestForDocumentPlot(unbounded, { scope, windowS: 300 }).maxPoints, 0);
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
  // Drag Options Zoom Either Axis Alone or Both Together
  assert.equal(PLOT_DRAG_OPTIONS.x, true);
  assert.equal(PLOT_DRAG_OPTIONS.y, true);
  assert.ok(PLOT_DRAG_OPTIONS.uni > 0, "a uni threshold makes one-axis drags possible");
  assert.ok(PLOT_DRAG_OPTIONS.dist > 0, "a click is not a zoom");
  assert.equal(PLOT_DRAG_OPTIONS.setScale, true);

  // A Diagonal Box Holds Both Axes in Value Space
  assert.deepEqual(heldViewport({ left: 100, top: 50, width: 100, height: 50 }, AREA, posToVal), {
    x: { min: 25, max: 50 },
    y: { min: 25, max: 37.5 },
  });

  // A Horizontal Drag Spans the Full Height and Holds X Alone
  assert.deepEqual(heldViewport({ left: 40, top: 0, width: 80, height: 200 }, AREA, posToVal), {
    x: { min: 10, max: 30 },
    y: null,
  });

  // A Vertical Drag Spans the Full Width and Holds Y Alone
  assert.deepEqual(heldViewport({ left: 0, top: 20, width: 400, height: 40 }, AREA, posToVal), {
    x: null,
    y: { min: 35, max: 45 },
  });

  // An Empty or Full-Area Box Holds Nothing
  assert.equal(heldViewport({ left: 0, top: 0, width: 0, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 10, top: 10, width: 20, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 0, top: 0, width: 400, height: 200 }, AREA, posToVal), null);

  // Ranges Are Ordered Whichever Way the Axis Runs
  const flipped = (position: number, axis: "x" | "y") => (axis === "x" ? 100 - position : position);
  assert.deepEqual(heldViewport({ left: 10, top: 10, width: 10, height: 10 }, AREA, flipped), {
    x: { min: 80, max: 90 },
    y: { min: 10, max: 20 },
  });
});

test("Histogram Axes", () => {
  // A Histogram Draws Its Declared Value on X and the Host's Count on Y
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

  // A Histogram Without a Column Spec Titles X by Its Declared Field
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

  // A Histogram Never Titles X by the Row Timestamp
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
  const plot = documentPlotSpec({ plot: descriptor, payload, telemetry });
  assert.equal(plot.booleanAxis, true);
  assert.deepEqual(payload.cartesian?.series[0].y.map(formatBooleanState), ["False", "True", "--", "False"]);
  assert.equal(formatBooleanState(0.5), "--");
  for (const zoom of [false, true]) {
    const html = renderToStaticMarkup(createElement(PlotInteractionProvider, { zoom }, createElement(TimeSeriesPlot, { plot })));
    assert.match(html, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
    assert.doesNotMatch(html, /<table\b/);
  }
  columns[0].value_type = "number";
  assert.equal(documentPlotSpec({ plot: descriptor, payload, telemetry }).booleanAxis, false);
});

test("All Marker Kinds Draw and Remain Hoverable", () => {
  const markers = [
    { x: 0, label: "first", record_type: "kind-a", color: "" },
    { x: 5, label: "second", record_type: "kind-a", color: "" },
    { x: 10, label: "third", record_type: "kind-a", color: "" },
    { x: 7, label: "other", record_type: "kind-b", color: "" },
  ];
  const prompts = [
    { kind: "asked" as const, x: 5, title: "fixture" },
    { kind: "answered" as const, x: 8, title: "fixture" },
  ];
  const kinds = markerKinds(markers, prompts);
  const stems: number[] = [];
  const plot = {
    ctx: {
      save() {},
      restore() {},
      setLineDash() {},
      beginPath() {},
      moveTo() {},
      lineTo(x: number, y: number) {
        if (y === 100) stems.push(x);
      },
      closePath() {},
      stroke() {},
      fill() {},
    },
    bbox: { left: 0, top: 0, width: 10, height: 100 },
    over: { clientWidth: 10 },
    valToPos: (x: number) => x,
  } as unknown as uPlot;
  drawMarkerKinds(plot, kinds, []);
  assert.deepEqual(stems, [0, 5, 10, 7, 5, 8]);
  assert.equal(nearestMarker(plot, kinds.slice(0, 2), 5)?.label, markers[1].label);
  assert.equal(nearestMarker(plot, kinds, 5)?.label, promptMarkerLabel(prompts[0]));
  assert.equal(nearestMarker(plot, kinds, 8)?.label, promptMarkerLabel(prompts[1]));
  assert.deepEqual(markerKinds([], []), []);
  assert.deepEqual(markerKinds([{ ...markers[0], x: null }], []), []);
});

test("Event and Prompt Markers Share Tooltip State at Every Card Edge", () => {
  const kinds = markerKinds(
    [{ x: 0, label: "fixture", record_type: "fixture-kind", color: "" }],
    [
      { kind: "asked", x: 0, title: "fixture" },
      { kind: "answered", x: 0, title: "fixture" },
    ],
  );
  for (const width of [120, 320, 680]) {
    const card = { left: 137, width };
    const areaWidth = width - 60;
    for (const [left, top] of [
      [0, 0],
      [areaWidth, 0],
      [0, 200],
      [areaWidth, 200],
      [areaWidth / 2, 100],
    ]) {
      const plot = {
        cursor: { left, top },
        over: { clientWidth: areaWidth, getBoundingClientRect: () => ({ left: card.left + 40 }) },
        valToPos: () => left,
      } as unknown as uPlot;
      const states = kinds.map((kind) => {
        const tooltip = markerTooltip(plot, [kind], card);
        assert.ok(tooltip);
        assert.equal(tooltip.label, kind.moments[0].label);
        assert.ok(tooltip.left >= 0);
        assert.ok(tooltip.width > 0);
        assert.ok(tooltip.left + tooltip.width <= card.width);
        return { left: tooltip.left, width: tooltip.width };
      });
      assert.deepEqual(states[0], states[1]);
      assert.deepEqual(states[0], states[2]);
      plot.cursor.left = -1;
      assert.equal(markerTooltip(plot, kinds, card), null);
    }
  }
});

test("Marker Canvas Draws Glyphs Without Text", () => {
  const kinds = markerKinds(
    [{ x: 0, label: "fixture", record_type: "fixture-kind", color: "" }],
    [
      { kind: "asked", x: 0, title: "fixture" },
      { kind: "answered", x: 0, title: "fixture" },
    ],
  );
  const ctx = {
    save() {},
    restore() {},
    setLineDash() {},
    beginPath() {},
    moveTo() {},
    lineTo() {},
    closePath() {},
    stroke: vi.fn(),
    fill: vi.fn(),
    fillText: vi.fn(),
    strokeText: vi.fn(),
  };
  const plot = { ctx, bbox: { left: 0, top: 0, width: 400, height: 200 }, valToPos: () => 100 } as unknown as uPlot;
  drawMarkerKinds(plot, kinds, []);
  assert.ok(ctx.stroke.mock.calls.length >= kinds.length);
  assert.ok(ctx.fill.mock.calls.length > 0);
  assert.equal(ctx.fillText.mock.calls.length, 0);
  assert.equal(ctx.strokeText.mock.calls.length, 0);
});

test("Legend Rows Contain Only Traces on Both Surfaces", () => {
  const payload = hostFrame(
    [0, 5, 10],
    [
      [1, 2, 3],
      [4, 5, 6],
    ],
  );
  const markers = [{ x: 5, label: "fixture", record_type: "fixture-kind", color: "" }];
  for (const zoom of [false, true]) {
    const html = renderToStaticMarkup(
      createElement(
        PlotInteractionProvider,
        { zoom },
        createElement(CartesianPlot, {
          title: "Fixture",
          cartesian: payload.cartesian,
          series: payload.series,
          markers,
          promptMarkers: [
            { x: 7, kind: "asked", title: "fixture" },
            { x: 8, kind: "answered", title: "fixture" },
          ],
          legend: { summaries: [], pending: false },
        }),
      ),
    );
    const document = new DOMParser().parseFromString(html, "text/html");
    const rows = document.querySelectorAll("tbody tr");
    assert.deepEqual(
      [...rows].map((row) => row.getAttribute("data-row-id")),
      payload.series.map((trace) => trace.key),
    );
    const withoutStatistics = renderToStaticMarkup(
      createElement(
        PlotInteractionProvider,
        { zoom },
        createElement(CartesianPlot, {
          title: "Fixture",
          cartesian: payload.cartesian,
          series: payload.series,
          markers,
        }),
      ),
    );
    assert.doesNotMatch(withoutStatistics, /<table\b/);
  }
});

test("Application Theme Selects the Plot Palette Without a Browser Theme Provider", () => {
  const light = ["#123456", "#654321"];
  const dark = ["#abcdef", "#fedcba"];
  const family: appearance.RuntimeThemeFamily = {
    plugin_id: "organization",
    theme_id: "plot-visibility",
    label: "Plot Visibility",
    light: { variables: {}, plot_palette: light },
    dark: { variables: {}, plot_palette: dark },
  };
  const frame: ReturnType<typeof appearance.useAppearance> = {
    app_name: "Plot Test",
    logos: [],
    themes: [family],
    active_theme: family.theme_id,
    theme_family_locked: false,
    mode_policy: "user_or_system",
    mode_locked: false,
    source_plugin_id: null,
  };
  const readAppearance = vi.spyOn(appearance, "useAppearance").mockReturnValue(frame);
  onTestFinished(() => readAppearance.mockRestore());
  function Palette() {
    return createElement("output", null, JSON.stringify(usePlotPalette()));
  }
  for (const mode of ["light", "dark"] as const) {
    const html = renderToStaticMarkup(
      createElement(ApplicationThemeProvider, { value: { resolvedTheme: mode, setTheme() {} } }, createElement(Palette)),
    );
    assert.equal(html, renderToStaticMarkup(createElement("output", null, JSON.stringify(family[mode].plot_palette))));
  }

  // Frozen Reports Read the Palette Already Applied for Their Displayed Mode.
  const originalClass = document.documentElement.className;
  const originalStyle = document.documentElement.style.cssText;
  const originalTitle = document.title;
  onTestFinished(() => {
    applyThemeVariables(null);
    document.documentElement.className = originalClass;
    document.documentElement.style.cssText = originalStyle;
    document.title = originalTitle;
    vi.unstubAllGlobals();
  });
  readAppearance.mockReturnValue({ ...frame, themes: [], active_theme: null });
  for (const mode of ["light", "dark"] as const) {
    vi.stubGlobal("matchMedia", () => ({ matches: mode === "dark" }));
    applyReportAppearance(frame);
    assert.equal(document.documentElement.style.colorScheme, mode);
    const html = renderToStaticMarkup(createElement(Palette));
    assert.equal(html, renderToStaticMarkup(createElement("output", null, JSON.stringify(family[mode].plot_palette))));
  }
});

test("Point Marks", () => {
  const color = "rgb(96, 165, 250)";
  const background = "rgb(10, 10, 10)";
  for (const dense of [false, true]) {
    const solid = seriesPointMarks("points", dense, color, background);
    assert.equal(solid.show, true);
    assert.equal(solid.stroke, background);
    assert.equal(solid.fill, color);
    assert.ok(solid.width > 0 && solid.width < solid.size / 2);

    const linePoints = seriesPointMarks("line_points", dense, color, background);
    // Dense Line Points Keep Only Isolated Marks, Drawn Like Solid Points.
    assert.equal(linePoints.show, !dense);
    assert.equal(linePoints.stroke, background);
    assert.equal(linePoints.fill, color);
    assert.equal(linePoints.size, solid.size);
    assert.equal(linePoints.width, solid.width);

    for (const style of ["line", "step"] as const) {
      assert.equal(seriesPointMarks(style, dense, color, background).show, false);
    }
  }
  const sparse = seriesPointMarks("points", false, color, background);
  const dense = seriesPointMarks("points", true, color, background);
  assert.ok(dense.size < sparse.size);
});

test("Sparse Aligned Point Paths", () => {
  const x = Array.from({ length: 20 }, (_, index) => 1_700_000_000 + index * 11);
  const traces = [0, 1].map((parity) => x.map((_, index) => (index % 2 === parity ? index : null)));
  for (const fitPixels of [false, true]) {
    const data = drawnData({ source: [x, ...traces], aligned: true, range: null, everySample: true }, null, 640, fitPixels);
    assert.deepEqual(data, [x, ...traces]);
    for (const style of ["points", "line", "line_points", "step"] as const) {
      const marks = seriesPointMarks(style, false, "currentColor", "transparent");
      for (let index = 1; index <= traces.length; index += 1) {
        const plot = { data, series: [{}, { spanGaps: false }, { spanGaps: false }] } as unknown as uPlot;
        const filter = (marks as uPlot.Series.Points).filter;
        const selected = typeof filter === "function" ? filter(plot, index, marks.show) : filter;
        const count = marks.show ? traces[index - 1].filter((value) => value !== null).length : (selected?.length ?? 0);
        assert.equal(count, 10, `${style} retains every isolated aligned sample`);
      }
    }
  }
  for (const style of ["line", "line_points", "step"] as const) {
    const marks = seriesPointMarks(style, true, "currentColor", "transparent");
    const filter = marks.filter;
    assert.equal(typeof filter, "function");
    if (typeof filter !== "function") continue;
    const plot = {
      data: [
        [0, 1, 2, 3, 4, 5, 6],
        [1, 2, null, 3, null, 4, 5],
      ],
      series: [{}, { spanGaps: false, idxs: [0, 6] }],
    } as unknown as uPlot;
    assert.deepEqual(filter(plot, 1, false), [3]);
    plot.series[1].spanGaps = true;
    assert.equal(filter(plot, 1, false), null);
    (plot.data as unknown[])[1] = [null, null, null, 3, null, null, null];
    assert.deepEqual(filter(plot, 1, false), [3]);
    assert.equal(filter(plot, 1, true), null);
  }
});

test("Sparse Faceted Point Paths", () => {
  const circles: number[][] = [];
  class CanvasPath {
    moveTo() {}
    lineTo() {}
    arc(...values: number[]) {
      circles.push(values);
    }
  }
  vi.stubGlobal("Path2D", CanvasPath);
  onTestFinished(() => {
    vi.unstubAllGlobals();
  });
  const plot = {
    data: [
      null,
      [
        [0, null, 2, 3, 4, 5],
        [1, 2, null, 4, null, 6],
      ],
    ],
    ctx: { save() {}, restore() {}, beginPath() {}, rect() {}, clip() {}, fill() {}, stroke() {} },
    bbox: { left: 0, top: 0, width: 100, height: 100 },
    valToPos: (value: number) => value * 10,
  } as unknown as uPlot;
  const source = plot.data;
  for (const range of [null, { min: 1, max: 4 }]) {
    const drawn = { source, aligned: false, range: null, everySample: false };
    assert.equal(drawnData(drawn, range, 1, true), source);
    assert.equal(drawn.everySample, true);
  }
  for (const style of ["points", "line", "line_points", "step"] as const) {
    circles.length = 0;
    facetedSeriesPaths(style, seriesPointMarks(style, false, "currentColor", "transparent"), "currentColor")(plot, 1, 0, 5);
    assert.deepEqual(
      circles.map(([x, y]) => [x, y]),
      [
        [0, 10],
        [30, 40],
        [50, 60],
      ],
      `${style} draws only finite pairs`,
    );
  }
});

test("Window Reduction Retains Gaps", () => {
  const x = Array.from({ length: 100 }, (_, index) => index);
  const y = x.map((index) => (index === 49 ? 1_000 : [0, 99].includes(index) ? index : null));
  for (const range of [null, { min: 40, max: 60 }])
    for (const fitPixels of [false, true]) {
      const indices = windowIndices(x, [y], range, 1, fitPixels);
      assert.ok(indices?.includes(49));
      const data = drawnData({ source: [x, y], aligned: true, range: null, everySample: true }, range, 1, fitPixels);
      const reduced = data[1];
      const middle = data[0].indexOf(49);
      assert.equal(reduced[middle - 1], null);
      assert.equal(reduced[middle + 1], null);
    }
});

test("Cursor Point Marks", () => {
  const colors = ["rgb(96, 165, 250)", "rgb(251, 146, 60)"];
  const background = "rgb(10, 10, 10)";
  for (const style of ["points", "line_points"] as const) {
    const marks = colors.map((color) => seriesPointMarks(style, false, color, background));
    const cursor = cursorPointOptions(marks);
    assert.equal(cursor.width, marks[0].width);
    colors.forEach((color, index) => {
      const seriesIdx = index + 1;
      assert.equal(callPointOption(cursor.fill, seriesIdx), color);
      assert.equal(callPointOption(cursor.stroke, seriesIdx), background);
      assert.ok(Number(callPointOption(cursor.size, seriesIdx)) > marks[index].size);
    });
  }
});

test("XY Styles Reach the Native Request", () => {
  for (const style of ["line", "points", "line_points", "step"] as const) {
    const plot = {
      plot_id: "tracking.path",
      title: "Tracking Path",
      stream: "tracking",
      plot_type: "xy",
      x: "easting_m",
      y: ["northing_m"],
      style,
    } as DocumentViewPlot;
    const request = plotRequestForDocumentPlot(plot, { scope: null });
    assert.equal(request.plotType, "xy");
    assert.equal(request.style, style);
    const native = toNativePlotRequest(request);
    assert.equal(native.plotType, PlotKind.XY);
    assert.equal(
      native.style,
      { line: LineStyle.LINE, points: LineStyle.POINTS, line_points: LineStyle.LINE_POINTS, step: LineStyle.STEP }[style],
    );
    const spec = documentPlotSpec({ plot, payload: hostFrame([1, 2], [[3, 4]]) });
    assert.equal(spec.style, style);
    const html = renderToStaticMarkup(createElement(PlotInteractionProvider, { zoom: false }, createElement(XyPlot, { plot: spec })));
    assert.match(html, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
  }
});

test("Plot Statistics", () => {
  const payload = hostFrame([0, 1, 2], [[2, 5, 11]], { keys: ["distance"] });
  payload.summaries = [{ ...SUMMARIES[0], series_id: "distance" }];
  payload.series[0].points = [0, 1, 2].map((x) => ({ x, y: [2, 5, 11][x], instant: `2026-01-01T00:00:0${x}Z` }));
  const descriptor = {
    plot_id: "distance",
    title: "Distance",
    stream: "pulse",
    plot_type: "time_series",
    x: "timestamp",
    y: ["distance"],
  } as DocumentViewPlot;
  const plot = documentPlotSpec({ plot: descriptor, payload });
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
    for (const Component of [XyPlot, TrajectoryPlot]) {
      const xy = render(createElement(Component, { plot: { ...plot, timeAxis: false } }));
      assert.match(xy, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
      assert.doesNotMatch(xy, /<table\b/);
    }
    const xy = render(createElement(XyIntervalPlot, { plot: interval }));
    assert.match(xy, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
    assert.doesNotMatch(xy, /<table\b/);
  }
});

test("Series Legend", () => {
  // Live and Report Legends List the Trace, the Cursor and the Statistics with the Unit in Each Header
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
  // Widths Depend on the Column Alone, So Every Table on a Page Lines up Whatever Its Unit
  const widths = (unit: string | null) => seriesLegendColumns(unit).map((column) => seriesLegendColumnWidth(column.id));
  assert.deepEqual(widths("dBm"), widths(null));
  assert.deepEqual(widths("M\u03a9"), widths("V"));
  assert.deepEqual(widths(null).slice(1), ["w-52", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32", "w-32"]);

  // Rows Follow Trace Order and Read Each Trace's Own Backend Summary
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

  // A Trace Without a Summary Shows Dashes, Never Browser-Side Statistics
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

  // Clicking a Row Toggles Its Trace and a Hidden Row Keeps Its Numbers
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

  // The Cursor Column Is Blank Until the Pointer Has Been over the Plot
  assert.deepEqual(
    rows.map((row) => [row.xCursor, row.yCursor]),
    [
      ["", ""],
      ["", ""],
    ],
  );

  // The Cursor Reads Each Trace's Nearest Drawn Point and Skips Its Gaps
  const x = [0, 1, 2, 3, 4];
  const voltage = [10, 11, null, 13, 14];
  const current = [null, null, null, null, null];
  assert.deepEqual(nearestDrawnPoint(x, voltage, 0.4), { x: 0, y: 10 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 0.6), { x: 1, y: 11 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 2.2), { x: 3, y: 13 });
  assert.deepEqual(nearestDrawnPoint(x, voltage, 99), { x: 4, y: 14 });
  assert.equal(nearestDrawnPoint(x, current, 1), null);
  assert.deepEqual(nearestDrawnPoint([3, 0, 2], [30, 0, 20], 0.9), { x: 0, y: 0 });
  // Live or Report, the Cursor Cells Follow the Plot Cursor
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

  // On a Time Axis the X Cursor Reads as the Local Date and Clock, to What One Pixel Resolves
  const evening = new Date(2026, 9, 20, 22, 4, 12, 345).getTime() / 1000;
  const timeAxis = { timeAxis: true, xFormat: null, xElapsedUnit: null, origin: {}, widthPx: 600 };
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 3_600 }), "10/20/26 10:04:12 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 15 }), "10/20/26 10:04:12.35 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 1 }), "10/20/26 10:04:12.345 PM");
  assert.equal(formatCursorX(evening, { ...timeAxis, span: 120 }), "10/20/26 10:04:12.3 PM");
  assert.equal(formatCursorX(new Date(2026, 9, 21, 0, 0, 5).getTime() / 1000, { ...timeAxis, span: 3_600 }), "10/21/26 12:00:05 AM");
  // Elapsed and Numeric Axes Keep Their Own Readouts, Never a Date
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

  // The Table Dims Only While a Summary Request Is in Flight
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
  // A Histogram Draws One Bar per Backend Bin with the Backend's Counts
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

  // The Summary Grid Prints the Backend's Statistics and Count
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

  // Statistics the Backend Has Not Delivered Print Dashes, Never Browser Numbers
  const missing = statsSummaryCells({ label: "Voltage", unit: "V", statistics: null, count: 0 });
  assert.deepEqual(
    missing.map(([, value]) => value),
    ["-", "-", "-", "-", "-", "-", "0"],
  );
});

test("Plot Placeholders", () => {
  // A Plot Without Samples Waits with the Shared Label and Detail
  assert.deepEqual(plotPlaceholderText({}), {
    label: WAITING_FOR_SAMPLES,
    detail: NO_NUMERIC_SAMPLES,
  });
  assert.equal(WAITING_FOR_SAMPLES, "Waiting for Samples");

  // A Stated Stream Condition Replaces the Wait in the Placeholder
  const stated = plotPlaceholderText({
    stateLabel: "Reconnecting",
    stateDetail: "The last samples remain visible while the plot stream reopens.",
  });
  assert.deepEqual(stated, {
    label: "Reconnecting",
    detail: "The last samples remain visible while the plot stream reopens.",
  });

  // A Plot Kind May Word Its Own Detail but Keeps the Shared Label
  const worded = plotPlaceholderText({}, "No categorical samples are available.");
  assert.equal(worded.label, WAITING_FOR_SAMPLES);
  assert.equal(worded.detail, "No categorical samples are available.");

  // A Plot the Backend Could Not Produce Says So Instead of Waiting
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
      viewportMode: "fit",
    }),
  );
  assert.match(html, /role="alert"/);
  assert.match(html, /Voltage channel disconnected/);
  assert.doesNotMatch(html, /Trace Statistics/);
});

test("Plot Edits Survive Presentation Changes", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  let visibility: IntersectionObserverCallback;
  let element: Element;
  vi.stubGlobal(
    "IntersectionObserver",
    class {
      constructor(callback: IntersectionObserverCallback) {
        visibility = callback;
      }
      observe(target: Element) {
        element = target;
      }
      unobserve() {}
    },
  );
  const show = async (visible: boolean) => {
    await act(async () =>
      visibility([{ target: element, isIntersecting: visible } as IntersectionObserverEntry], {} as IntersectionObserver),
    );
  };
  let renders = 0;
  const submit = vi.fn(async () => undefined);
  function Editor(props: ComponentProps<typeof PlotSettingsButton>) {
    renders += 1;
    return createElement(PlotSettingsButton, props);
  }
  const descriptor: DocumentViewPlot = {
    plot_id: "pulse",
    title: "Pulse",
    stream: "pulse",
    plot_type: "time_series",
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    x: "timestamp",
    y: ["trace-0"],
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
  };
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    host.remove();
    vi.unstubAllGlobals();
  });
  const render = async (state = emptyLivePlotState()) => {
    await act(async () =>
      root.render(
        createElement(
          PlotSettingsEditorContext.Provider,
          { value: Editor },
          createElement(LivePlotGroupGrid, {
            documentPlots: [descriptor],
            liveStates: { pulse: state },
            title: "Signals",
            onPlotSettingsChange: submit,
          }),
        ),
      ),
    );
  };
  await render();
  assert.equal(renders, 0, "off-screen plots mount lazily");
  await show(true);
  const trigger = host.querySelector<HTMLButtonElement>("button[aria-label^='Plot Settings for ']");
  assert.ok(trigger);
  await act(async () => trigger.click());
  const input = document.querySelector<HTMLInputElement>("[role=dialog] input[type=number]");
  assert.ok(input);
  const payload = hostFrame([0, 1], [[2, 3]]);
  const ready = applyPlotMessages(emptyLivePlotState(), [streamMessage("snapshot", 1, payload)]);
  const failed = applyPlotMessages(ready, [{ ...streamMessage("error", 1, payload), warnings: ["fixture"] }]);
  input.focus();
  for (const value of ["", "2", "25", "250", "2500"]) {
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await render(ready);
    assert.equal(document.querySelector("[role=dialog] input[type=number]"), input);
    assert.equal(document.activeElement, input);
    assert.equal(input.value, value);
    assert.equal(submit.mock.calls.length, 0, "typing does not submit plot settings");
  }
  await render();
  await show(false);
  const pausedRenders = renders;
  for (const state of [ready, failed, ready]) {
    await render(state);
    const current = document.querySelector<HTMLInputElement>("[role=dialog] input[type=number]");
    assert.ok(current);
    assert.equal(current.value, "2500");
  }
  assert.equal(renders, pausedRenders, "off-screen payloads do not render the plot subtree");
  await show(true);
  assert.ok(renders > pausedRenders, "returning applies the latest retained payload");
  const cancel = [...document.querySelectorAll<HTMLButtonElement>("[role=dialog] button")].find(
    (button) => button.textContent === "Cancel",
  );
  assert.ok(cancel);
  await act(async () => cancel.click());
  await vi.waitFor(() => assert.ok(document.activeElement === host.querySelector("button[aria-label^='Plot Settings for ']")));
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
    assert.match(html, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
    assert.ok(html.includes("[dBm]"));
    assert.ok(html.includes("fixture-network / radio-2 / 6"));
    const missingFrame = renderToStaticMarkup(createElement(CartesianPlot, { title: "Signal", series }));
    assert.doesNotMatch(missingFrame, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
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
  const spec = documentPlotSpec({ plot, payload, plotPalette: [] });
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
  assert.match(html, /<button\b[^>]*data-slot="button"[^>]*aria-pressed="true"/);
  assert.ok(html.includes("fixture-network"));
});

test("Reference Values Resolve Constants Parameters and Tolerances Per Case in Live and Frozen Plots", () => {
  const parameter = (name: string) => ({ source: { type: "parameter" as const, name }, transforms: [] });
  const descriptor = {
    plot_id: "reference-values",
    title: "Temperature",
    stream: "temperature",
    plot_type: "time_series",
    y: ["temperature_c"],
    scale: 10,
    reference_lines: [
      { value: 0, label: "Zero" },
      { value: parameter("limit_c"), label: "Limit" },
    ],
    reference_bands: [
      { min: -10, max: 10 },
      { min: parameter("lower_limit_c"), max: parameter("upper_limit_c") },
      { center: 0, tolerance: parameter("band_c") },
      { center: parameter("target_c"), tolerance: 2 },
      { center: parameter("missing"), tolerance: 1 },
      { center: 0, tolerance: parameter("negative") },
      { min: parameter("upper_limit_c"), max: parameter("lower_limit_c") },
    ],
  } as DocumentViewPlot;
  const original = structuredClone(descriptor);
  for (const target_c of [5, 20]) {
    const parameters = {
      target_c,
      limit_c: target_c + 3,
      lower_limit_c: target_c - 4,
      upper_limit_c: target_c + 4,
      band_c: 1,
      negative: -1,
    };
    const live = documentPlotSpec({ plot: descriptor, payload: null, parameters });
    const frozen = documentPlotSpec({
      plot: JSON.parse(JSON.stringify(descriptor)),
      payload: null,
      parameters: JSON.parse(JSON.stringify(parameters)),
    });
    assert.deepEqual(
      live.referenceLines.map(({ value }) => value),
      [0, (target_c + 3) * 10],
    );
    assert.deepEqual(
      live.referenceBands.map(({ min, max }) => [min, max]),
      [
        [-100, 100],
        [(target_c - 4) * 10, (target_c + 4) * 10],
        [-10, 10],
        [(target_c - 2) * 10, (target_c + 2) * 10],
        [null, null],
        [null, null],
        [null, null],
      ],
    );
    assert.deepEqual(frozen, live);
    const nativeParameters = Object.fromEntries(
      Object.entries(parameters).map(([name, value]) => [name, { __dokime_native_number__: true, decimal: String(value) }]),
    );
    const native = documentPlotSpec({ plot: descriptor, payload: null, parameters: nativeParameters });
    assert.deepEqual(native.referenceLines, live.referenceLines);
    assert.deepEqual(native.referenceBands, live.referenceBands);
  }
  assert.deepEqual(descriptor, original);
});

test("Reference Expressions Apply Numeric Maps in Order", () => {
  const plot = {
    plot_id: "mapped-reference",
    title: "Mapped Reference",
    stream: "readings",
    plot_type: "xy",
    y: ["value"],
    reference_lines: [
      {
        value: {
          source: { type: "parameter", name: "target" },
          transforms: [
            { type: "scale", factor: 2 },
            { type: "offset", value: -1 },
          ],
        },
      },
    ],
    reference_bands: [
      {
        center: {
          source: { type: "parameter", name: "target" },
          transforms: [
            { type: "offset", value: -1 },
            { type: "scale", factor: 2 },
          ],
        },
        tolerance: 1,
      },
    ],
  } as DocumentViewPlot;
  const live = documentPlotSpec({ plot, payload: null, parameters: { target: 5 } });
  assert.equal(live.referenceLines[0].value, 9);
  assert.deepEqual([live.referenceBands[0].min, live.referenceBands[0].max], [7, 9]);
  assert.deepEqual(documentPlotSpec({ plot: JSON.parse(JSON.stringify(plot)), payload: null, parameters: { target: 5 } }), live);
});

test("Generic Grid and Contour State Does Not Depend on Host Scan Fields", () => {
  for (const plot_type of ["grid_chart", "contour"] as const) {
    const plot = { plot_id: "generic", title: "Generic", stream: "host.wifi.scan", plot_type, y: ["value"] } as DocumentViewPlot;
    const plain = documentPlotSpec({ plot, payload: null });
    const withHostFields = documentPlotSpec({
      plot,
      payload: null,
      telemetry: {
        "host.wifi.scan": {
          stream: "host.wifi.scan",
          spec: null,
          summary: {},
          samples: [{ scan_status: "error", scan_error: "unavailable" }],
          max_samples: 1,
          total_samples: 1,
          dropped_samples: 0,
          updated_at: "2026-01-01T00:00:00Z",
        },
      } satisfies Record<string, TelemetryStream>,
    });
    assert.equal(withHostFields.stateLabel, plain.stateLabel);
    assert.equal(withHostFields.stateDetail, plain.stateDetail);
  }
});

test("Display Scale Converts Returned Points Statistics and References Without Mutating Evidence", () => {
  const descriptor = {
    plot_id: "distance",
    title: "Distance Delta",
    stream: "distance",
    plot_type: "time_series",
    y: ["delta_m"],
    unit: "mm",
    scale: 1000,
    reference_lines: [
      { value: 0.004, color: "red" },
      { value: { source: { type: "parameter", name: "target_m" }, transforms: [] }, color: "green" },
    ],
    reference_bands: [
      { min: -0.002, max: 0.003, color: "red" },
      {
        center: { source: { type: "parameter", name: "target_m" } },
        tolerance: { source: { type: "parameter", name: "tolerance_m" } },
        color: "green",
      },
    ],
  } as DocumentViewPlot;
  const payload = hostFrame([1, 2, 3], [[-0.002, null, 0.004]], { keys: ["delta_m"] });
  if (payload.cartesian) payload.cartesian.y_domain = { min: -0.002, max: 0.004 };
  payload.summaries = [
    {
      series_id: "delta_m",
      count: 2,
      min: -0.002,
      max: 0.004,
      mean: 0.001,
      median: 0.001,
      std: 0.003,
      rms: 0.003162,
      window: { from: 1, to: 3 },
    },
  ];
  const original = structuredClone({ payload, descriptor });
  const present = (frame: PlotPayload) =>
    documentPlotSpec({
      plot: descriptor,
      payload: frame,
      parameters: { target_m: 0.005, tolerance_m: 0.001 },
    });
  const plot = present(payload);
  assert.deepEqual(plot.series[0].points, []);
  // Non-Cartesian Plots Still Scale Their Single Point Projection.
  const column = payload.cartesian?.series[0];
  assert.ok(column);
  const points = {
    ...payload,
    cartesian: null,
    series: [{ ...payload.series[0], points: column.x.map((x, row) => ({ x, y: column.y[row] })) }],
  };
  const originalPoints = structuredClone(points);
  const bar = documentPlotSpec({ plot: { ...descriptor, plot_type: "bar" }, payload: points });
  assert.deepEqual(
    bar.series[0].points.map(({ x, y }) => [x, y]),
    [
      [1, -2],
      [2, null],
      [3, 4],
    ],
  );
  assert.deepEqual(points, originalPoints);
  assert.deepEqual(plot.cartesian?.series[0], { x: [1, 2, 3], y: [-2, null, 4] });
  assert.deepEqual(plot.cartesian?.y_domain, { min: -2, max: 4 });
  assert.deepEqual(plot.summaries, [
    { series_id: "delta_m", count: 2, min: -2, max: 4, mean: 1, median: 1, std: 3, rms: 3.162, window: { from: 1, to: 3 } },
  ]);
  assert.deepEqual(
    plot.referenceBands.map(({ min, max }) => [min, max]),
    [
      [-2, 3],
      [4, 6],
    ],
  );
  assert.deepEqual(
    plot.referenceLines.map(({ value }) => value),
    [4, 5],
  );
  // Repeated frames and frozen JSON take the same conversion path once.
  assert.deepEqual(present(JSON.parse(JSON.stringify(payload))), plot);
  assert.deepEqual({ payload, descriptor }, original);
  const unscaled = documentPlotSpec({ plot: { ...descriptor, scale: undefined }, payload });
  assert.equal(unscaled.cartesian, payload.cartesian);
  assert.equal(unscaled.summaries, payload.summaries);
});

test("Generated Plot Series Labels Respect Word Positions", () => {
  const keys = ["number_of_rows", "wait_until_chamber_is_ready", "log_in", "turn_off_the_heater"];
  const descriptor = {
    plot_id: "generated-labels",
    title: "Generated Labels",
    stream: "fixture",
    plot_type: "time_series",
    x: "timestamp",
    y: keys,
  } as DocumentViewPlot;
  const payload = hostFrame(
    [0],
    keys.map(() => [1]),
    { keys },
  );
  const plot = documentPlotSpec({ plot: descriptor, payload });
  assert.deepEqual(
    plot.series.map((series) => series.label),
    ["Number of Rows", "Wait Until Chamber Is Ready", "Log In", "Turn Off the Heater"],
  );
});

test("Binned Contours and Declared Grids Keep Zero Values Separate From Gaps", () => {
  const declaration = {
    plot_id: "surface",
    title: "Sensor Values",
    stream: "readings",
    plot_type: "contour",
    y: ["value"],
  } as DocumentViewPlot;
  const payload: PlotPayload = {
    stream: "readings",
    x_key: "x",
    series: [],
    max_points: 4,
    generated_at: "2026-01-01T00:00:00Z",
    analysis: {
      kind: "contour",
      grid: { columns: 2, rows: 2, x_edges: [0, 2, 3], y_edges: [0, 1, 4], values: [0, 0, 0, 4], valid: [true, false, false, true] },
    },
  };
  const plot = documentPlotSpec({ plot: declaration, payload });
  const host = document.createElement("div");
  host.innerHTML = renderToStaticMarkup(createElement(ContourPlot, { plot }));
  assert.equal(host.querySelectorAll("rect[data-bin]").length, 2);
  assert.ok(host.querySelector('rect[data-bin="0"]')?.getAttribute("fill"));
  assert.equal(host.querySelector('rect[data-bin="1"]'), null);
  const first = host.querySelector('rect[data-bin="0"]') as SVGRectElement;
  const last = host.querySelector('rect[data-bin="3"]') as SVGRectElement;
  assert.ok(Math.abs(Number(first.getAttribute("width")) / Number(last.getAttribute("width")) - 2) < 1e-12);
  assert.ok(Math.abs(Number(last.getAttribute("height")) / Number(first.getAttribute("height")) - 3) < 1e-12);
  host.innerHTML = renderToStaticMarkup(createElement(ContourPlot, { plot: { ...plot, render: "bands" } }));
  assert.ok(host.querySelectorAll("[data-contour-mask] path").length > 0);
  assert.equal(host.querySelectorAll("clipPath rect").length, 2);
  assert.equal(host.querySelector('clipPath rect[data-valid-bin="1"]'), null);
  const mask = host.querySelector('clipPath rect[data-valid-bin="3"]') as SVGRectElement;
  const cell = host.querySelector('rect[data-bin="3"]') as SVGRectElement;
  for (const attribute of ["x", "y", "width", "height"]) assert.equal(mask.getAttribute(attribute), cell.getAttribute(attribute));
  const points = [...host.querySelectorAll("[data-contour-mask] path")].flatMap((path) =>
    [...(path.getAttribute("d") ?? "").matchAll(/[ML](-?[\d.]+),(-?[\d.]+)/g)].map((match) => [Number(match[1]), Number(match[2])]),
  );
  const centerX = Number(cell.getAttribute("x")) + Number(cell.getAttribute("width")) / 2;
  const centerY = Number(cell.getAttribute("y")) + Number(cell.getAttribute("height")) / 2;
  assert.ok(points.some(([x]) => Math.abs(x - centerX) < 1e-3));
  assert.ok(points.some(([, y]) => Math.abs(y - centerY) < 1e-3));

  const grid: PlotPayload = {
    ...payload,
    analysis: {
      kind: "grid_chart",
      data: {
        rows: [{ key: "sensor", label: "Sensor", order: 0, group: "North" }],
        columns: [
          { key: "left", label: "Left", order: 0 },
          { key: "right", label: "Right", order: 1 },
        ],
        frames: [
          { instant: "2026-01-01T00:00:00Z", values: [0, 0], valid: [true, false], highlights: [{ row: 0, column: 0, label: "Selected" }] },
        ],
        summary_label: "Readings",
      },
    },
  };
  const cellPlot = documentPlotSpec({ plot: { ...declaration, plot_type: "grid_chart" }, payload: grid });
  host.innerHTML = renderToStaticMarkup(createElement(GridChartPlot, { plot: cellPlot }));
  const populated = host.querySelector('[data-cell="sensor/left"]');
  const absent = host.querySelector('[data-cell="sensor/right"]');
  assert.equal(populated?.textContent, "0");
  assert.equal(populated?.getAttribute("data-valid"), "true");
  assert.equal(populated?.getAttribute("data-highlight"), "true");
  assert.equal(absent?.textContent, "");
  assert.equal(absent?.getAttribute("data-valid"), "false");
  assert.equal(absent?.getAttribute("style"), null);
  const field = (name: string) => ({ source: { type: "telemetry", stream: "readings", field: name }, transforms: [] });
  const request = toNativePlotRequest({
    stream: "readings",
    yKeys: ["value"],
    plotType: "grid_chart",
    reduction: "max",
    grid: {
      row: field("row"),
      column: field("column"),
      frame: field("timestamp"),
      rows: grid.analysis?.kind === "grid_chart" ? grid.analysis.data.rows : [],
      columns: grid.analysis?.kind === "grid_chart" ? grid.analysis.data.columns : [],
      highlights: [],
      summary_label: "Readings",
    },
  });
  assert.equal(request.reduction, Reduction.MAX);
  assert.equal(request.grid?.frame?.kind.case, "stream");
  assert.equal(request.grid?.rows[0].key, "sensor");
});

test("Numeric Expression Maps Preserve Their Wire Order", () => {
  const request = toNativePlotRequest({
    stream: "scan",
    yKeys: [],
    plotType: "contour",
    zExpression: {
      source: { type: "telemetry", stream: "scan", field: "temperature" },
      transforms: [
        { type: "scale", factor: 2 },
        { type: "offset", value: -1 },
      ],
    },
  });
  const outer = request.zExpression?.kind;
  assert.ok(outer?.case === "transform");
  assert.equal(outer.value.kind.case, "offset");
  assert.equal(outer.value.kind.value, -1);
  const inner = outer.value.input?.kind;
  assert.ok(inner?.case === "transform");
  assert.equal(inner.value.kind.case, "scale");
  assert.equal(inner.value.kind.value, 2);
  assert.equal(inner.value.input?.kind.case, "stream");
});

test("Bar Arrangements Pareto and Pie Draw Their Distinct Options", () => {
  const declaration = { plot_id: "totals", title: "Totals", stream: "values", plot_type: "bar", y: ["value"] } as DocumentViewPlot;
  const payload: PlotPayload = {
    stream: "values",
    x_key: "category",
    max_points: 10,
    generated_at: "2026-01-01T00:00:00Z",
    series: [
      {
        key: "total",
        label: "Total",
        raw_points: 3,
        downsampled: false,
        points: [
          { x: "a", y: 2 },
          { x: "b", y: 3 },
          { x: "c", y: 3 },
        ],
      },
    ],
  };
  const plot = documentPlotSpec({ plot: declaration, payload });
  const host = document.createElement("div");
  host.innerHTML = renderToStaticMarkup(createElement(BarPlot, { plot: { ...plot, pareto: true } }));
  assert.deepEqual(
    [...host.querySelectorAll("rect[data-category]")].map((node) => node.getAttribute("data-category")),
    ["b", "c", "a"],
  );
  assert.deepEqual(
    [...host.querySelectorAll("circle[data-percent]")].map((node) => Number(node.getAttribute("data-percent"))),
    [37.5, 75, 100],
  );
  const zero = {
    ...plot,
    series: plot.series.map((series) => ({ ...series, points: series.points.map((point) => ({ ...point, y: 0 })) })),
  };
  host.innerHTML = renderToStaticMarkup(createElement(BarPlot, { plot: { ...zero, pareto: true } }));
  assert.equal(host.querySelectorAll("rect[data-category]").length, 3);
  assert.equal(host.querySelector("[data-cumulative]"), null);
  host.innerHTML = renderToStaticMarkup(createElement(PiePlot, { plot: { ...zero, plotType: "pie" } }));
  assert.equal(host.querySelectorAll("path").length, 0);
  assert.ok(host.querySelector('[data-testid="plot-placeholder"]'));
  host.innerHTML = renderToStaticMarkup(createElement(PiePlot, { plot: { ...plot, plotType: "pie", hole: 0.5 } }));
  assert.equal(host.querySelectorAll("path[data-category]").length, 3);
  const grouped = {
    ...plot,
    series: [
      plot.series[0],
      {
        ...plot.series[0],
        key: "second",
        label: "Second",
        points: [
          { x: "a", y: 4 },
          { x: "b", y: 1 },
        ],
      },
    ],
  };
  host.innerHTML = renderToStaticMarkup(createElement(BarPlot, { plot: { ...grouped, arrangement: "grouped" } }));
  let bars = [...host.querySelectorAll('rect[data-category="a"]')];
  assert.equal(bars.length, 2);
  assert.notEqual(bars[0].getAttribute("y"), bars[1].getAttribute("y"));
  host.innerHTML = renderToStaticMarkup(createElement(BarPlot, { plot: { ...grouped, arrangement: "stacked" } }));
  bars = [...host.querySelectorAll('rect[data-category="a"]')];
  assert.equal(bars[0].getAttribute("y"), bars[1].getAttribute("y"));
  assert.ok(Number(bars[1].getAttribute("x")) >= Number(bars[0].getAttribute("x")) + Number(bars[0].getAttribute("width")) - 1e-9);
  const request = toNativePlotRequest({ stream: "values", yKeys: ["value"], plotType: "xy", xyInterval: "sigma" });
  assert.equal(request.plotType, PlotKind.XY);
  assert.equal(request.xyInterval, XyInterval.SIGMA);
  assert.equal(toNativePlotRequest({ stream: "values", yKeys: ["value"], plotType: "pie", hole: 0.5 }).hole, 0.5);
});

test("Native Columns Reach Trajectory and Categorical Renderers", () => {
  const instants = ["2026-01-01T00:00:00.000000001Z", "2026-01-01T00:00:04Z"];
  for (const kind of ["trajectory", "bar", "pie"] as const) {
    for (const categories of [
      ["First", "Second"],
      [0.25, 0.75],
      [1152921504606847975n, 1152921504606847976n],
    ]) {
      const wire = create(SnapshotsWatchResponseSchema, {
        updates: [
          {
            kind: PlotStreamKind.SNAPSHOT,
            envelope: {
              plotId: kind,
              epoch: "epoch",
              selection: kind,
              diagnostics: {},
              payload: {
                stream: "values",
                xKey: "coordinate",
                maxPoints: 2n,
                series: [{ key: "value", label: "Value", rawPoints: 2n }],
                frame: {
                  mode: PlotCartesianMode.FACETED,
                  pointCount: 2n,
                  series: [
                    {
                      x: (kind === "trajectory" ? [1, 4] : categories).map(jsonToNativeValue),
                      y: [{ value: 2 }, { value: 3 }],
                      totals: kind === "trajectory" ? [] : [1152921504606847975n, 1152921504606847976n].map(jsonToNativeValue),
                      instants: kind === "trajectory" ? instants : [],
                    },
                  ],
                },
              },
            },
          },
        ],
      });
      const update = mapNativePlotWatchPage(wire, new Map([[kind, kind]])).updates[0];
      assert.ok(update.kind !== "pending");
      assert.equal(update.payload.cartesian, null);
      const plot = documentPlotSpec({
        plot: {
          plot_id: kind,
          title: "Values",
          stream: "values",
          plot_type: kind,
          y: ["value"],
          arrangement: "grouped",
          style: "line",
          x_scale: "linear",
          y_scale: "linear",
          fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
        },
        payload: update.payload,
      });
      const host = document.createElement("div");
      host.innerHTML = renderToStaticMarkup(
        createElement(
          PlotInteractionProvider,
          { zoom: false },
          kind === "trajectory"
            ? createElement(TrajectoryPlot, { plot })
            : kind === "bar"
              ? createElement(BarPlot, { plot })
              : createElement(PiePlot, { plot }),
        ),
      );
      assert.equal(host.querySelector('[data-testid="plot-placeholder"]'), null);
      if (kind === "trajectory") {
        assert.equal(host.querySelector("[data-trajectory-instant]")?.getAttribute("data-trajectory-instant"), instants[1]);
        assert.deepEqual(
          plot.series[0].points.map((point) => point.instant),
          instants,
        );
      } else {
        assert.deepEqual(
          [...host.querySelectorAll("[data-category]")].map((element) => element.getAttribute("data-category")),
          categories.map(String),
        );
        assert.ok(host.textContent?.includes("1152921504606847975"));
        assert.ok(host.textContent?.includes("1152921504606847976"));
      }
    }
  }
});

test("Playback Holds Loaded Frames and Keeps Each Plot Independent", async () => {
  vi.useFakeTimers();
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const fetch = vi.fn(() => {
    throw new Error("Playback attempted a request");
  });
  vi.stubGlobal("fetch", fetch);
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  let incoming = ["2026-01-01T00:00:00.000000001Z", "2026-01-01T00:00:00.000000002Z", "2026-01-01T00:00:04Z"];
  let trajectory!: ReturnType<typeof usePlotPlayback<string[]>>;
  let grid!: ReturnType<typeof usePlotPlayback<string[]>>;
  const windows = useSummaryWindowStore.getState().windows;
  function View() {
    const available = orderedInstants(incoming);
    trajectory = usePlotPlayback(incoming, available);
    grid = usePlotPlayback(incoming, available);
    return createElement(
      "div",
      null,
      createElement(PlotPlayback, { title: "Trajectory", playback: trajectory }),
      createElement(PlotPlayback, { title: "Grid", playback: grid }),
    );
  }
  const render = () =>
    act(async () => {
      root.render(createElement(View));
    });
  try {
    await render();
    const first = incoming[0];
    const last = incoming.at(-1);
    assert.notEqual(instantTime(incoming[0]), instantTime(incoming[1]));
    assert.equal(trajectory.instant, last);
    const slider = host.querySelector('[role="slider"]');
    assert.ok(slider);
    await act(async () => {
      slider.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", code: "ArrowLeft", bubbles: true }));
    });
    assert.equal(trajectory.instant, incoming[1]);
    assert.equal(grid.instant, last);
    const held = trajectory.data;
    incoming = [...incoming, "2026-01-01T00:00:08Z"];
    await render();
    assert.equal(trajectory.instant, incoming[1]);
    assert.equal(trajectory.data, held);
    assert.equal(grid.instant, incoming.at(-1));
    await act(async () => {
      trajectory.replay();
    });
    assert.equal(trajectory.instant, first);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });
    assert.equal(trajectory.instant, held[1]);
    await act(async () => {
      trajectory.pause();
      await vi.advanceTimersByTimeAsync(600);
    });
    assert.equal(trajectory.instant, held[1]);
    await act(async () => {
      trajectory.play();
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(600);
    });
    assert.equal(trajectory.instant, last);
    assert.equal(trajectory.playing, false);
    assert.equal(trajectory.following, false);
    await act(async () => {
      trajectory.seek(held.length - 1, true);
    });
    assert.equal(trajectory.instant, incoming.at(-1));
    assert.equal(trajectory.following, true);
    await act(async () => {
      trajectory.play();
    });
    assert.equal(trajectory.following, false);
    incoming = [...incoming, "2026-01-01T00:00:12Z"];
    await render();
    assert.equal(trajectory.instant, "2026-01-01T00:00:08Z");
    await act(async () => {
      trajectory.latest();
    });
    assert.equal(trajectory.instant, incoming.at(-1));
    incoming = [first];
    await render();
    assert.equal((host.querySelector("button") as HTMLButtonElement).disabled, true);
    incoming = [];
    await render();
    assert.equal(trajectory.instant, null);
    assert.equal(host.querySelector('[data-slot="slider"]')?.getAttribute("aria-disabled"), "true");
    assert.equal(useSummaryWindowStore.getState().windows, windows);
    assert.equal(fetch.mock.calls.length, 0);
  } finally {
    await act(async () => {
      root.unmount();
    });
    host.remove();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});

test("Two Clients Load One Hundred Producing Plots and Fence a Reconnect", async () => {
  vi.useFakeTimers();
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const raf = vi
    .spyOn(window, "requestAnimationFrame")
    .mockImplementation((callback) => setTimeout(() => callback(0), 16) as unknown as number);
  const cancel = vi.spyOn(window, "cancelAnimationFrame").mockImplementation((handle) => clearTimeout(handle));
  const plots = Array.from(
    { length: 100 },
    (_, index) =>
      ({ plot_id: `live-${index}`, title: `Plot ${index}`, stream: "signal", plot_type: "time_series", y: ["value"] }) as DocumentViewPlot,
  );
  useSummaryWindowStore.setState({ visible: Object.fromEntries(plots.map((plot) => [plot.plot_id, true])) });
  refreshFixture.watch.mockReset();
  const streams: { generation: string; ids: string[]; stream: ReturnType<typeof plotStream> }[] = [];
  let release!: () => void;
  const loading = new Promise<void>((resolve) => {
    release = resolve;
  });
  refreshFixture.watch.mockImplementation(async (identity, requests) => {
    const stream = plotStream();
    streams.push({ generation: identity.generation, ids: requests.map((request: { plotId: string }) => request.plotId), stream });
    if (identity.generation === "1") await loading;
    return stream;
  });
  let current = true;
  const session = (generation: string, isCurrent = () => true) =>
    ({ identity: { generation, isCurrent }, own: () => () => {} }) as unknown as ConnectedSession;
  const first = session("1", () => current);
  const second = session("2");
  const replacement = session("3");
  const states = new Map<string, ReturnType<typeof useLivePlotPayloads>>();
  function Probe({ name }: { name: string }) {
    states.set(
      name,
      useLivePlotPayloads({ enabled: true, plots, scope: { test_cycle_id: "cycle", test_execution_id: "execution" }, windowS: 300 }),
    );
    return null;
  }
  const roots = [createRoot(document.createElement("div")), createRoot(document.createElement("div"))];
  const render = async (index: number, owner: ConnectedSession) =>
    act(async () => {
      roots[index].render(createElement(Connection.Provider, { value: owner }, createElement(Probe, { name: String(index) })));
    });
  try {
    await render(0, first);
    await render(1, second);
    assert.equal(streams.filter((group) => group.generation === "1").length, 4);
    assert.equal(streams.filter((group) => group.generation === "2").length, 4);
    assert.ok(streams.every((group) => group.ids.length <= 32));
    current = false;
    await render(0, replacement);
    await act(async () => {
      release();
    });
    await act(async () => {
      for (const group of streams) {
        // A Producer Can Deliver Many Revisions Before Either Client Paints.
        for (let revision = 1; revision <= 128; revision++) {
          group.stream.push(
            ...group.ids.map((plot_id) => ({
              ...streamMessage("snapshot", revision, hostFrame([revision], [[group.generation === "1" ? -1 : revision]])),
              plot_id,
            })),
          );
        }
      }
      await vi.advanceTimersByTimeAsync(20);
    });
    for (const name of ["0", "1"]) {
      const values = Object.values(states.get(name) ?? {});
      assert.equal(values.length, 100);
      assert.ok(values.every((state) => state.revision === 128));
      assert.ok(values.every((state) => state.payload?.cartesian?.series[0].y[0] === 128));
      assert.ok(values.every((state) => state.droppedBatches === 0));
    }
    assert.ok(streams.filter((group) => group.generation === "1").every((group) => group.stream.cancel.mock.calls.length === 1));
    const retained = states.get("0")?.["live-0"];
    await act(async () => {
      const group = streams.find((group) => group.generation === "3" && group.ids.includes("live-96"));
      assert.ok(group);
      group.stream.push(...group.ids.map((plot_id) => ({ ...streamMessage("replace", 129, hostFrame([129], [[129]])), plot_id })));
      await vi.advanceTimersByTimeAsync(20);
    });
    assert.equal(states.get("0")?.["live-0"], retained);
    assert.equal(states.get("1")?.["live-96"].revision, 128);
    assert.equal(states.get("0")?.["live-96"].revision, 129);
  } finally {
    await act(async () => {
      for (const root of roots) root.unmount();
    });
    useSummaryWindowStore.setState({ visible: {}, windows: {}, widths: {}, heights: {} });
    raf.mockRestore();
    cancel.mockRestore();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});
