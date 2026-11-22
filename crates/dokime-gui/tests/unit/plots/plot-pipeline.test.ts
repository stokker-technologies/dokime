/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Viewport Requests And Bucket Replacements
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test, vi } from "vitest";
import { create } from "@bufbuild/protobuf";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// Local Imports
import { DataExpressionSchema, PlotKind, PlotCartesianMode, Reduction, SnapshotsWatchResponseSchema } from "@generated/host/model";
import { documentPlot } from "@dokime/document/lib/document-presentation";
import { Downsample, PlotStreamKind } from "@generated/host/model";
import { mapNativePlotWatchPage, openNativePlotStream } from "@/lib/api/native-plots";
import { plotsSnapshotsWatch } from "@/lib/host/client";
import { LivePlotGroupGrid } from "@dokime/plots/components/document/PlotGroupGrid";
import type { DocumentViewPlot, PlotPayloadStreamMessage } from "@dokime/plots/lib/types";
import { applyPlotMessages, emptyLivePlotState } from "@dokime/plots/live-frames";

import { plotRequestForDocumentPlot } from "@/plots/live-data";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function message(
  kind: PlotPayloadStreamMessage["kind"],
  revision: number,
  time: number[],
  values: Array<number | null>,
): PlotPayloadStreamMessage {
  return {
    kind,
    revision,
    epoch: "epoch",
    selection: "signal",
    plot_id: "signal",
    warnings: [],
    payload: {
      stream: "signal",
      x_key: "timestamp",
      generated_at: "",
      max_points: 100,
      bucket_width_s: 10,
      bucket_window: { from: 0, to: 30 },
      replaced_buckets: [],
      series: [{ key: "value", label: "Value", unit: null, raw_points: 10, downsampled: true, points: [] }],
      cartesian: { mode: "aligned", series: [{ x: time, y: values }], point_count: values.length, signature: String(revision) },
    },
  };
}

/* === Entry Point === */

vi.mock("@/lib/host/client", () => ({ plotsSnapshotsWatch: vi.fn() }));

test("Running Execution Heat Strip Draws Native Watch Cells and Replaces Them When Rows Arrive", async () => {
  const descriptor = { plot_id: "heat", title: "Heat Strip", stream: "signal", plot_type: "heat_strip", y: ["value"] } as DocumentViewPlot;
  const pages = [3, 7].map((value, index) =>
    create(SnapshotsWatchResponseSchema, {
      updates: [
        {
          kind: index ? PlotStreamKind.REPLACE : PlotStreamKind.SNAPSHOT,
          envelope: {
            plotId: "heat",
            epoch: "running",
            selection: "execution",
            revision: BigInt(index + 1),
            diagnostics: {},
            payload: {
              stream: "signal",
              series: [{ key: "band", label: "Band", rawPoints: BigInt(index + 1) }],
              frame: {
                mode: PlotCartesianMode.FACETED,
                pointCount: 1n,
                series: [
                  {
                    x: [{ kind: { case: "unsignedInteger", value: 6n } }],
                    y: [{ value }],
                    instants: [`1970-01-01T00:00:0${index + 1}Z`],
                  },
                ],
              },
            },
          },
        },
      ],
    }),
  );
  vi.mocked(plotsSnapshotsWatch).mockResolvedValue({
    cancel: async () => {},
    async *[Symbol.asyncIterator]() {
      yield* pages;
    },
  });
  const session = { generation: "1", isCurrent: () => true };
  const request = plotRequestForDocumentPlot(descriptor, { scope: null });
  const stream = await openNativePlotStream(session, [request]);
  let state = emptyLivePlotState();
  let frames = 0;
  for await (const page of stream) {
    for (const update of page.updates) {
      assert.notEqual(update.kind, "pending");
      if (update.kind === "pending") continue;
      state = applyPlotMessages(state, [update]);
    }
    assert.equal(state.status, "live");
    const markup = renderToStaticMarkup(
      createElement(LivePlotGroupGrid, {
        documentPlots: [descriptor],
        liveStates: { heat: state },
        title: "Values",
        viewportMode: "live",
      }),
    );
    assert.ok(markup.includes('data-testid="heat-strip-plot"'));
    assert.ok(markup.includes('data-cell="6"'));
    assert.ok(markup.includes(`6 · Band · ${frames ? 7 : 3}`));
    frames += 1;
  }
  assert.equal(frames, 2);
  assert.equal(state.payload?.cartesian, null);
});

