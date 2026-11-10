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
import { test } from "vitest";
import { create } from "@bufbuild/protobuf";

// Local Imports
import { Row3Schema as WatchFrameSchema } from "@generated/dokime/v1/PlotsMessages/Watch/messages_pb";
import { mapNativePlotStreamFrame, toNativePlotRequest } from "@/lib/api/native-plots";
import type { DocumentViewPlot, PlotPayloadStreamMessage } from "@/lib/types";
import { applyPlotMessages, emptyLivePlotState } from "@/plots/live-frames";

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
  const input = plotRequestForDocumentPlot(plot, { scope: null, windowS: 300, summaryWindow: { from: 10, to: 20 }, panelWidth: 960 });
  const native = toNativePlotRequest(input);
  assert.deepEqual({ from: native.viewport?.from, to: native.viewport?.to }, { from: 10, to: 20 });
  assert.equal(native.panelWidth, 960);
  assert.ok((native.maxPoints ?? 0n) > 0n);
  assert.throws(() => toNativePlotRequest({ ...input, panelWidth: 0 }));
});

test("The Native Watch Decodes Bucket Deltas and Independent Statistics", () => {
  const wire = create(WatchFrameSchema, {
    updates: ["Delta", "Statistics"].map((kind) => ({
      kind: { HarmosVariant: { case: kind as "Delta" | "Statistics", value: {} } },
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
        },
      },
    })),
  });
  const decoded = mapNativePlotStreamFrame(wire).updates;
  assert.deepEqual(
    decoded.map((message) => message.kind),
    ["delta", "statistics"],
  );
  assert.ok(decoded[0].kind !== "pending");
  assert.equal(decoded[0].payload.bucket_width_s, 10);
  assert.deepEqual(decoded[0].payload.replaced_buckets, [1]);
});
