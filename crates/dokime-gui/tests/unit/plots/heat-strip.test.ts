// @vitest-environment happy-dom
/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Heat Strip Samples and Layout
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { onTestFinished, test, vi } from "vitest";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";

// Local Imports
import { mapNativePlotPayload, plotType } from "@/lib/api/native-plot-projection";
import { PlotKind, PlotPayloadSchema, PlotCartesianMode } from "@generated/host/model";
import { HeatStripPlot, heatStripSample } from "../../../src/plots/catalog/HeatStripPlot";
import { documentPlotSpec } from "@/plots/grouping";
import type { DocumentViewPlot, PlotPayload, PlotSeries } from "@/lib/types";
import { usePlotPlayback } from "@/plots/playback";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Dense Heat Strip Layout Wraps Readable Cells and Preserves Sparse Rows and Hover Values", () => {
  const instant = "1970-01-01T00:00:01Z";
  const payload: PlotPayload = {
    stream: "samples",
    x_key: "channel",
    max_points: 100,
    generated_at: "",
    series: [13, 28, 59].map((count, index) => ({
      key: String(index),
      label: ["2.4GHz", "5GHz", "6GHz"][index],
      downsampled: false,
      raw_points: count,
      points: Array.from({ length: count }, (_, channel) => ({
        x: 1 + channel * (index === 2 ? 4 : 1),
        y: channel === 0 ? 2.6 : 0,
        instant,
      })),
    })),
  };
  const descriptor = { plot_id: "heat", title: "Channel Occupancy", plot_type: "heat_strip", y: ["occupancy"] } as DocumentViewPlot;
  const plot = documentPlotSpec({ plot: descriptor, payload });
  const container = document.createElement("div");
  container.innerHTML = renderToStaticMarkup(createElement(HeatStripPlot, { plot }));
  const facets = container.querySelectorAll("[data-facet]");
  assert.equal(facets.length, 3);
  const sparse = facets[0].querySelector<HTMLElement>("[data-cell]");
  assert.equal(sparse?.parentElement?.style.gridTemplateColumns, "repeat(13, minmax(0, 1fr))");
  for (const [index, count] of [
    [1, 28],
    [2, 59],
  ]) {
    const cells = facets[index].querySelectorAll<HTMLElement>("[data-cell]");
    assert.equal(cells.length, count);
    assert.equal(cells[0].parentElement?.style.gridTemplateColumns, "repeat(auto-fit, minmax(min(100%, 2.5rem), 1fr))");
    assert.ok(cells[0].title.includes(" · 2.6 · "));
    assert.ok(cells[1].title.includes(" · 0 · "));
    assert.ok(cells[1].title.endsWith(instant));
    assert.ok(Array.from(cells).every((cell) => cell.dataset.valid === "true" && cell.textContent === cell.dataset.cell));
  }
  assert.equal(facets[2].querySelector<HTMLElement>("[data-cell='233']")?.textContent, "233");
});

test("Sample Selection Uses Only the Selected Instant and Keeps Each Facet's Categories", () => {
  const first = "1970-01-01T00:00:01.000000001Z";
  const last = "1970-01-01T00:00:01.000000002Z";
  const series: PlotSeries[] = [
    {
      key: "a",
      label: "A",
      downsampled: false,
      raw_points: 3,
      points: [
        { x: 1, y: 4, instant: first },
        { x: 1, y: 9, instant: last },
        { x: 6, y: null, instant: first },
      ],
    },
    { key: "b", label: "B", downsampled: false, raw_points: 1, points: [{ x: 36, y: 7, instant: first }] },
  ];
  const selected = heatStripSample(series, last);
  assert.deepEqual(
    selected.map((series) => series.points.map(({ x, y }) => [x, y])),
    [
      [
        [1, 9],
        [6, null],
      ],
      [[36, null]],
    ],
  );
  assert.deepEqual(
    heatStripSample(series, first).map((series) => series.points.map((point) => point.y)),
    [[4, null], [7]],
  );
  assert.equal(series[0].points.length, 3);
  assert.equal(series[0].points[0].y, 4);
  assert.equal(heatStripSample(series, null)[0].points[0].y, null);
});

test("Live Samples Follow New Rows Until Seeking and Latest Resumes Follow", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const host = document.createElement("div");
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    vi.unstubAllGlobals();
  });
  let playback!: ReturnType<typeof usePlotPlayback<number[]>>;
  function Probe({ samples }: { samples: number[] }) {
    playback = usePlotPlayback(
      samples,
      samples.map((sample) => `1970-01-01T00:00:0${sample}Z`),
    );
    return null;
  }
  await act(async () => root.render(createElement(Probe, { samples: [1, 2] })));
  assert.equal(playback.index, 1);
  await act(async () => root.render(createElement(Probe, { samples: [1, 2, 3] })));
  assert.equal(playback.index, 2);
  await act(async () => playback.seek(0));
  await act(async () => root.render(createElement(Probe, { samples: [2, 3, 4] })));
  assert.equal(playback.following, false);
  assert.equal(playback.instant, "1970-01-01T00:00:01Z");
  assert.deepEqual(playback.data, [1, 2, 3]);
  await act(async () => playback.latest());
  assert.equal(playback.following, true);
  assert.equal(playback.instant, "1970-01-01T00:00:04Z");
  assert.deepEqual(playback.data, [2, 3, 4]);
});

test("Heat Strip Projection Preserves Typed Cell Labels Empty Values and Contribution Times", () => {
  const native = create(PlotPayloadSchema, {
    series: [
      { key: "a", label: "a" },
      { key: "b", label: "b" },
    ],
    frame: {
      mode: PlotCartesianMode.FACETED,
      series: [
        {
          x: [{ kind: { case: "unsignedInteger", value: 2n } }, { kind: { case: "unsignedInteger", value: 9007199254740993n } }],
          y: [{ value: 4 }, {}],
          instants: ["1970-01-01T00:00:01Z", ""],
          totals: [],
        },
        {
          x: [{ kind: { case: "text", value: "10" } }, { kind: { case: "text", value: "2" } }],
          y: [{}, { value: 8 }],
          instants: ["", "1970-01-01T00:00:02Z"],
        },
      ],
    },
  });
  const payload = mapNativePlotPayload(native, "heat_strip");
  assert.equal(plotType("heat_strip"), PlotKind.HEAT_STRIP);
  assert.equal(payload.cartesian, null);
  assert.deepEqual(
    payload.series[0].points.map((point) => String(point.x)),
    ["2", "9007199254740993"],
  );
  assert.deepEqual(
    payload.series[1].points.map((point) => point.x),
    ["10", "2"],
  );
  assert.deepEqual(
    payload.series[0].points.map((point) => point.y),
    [4, null],
  );
  assert.equal(payload.series[1].points[1].instant, "1970-01-01T00:00:02Z");
});

test("Heat Strip Projection Refuses Cells Without Matching Sample Instants", () => {
  const native = create(PlotPayloadSchema, {
    series: [{ key: "a", label: "A" }],
    frame: { mode: PlotCartesianMode.FACETED, series: [{ x: [{ kind: { case: "number", value: 6 } }], y: [{ value: 4 }] }] },
  });
  assert.throws(() => mapNativePlotPayload(native, "heat_strip"));
});