test("Late Buckets Replace Retained Rows and Expire Without Duplicates", () => {
  const opening = message("snapshot", 1, [0, 1, 10, 11, 20], [1, 2, 3, null, 4]);
  const old = applyPlotMessages(emptyLivePlotState(), [opening]);
  const late = message("delta", 2, [0, 1, 5, 20], [1, 2, -9, 7]);
  late.payload.replaced_buckets = [0, 2];
  const updated = applyPlotMessages(old, [late]);
  assert.deepEqual(updated.payload?.cartesian?.series[0].x, [0, 1, 5, 10, 11, 20]);
  assert.deepEqual(updated.payload?.cartesian?.series[0].y, [1, 2, -9, 3, null, 7]);
  assert.deepEqual(old.payload?.cartesian?.series[0].x, [0, 1, 10, 11, 20]);
  const expiry = message("delta", 3, [], []);
  expiry.payload.replaced_buckets = [0];
  expiry.payload.bucket_window = { from: 10, to: 30 };
  const expired = applyPlotMessages(updated, [expiry]);
  assert.deepEqual(expired.payload?.cartesian?.series[0].x, [10, 11, 20]);
  const stale = { ...late, revision: 4, selection: "other" };
  assert.equal(applyPlotMessages(expired, [stale]).payload, expired.payload);
  const reopened = { ...message("snapshot", 0, [100], [8]), epoch: "reconnected" };
  assert.deepEqual(applyPlotMessages(expired, [reopened]).payload?.cartesian?.series[0].x, [100]);
});

test("Exact Statistics Can Lag the Drawn Frame Without Replacing It", () => {
  const opening = message("snapshot", 4, [0], [1]);
  const delta = message("delta", 6, [0, 1], [1, 2]);
  delta.payload.replaced_buckets = [0];
  const drawn = applyPlotMessages(emptyLivePlotState(), [opening, delta]);
  const statistics = message("statistics", 4, [], []);
  statistics.payload.summaries = [{ series_id: "value", count: 1, min: 1, max: 1, mean: 1, median: 1, std: 0, rms: 1 }];
  const updated = applyPlotMessages(drawn, [statistics]);
  assert.equal(updated.revision, 6);
  assert.equal(updated.statisticsRevision, 4);
  assert.equal(updated.payload?.cartesian, drawn.payload?.cartesian);
  assert.equal(updated.payload?.summaries?.[0].count, 1);
  assert.equal(updated.summaryInFlight, false);
  assert.equal(applyPlotMessages(updated, [{ ...statistics, revision: 3 }]).payload, updated.payload);
});

test("Viewport and Panel Width Reach the Native Request With Defaults Enabled", () => {
  const plot = { plot_id: "signal", stream: "signal", plot_type: "time_series", y: ["value"] } as DocumentViewPlot;
  const native = plotRequestForDocumentPlot(plot, { scope: null, windowS: 300, summaryWindow: { from: 10, to: 20 }, panelWidth: 960 });
  assert.deepEqual({ from: native.viewport?.from, to: native.viewport?.to }, { from: 10, to: 20 });
  assert.equal(native.panelWidth, 960);
  assert.ok((native.maxPoints ?? 0n) > 0n);
  assert.throws(() => plotRequestForDocumentPlot(plot, { scope: null, panelWidth: 0 }));
});

test("Heat Strip Requests Preserve Grouping and Every Reducer Across Windows", () => {
  const expression = (field: string) =>
    create(DataExpressionSchema, {
      kind: { case: "stream", value: { identifier: "signal", field } },
    });
  const plot: DocumentViewPlot = {
    plot_id: "heat",
    title: "Heat Strip",
    stream: "signal",
    plot_type: "heat_strip",
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    y: ["value"],
    y_expressions: [expression("value")],
    cell_expression: expression("cell"),
    facet_expression: expression("facet"),
  };
  for (const [reduction, expected] of [
    ["last", Reduction.LAST],
    ["max", Reduction.MAX],
    ["mean", Reduction.MEAN],
    ["count", Reduction.COUNT],
  ] as const) {
    const request = plotRequestForDocumentPlot({ ...plot, reduction }, { scope: null, windowS: 300, summaryWindow: { from: 10, to: 20 } });
    assert.equal(request.plotType, PlotKind.HEAT_STRIP);
    assert.equal(request.reduction, expected);
    assert.deepEqual(request.cellExpression, plot.cell_expression);
    assert.deepEqual(request.facetExpression, plot.facet_expression);
    assert.deepEqual(request.yExpressions, plot.y_expressions);
    assert.equal(request.windowS, 300);
    assert.deepEqual({ from: request.viewport?.from, to: request.viewport?.to }, { from: 10, to: 20 });
  }
  const request = plotRequestForDocumentPlot({ ...plot, facet_expression: null }, { scope: null });
  assert.equal(request.facetExpression, undefined);
  assert.equal(request.reduction, undefined);
});

