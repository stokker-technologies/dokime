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
import { renderToStaticMarkup } from "react-dom/server";
import type uPlot from "uplot";
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import { XyPlot } from "@dokime/plots/catalog/XyPlot";
import { TimeSeriesPlot } from "@dokime/plots/catalog/TimeSeriesPlot";
import { LivePlotGroupGrid } from "@dokime/plots/components/document/PlotGroupGrid";
import { documentPlotSpec } from "@dokime/plots/grouping";
import type { DocumentViewPlot, PlotPayload } from "@dokime/plots/lib/types";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { NativeIntegerValue } from "@dokime/plots/lib/api/native-values";
import { emptyLivePlotState } from "@dokime/plots/live-frames";
import { PlotPlaceholder } from "../../src/PlotPlaceholder";

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
  cartesian: {
    mode: "faceted",
    series: [{ x: [1000, 1000.25, 1000.5], y: [1000, 1000.25, 1000.5] }],
    point_count: 3,
    signature: "position",
  },
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
    setSeries(index: number, options: uPlot.Series) {
      Object.assign(this.series[index], options);
    }
    setData() {}
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
  assert.ok(typeof x.values === "function");
  assert.ok(typeof y.values === "function");
  const splits = [1000, 1000.25, 1000.5];
  const labels = x.values({} as uPlot, splits, 0, 50, 0.01);
  assert.deepEqual(labels, ["1000", "1000.25", "1000.5"]);
  assert.deepEqual(labels, y.values({} as uPlot, splits, 1, 50, 0.01));
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
    const count = host.querySelector(".plot-card span.text-xs")?.textContent?.trim().split(" ")[0];
    assert.equal(Number(count), scale === "linear" ? 4 : 1);
  }
});

test("Document Plot Titles Remain in Their Cards Through Loading, Empty and Failure States", () => {
  const host = document.createElement("div");
  for (const plot_type of ["xy", "time_series"] as const) {
    const plot = { ...descriptor, plot_type };
    for (const status of ["loading", "live", "error"] as const) {
      const state = {
        ...emptyLivePlotState(),
        status,
        ...(status === "live"
          ? {
              payload: {
                ...payload,
                cartesian: { mode: "faceted" as const, signature: "empty", point_count: 0, series: [{ x: [], y: [] }] },
              },
            }
          : {}),
        ...(status === "error" ? { failure: failureOf(new Error("Reload the plot."), "Plot Refresh Failed") } : {}),
      };
      host.innerHTML = renderToStaticMarkup(
        createElement(LivePlotGroupGrid, {
          documentPlots: [plot],
          liveStates: { position: state },
          showGroupHeaders: false,
          title: plot.title,
          viewportMode: "fit",
        }),
      );
      const title = host.querySelector(".plot-card strong");
      assert.equal(title?.textContent, plot.title);
      assert.ok(!title?.classList.contains("sr-only"));
      assert.ok(status === "error" ? host.querySelector('[role="alert"]') : host.querySelector('[data-testid="plot-placeholder"]'));
    }
  }
});

test("Plot Placeholder Failures Retain Their Title and Settings Action", () => {
  const host = document.createElement("div");
  host.innerHTML = renderToStaticMarkup(
    createElement(PlotPlaceholder, {
      title: descriptor.title,
      label: "Stream Error",
      detail: "Reload the plot.",
      tone: "error",
      actions: createElement("button", { type: "button", "aria-label": "Plot Settings" }),
    }),
  );
  assert.equal(host.querySelector(".plot-card strong")?.textContent, descriptor.title);
  assert.ok(host.querySelector(".plot-card button[aria-label]"));
  assert.ok(host.querySelector('.plot-card [role="alert"]'));
});

