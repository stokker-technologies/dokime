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

// Third Party Imports
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import type uPlot from "uplot";
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import { XyPlot } from "@dokime/plots/catalog/XyPlot";
import { documentPlotSpec } from "@dokime/plots/grouping";
import type { DocumentViewPlot, PlotPayload } from "@dokime/plots/lib/types";

/* === Re-Exports === */

/* === Constants === */

const charts = vi.hoisted(() => ({ options: [] as uPlot.Options[] }));

const descriptor = {
  plot_id: "position",
  title: "XY Position",
  stream: "readings",
  plot_type: "xy",
  x: "x",
  y: ["y"],
} as DocumentViewPlot;

const payload: PlotPayload = {
  stream: "readings",
  x_key: "x",
  max_points: 100,
  generated_at: "2026-01-01T00:00:00Z",
  series: [{ key: "position", label: "Position", raw_points: 3, downsampled: false, points: [] }],
  cartesian: { mode: "faceted", series: [{ x: [1000, 1000.25, 1000.5], y: [1000, 1000.25, 1000.5] }], point_count: 3, signature: "position" },
};

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function mountPlot() {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({ width: 600, height: 230 } as DOMRect);
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    host.remove();
    charts.options.length = 0;
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });
  return { host, root };
}

vi.mock("uplot", () => ({
  default: class {
    static paths = {};
    over = document.createElement("div");
    series: uPlot.Series[];
    constructor(options: uPlot.Options) {
      charts.options.push(options);
      this.series = options.series;
    }
    setSize() {}
    destroy() {}
  },
}));

/* === Entry Point === */

test("XY Axes Keep Numeric Tick Spacing and Share Number Formatting", async () => {
  const { root } = mountPlot();
  const plot = documentPlotSpec({ plot: descriptor, payload });
  await act(async () => root.render(createElement(XyPlot, { plot })));
  assert.equal(charts.options.length, 1);
  const options = charts.options[0];
  assert.equal(options.scales?.x.time, false);
  const [x, y] = options.axes ?? [];
  assert.ok(!Object.hasOwn(x, "space") || (typeof x.space === "number" && Number.isFinite(x.space) && x.space > 0));
  assert.equal(typeof x.values, "function");
  assert.equal(typeof y.values, "function");
  const splits = [1000, 1000.25, 1000.5];
  const labels = (x.values as uPlot.Axis.Values)({} as uPlot, splits, 0, 50, 0.25);
  assert.deepEqual(labels, ["1000.00", "1000.25", "1000.50"]);
  assert.deepEqual(labels, (y.values as uPlot.Axis.Values)({} as uPlot, splits, 1, 50, 0.25));
});

test("XY Frames With No Drawable Pairs Show an Empty State and Recover", async () => {
  const { host, root } = mountPlot();
  for (const mode of ["aligned", "faceted"] as const) {
    for (const series of [
      { x: [0, 1, 2], y: [null, null, null] },
      { x: [Number.NaN, Number.POSITIVE_INFINITY, 2], y: [1, 2, null] },
    ]) {
      const plot = documentPlotSpec({
        plot: descriptor,
        payload: { ...payload, cartesian: { mode, series: [series], point_count: 91, signature: `missing:${mode}` } },
      });
      await act(async () => root.render(createElement(XyPlot, { plot })));
      assert.ok(host.querySelector('[data-testid="plot-placeholder"]'));
      assert.equal(host.querySelector(".plot-card strong")?.textContent, descriptor.title);
      assert.equal(charts.options.length, 0);
      assert.ok(!host.textContent?.includes("91"));
    }
  }
  const plot = documentPlotSpec({ plot: descriptor, payload });
  await act(async () => root.render(createElement(XyPlot, { plot })));
  assert.equal(host.querySelector('[data-testid="plot-placeholder"]'), null);
  assert.equal(charts.options.length, 1);
});

test("XY Counts Only Finite Pairs Admitted by the Selected Scales", async () => {
  const { host, root } = mountPlot();
  for (const scale of ["linear", "log"] as const) {
    const plot = documentPlotSpec({
      plot: { ...descriptor, x_scale: scale, y_scale: scale },
      payload: {
        ...payload,
        cartesian: {
          mode: "faceted",
          series: [{ x: [-1, 0, 1, 2, 3, Number.NaN], y: [2, 1, -1, 1, null, 1] }],
          point_count: 91,
          signature: scale,
        },
      },
    });
    await act(async () => root.render(createElement(XyPlot, { plot })));
    const count = host.querySelector(".plot-card .text-xs")?.textContent?.trim().split(" ")[0];
    assert.equal(Number(count), scale === "linear" ? 4 : 1);
  }
});