test("Record-Only Document Intervals Reach the Native Histogram Request", () => {
  const interval = create(DataExpressionSchema, {
    kind: {
      case: "transform",
      value: {
        input: { kind: { case: "record", value: { identifier: "connected", field: "" } } },
        kind: { case: "timeBetween", value: "disconnected" },
      },
    },
  });
  const plot: DocumentViewPlot = {
    plot_id: "intervals",
    title: "Intervals",
    stream: "",
    plot_type: "histogram",
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    y: [],
    x_expression: interval,
    histogram: { bins: 2 },
  };
  const native = plotRequestForDocumentPlot(plot, { scope: null });
  assert.equal(native.stream, "");
  assert.equal(native.plotType, PlotKind.HISTOGRAM);
  assert.deepEqual(native.xExpression, interval);
  assert.deepEqual(native.yExpressions, []);
  assert.equal(native.histogram?.bins, 2n);
});

test("The Native Watch Decodes Bucket Deltas and Independent Statistics", () => {
  const wire = create(SnapshotsWatchResponseSchema, {
    updates: [PlotStreamKind.DELTA, PlotStreamKind.STATISTICS].map((kind) => ({
      kind,
      envelope: {
        plotId: "signal",
        epoch: "epoch",
        selection: "signal",
        diagnostics: {},
        payload: {
          stream: "signal",
          xKey: "timestamp",
          bucketWidthS: 10,
          bucketWindow: { from: 10, to: 20 },
          replacedBuckets: [1],
          summaries: [
            {
              seriesId: "value",
              count: 2n,
              min: { kind: { case: "number" as const, value: 1.25 } },
              max: { kind: { case: "number" as const, value: 4.5 } },
              mean: { kind: { case: "number" as const, value: 2.875 } },
            },
          ],
        },
      },
    })),
  });
  const decoded = mapNativePlotWatchPage(wire).updates;
  assert.deepEqual(
    decoded.map((message) => message.kind),
    ["delta", "statistics"],
  );
  assert.ok(decoded[0].kind !== "pending");
  assert.equal(decoded[0].payload.bucket_width_s, 10);
  assert.equal(decoded[0].payload.summaries?.[0].min, 1.25);
  assert.equal(decoded[0].payload.summaries?.[0].max, 4.5);
  assert.equal(decoded[0].payload.summaries?.[0].mean, 2.875);
  assert.deepEqual(decoded[0].payload.replaced_buckets, [1]);
});

test("Document XY Requests Preserve the Declared Display Budget and Technique", () => {
  const plot = { plot_id: "voltage", stream: "measurement", plot_type: "xy", x: "elapsed_s", y: ["voltage"] } as DocumentViewPlot;
  const defaults = plotRequestForDocumentPlot(plot, { scope: null });
  assert.equal(defaults.maxPoints, 2000n);
  assert.equal(defaults.technique, Downsample.AUTO);
  for (const [downsample, expected] of [
    ["auto", Downsample.AUTO],
    ["raw", Downsample.RAW],
    ["min_max", Downsample.MIN_MAX],
    ["lttb", Downsample.LTTB],
  ] as const) {
    const native = plotRequestForDocumentPlot({ ...plot, display_points: 11801, downsample }, { scope: null });
    assert.equal(native.maxPoints, 11801n);
    assert.equal(native.technique, expected);
    assert.equal(native.xKey, "elapsed_s");
  }
});

test("Projected Record-Only Plots Request No Stream", () => {
  const interval = create(DataExpressionSchema, {
    kind: {
      case: "transform",
      value: {
        input: { kind: { case: "record", value: { identifier: "connected", field: "" } } },
        kind: { case: "timeBetween", value: "disconnected" },
      },
    },
  });
  const projected = documentPlot({
    plot_id: "time.reconnect",
    title: "Time",
    plot_type: "histogram",
    x_expression: interval,
    available_streams: ["telemetry"],
  });
  assert.equal(projected.stream, "");
  assert.equal(plotRequestForDocumentPlot(projected, { scope: null, panelWidth: 640 }).stream, "");
  const sampled = create(DataExpressionSchema, { kind: { case: "stream", value: { identifier: "telemetry", field: "value" } } });
  assert.equal(documentPlot({ plot_id: "series", plot_type: "time_series", y_expressions: [sampled] }).stream, "telemetry");
});