test("Live and Frozen Plots Keep All Traces in One Card Below the Description", async () => {
  const { host, root } = mountPlot();
  const series = Array.from({ length: 12 }, (_, index) => ({
    key: `trace-${index}`,
    label: `Trace ${index + 1}`,
    raw_points: 3,
    downsampled: false,
    points: [],
  }));
  const plot = documentPlotSpec({
    plot: { ...descriptor, plot_type: "time_series", description: "Signal over the shown window." },
    payload: {
      ...payload,
      series,
      cartesian: { ...payload.cartesian!, point_count: 36, series: series.map(() => payload.cartesian!.series[0]) },
      summaries: series.map((trace) => ({ series_id: trace.key, count: 3, min: 1000, max: 1000.5, mean: 1000.25 })),
    },
  });
  for (const viewportMode of ["live", "fit"] as const) {
    await act(async () =>
      root.render(
        createElement(TimeSeriesPlot, {
          plot,
          viewportMode,
          actions: createElement("button", { type: "button", "aria-label": "Plot Settings" }),
        }),
      ),
    );
    const card = host.querySelector("article.plot-card")!;
    assert.equal(host.querySelectorAll("article").length, 1);
    assert.ok(card.classList.contains("border"));
    const [header, chart, description, statistics] = Array.from(card.children);
    assert.ok(header.querySelector('button[aria-label="Plot Settings"]'));
    assert.ok(header.querySelector("button[aria-pressed]"));
    assert.ok(chart.querySelector(".min-h-60"));
    assert.equal(description.tagName, "P");
    assert.equal(description.textContent, plot.description);
    const table = statistics.querySelector('table[aria-label="Trace Statistics"]')!;
    assert.equal(table.querySelectorAll("tbody tr").length, 12);
    assert.equal(card.querySelectorAll('[data-slot="data-table"], input, select, tfoot').length, 0);
    assert.equal(statistics.querySelectorAll("h1, h2, h3, h4, h5, h6, .border").length, 0);
    const toggle = table.querySelector("tbody button")!;
    await act(async () => toggle.dispatchEvent(new MouseEvent("click", { bubbles: true })));
    assert.equal(toggle.getAttribute("aria-pressed"), "false");
    assert.ok(toggle.querySelector('[data-trace-hidden="true"]'));
    await act(async () => toggle.dispatchEvent(new MouseEvent("click", { bubbles: true })));
    assert.equal(toggle.getAttribute("aria-pressed"), "true");
    assert.equal(table.querySelectorAll("tbody tr").length, 12);
  }
});

test("Live and Frozen Histograms Keep Every Statistic in One Plain Table Inside Their Card", async () => {
  const { host, root } = mountPlot();
  vi.stubGlobal("IntersectionObserver", undefined);
  const exact = new NativeIntegerValue(1152921504606847975n, true);
  for (const count of [1, 12]) {
    const series = Array.from({ length: count }, (_, index) => ({
      key: `metric-${index + 1}`,
      label: `Metric ${index + 1}`,
      raw_points: 42,
      downsampled: false,
      points: [],
    }));
    const plot = { ...descriptor, plot_type: "histogram", unit: "V", description: "Recorded distribution." } as DocumentViewPlot;
    const state = {
      ...emptyLivePlotState(),
      status: "live" as const,
      payload: {
        ...payload,
        series,
        analysis: {
          kind: "histogram" as const,
          series: series.map((item, index) => ({
            key: item.key,
            count: 42,
            bins: [{ lower: 11, upper: 14, count: 42 }],
            statistics: index === count - 1 && count > 1 ? undefined : { mean: exact, median: 12, std: 0.25, rms: 12.6, min: 11, max: 14 },
          })),
        },
      },
    };
    for (const viewportMode of ["live", "fit"] as const) {
      await act(async () =>
        root.render(
          createElement(LivePlotGroupGrid, {
            title: "Distributions",
            documentPlots: [plot],
            liveStates: { [plot.plot_id]: state },
            viewportMode,
          }),
        ),
      );
      const card = host.querySelector("article.plot-card")!;
      assert.ok(card.classList.contains("border"));
      const table = card.querySelector('table[aria-label="Histogram Statistics"]')!;
      assert.equal(host.querySelectorAll("article").length, 1);
      assert.equal(host.querySelectorAll("table").length, 1);
      assert.equal(table.querySelectorAll("tbody tr").length, count);
      assert.equal(card.querySelectorAll('[data-slot="data-table"], input, select, tfoot, dl, section.rounded-lg').length, 0);
      assert.equal(card.children[card.children.length - 2].textContent, plot.description);
      assert.deepEqual(
        [...table.querySelectorAll("thead th")].map((cell) => cell.textContent),
        ["Metric", "Count", "Min [V]", "Max [V]", "Mean [V]", "Median [V]", "Std [V]", "Rms [V]"],
      );
      assert.deepEqual(
        [...table.querySelectorAll("tbody tr:first-child td")].map((cell) => cell.textContent),
        [count === 1 ? plot.title : "Metric 1", "42", "11.00", "14.00", "1.15292e+18", "12.00", "0.25", "12.60"],
      );
      assert.equal(table.querySelector('tbody [data-exact-value="1152921504606847975"]')?.getAttribute("title"), exact.decimal);
      if (count > 1) {
        assert.deepEqual(
          [...table.querySelectorAll("tbody tr:last-child td")].map((cell) => cell.textContent),
          ["Metric 12", "42", "-", "-", "-", "-", "-", "-"],
        );
      }
    }
  }
});
