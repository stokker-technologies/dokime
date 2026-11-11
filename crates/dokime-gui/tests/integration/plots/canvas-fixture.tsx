/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Exercise Shared Live And Frozen Plot Rendering In A Real Canvas.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { createElement } from "react";
import { createRoot } from "react-dom/client";

// Local Imports
import type { DocumentPlotStyle, PlotCartesianFrame, PlotSeries } from "@/lib/types";
import { CartesianPlot } from "@/plots/CartesianPlot";
import { PlotInteractionProvider } from "@/plots/interaction";
import { HistogramChart } from "@/components/HistogramChart";
import { BarPlot } from "@/plots/catalog/BarPlot";
import { BoxSummarySeriesPlot } from "@/plots/catalog/BoxSummaryPlot";
import { WifiChannelOccupancyPlot } from "@/plots/catalog/WifiChannelOccupancyPlot";
import type { PlotCardSpec } from "@/plots/types";
import "@/styles.css";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

declare global {
  interface Window {
    canvasFixture: {
      mount: (style: DocumentPlotStyle, faceted: boolean, zoom: boolean, dark: boolean, withMarkers?: boolean) => Promise<void>;
      counts: () => number[];
      toggle: (index: number) => Promise<void>;
      summaries: (dark: boolean, touch: boolean) => Promise<void>;
    };
  }
}

/* === Components === */

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

async function settled() {
  for (let frame = 0; frame < 8; frame += 1) await new Promise(requestAnimationFrame);
}

function startCanvasFixture() {
  const rootElement = document.getElementById("root");
  if (!rootElement) throw new Error("Canvas fixture root is missing");
  const root = createRoot(rootElement);
  const arcs = new WeakMap<Path2D, Array<[number, number]>>();
  const painted = new WeakMap<HTMLCanvasElement, Set<string>>();
  const nativeArc = Path2D.prototype.arc;
  const nativeFill = CanvasRenderingContext2D.prototype.fill;
  const nativeClear = CanvasRenderingContext2D.prototype.clearRect;

  Path2D.prototype.arc = function (x, y, radius, start, end, counterclockwise) {
    const points = arcs.get(this) ?? [];
    points.push([x, y]);
    arcs.set(this, points);
    return nativeArc.call(this, x, y, radius, start, end, counterclockwise);
  };
  CanvasRenderingContext2D.prototype.fill = function (pathOrRule?: Path2D | CanvasFillRule, rule?: CanvasFillRule) {
    Reflect.apply(nativeFill, this, pathOrRule instanceof Path2D ? [pathOrRule, rule] : [pathOrRule]);
    const points = pathOrRule instanceof Path2D ? arcs.get(pathOrRule) : null;
    if (!points) return;
    const samples = painted.get(this.canvas) ?? new Set<string>();
    for (const [x, y] of points) {
      if (x < 0 || y < 0 || x >= this.canvas.width || y >= this.canvas.height) continue;
      const pixel = this.getImageData(Math.floor(x), Math.floor(y), 1, 1).data;
      if (pixel[3] > 0) samples.add(`${x}:${y}`);
    }
    painted.set(this.canvas, samples);
  };
  CanvasRenderingContext2D.prototype.clearRect = function (x, y, width, height) {
    painted.delete(this.canvas);
    return nativeClear.call(this, x, y, width, height);
  };

  window.canvasFixture = {
    async summaries(dark, touch) {
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.dataset.density = touch ? "touch" : "comfortable";
      const plot = {
        title: "Distribution",
        unit: "V",
        colors: [],
        series: [
          {
            key: "signal",
            label: "Signal",
            points: [
              { x: "A", y: 5 },
              { x: "B", y: -3 },
            ],
          },
        ],
        analysis: {
          kind: "box_summary",
          total: 12,
          series: [{ key: "A", label: "A", lower_whisker: 0, q1: 1, median: 2, q3: 3, upper_whisker: 4, outliers: [5] }],
        },
      } as unknown as PlotCardSpec;
      root.render(
        createElement(
          "div",
          {},
          createElement(
            "div",
            { "data-plot-kind": "histogram" },
            createElement(HistogramChart, {
              title: "Samples",
              unit: "V",
              distribution: {
                count: 12,
                bins: [
                  { lower: 0, upper: 1, count: 3 },
                  { lower: 1, upper: 2, count: 0 },
                  { lower: 2, upper: 3, count: 9 },
                ],
              },
            }),
          ),
          createElement("div", { "data-plot-kind": "bar" }, createElement(BarPlot, { plot })),
          createElement("div", { "data-plot-kind": "box" }, createElement(BoxSummarySeriesPlot, { plot })),
          createElement(
            "div",
            { "data-plot-kind": "wifi" },
            createElement(WifiChannelOccupancyPlot, {
              plot: {
                ...plot,
                analysis: {
                  kind: "wifi_occupancy",
                  channels: [1, 6],
                  occupied_channels: [1, 6],
                  bands: [{ band: "2.4 GHz", occupied_channels: 2 }],
                  scans: [
                    {
                      time: "2026-01-01T00:00:00Z",
                      channels: [
                        { channel: 1, count: 2 },
                        { channel: 6, count: 3 },
                      ],
                      bands: [{ band: "2.4 GHz", occupied_channels: 2, total_networks: 5, peak_count: 3 }],
                      occupied_channels: 2,
                      total_networks: 5,
                      peak_count: 3,
                    },
                  ],
                },
              },
            }),
          ),
        ),
      );
      await settled();
    },
    async mount(style, faceted, zoom, dark, withMarkers = false) {
      document.documentElement.classList.toggle("dark", dark);
      const x = Array.from({ length: 20 }, (_, index) => 1_700_000_000 + index * 11);
      const ys = [0, 1].map((parity) => x.map((_, index) => (index % 2 === parity ? 10 + index : null)));
      const series: PlotSeries[] = ys.map((y, index) => ({
        key: `trace-${index}`,
        label: `Trace ${index}`,
        points: x.map((at, row) => ({ x: at, y: y[row] })),
        raw_points: 10,
        downsampled: false,
      }));
      const cartesian: PlotCartesianFrame = {
        mode: faceted ? "faceted" : "aligned",
        series: ys.map((y) => (faceted ? { x: [...x, null, x[x.length - 1] + 11] as unknown as number[], y: [...y, 40, null] } : { x, y })),
        point_count: 20,
        signature: `${style}:${faceted}:${zoom}:${dark}:${withMarkers}`,
      };
      root.render(
        createElement(
          PlotInteractionProvider,
          { zoom },
          createElement(CartesianPlot, {
            key: cartesian.signature,
            title: "Samples",
            cartesian,
            series,
            style,
            timeAxis: !faceted,
            markers: withMarkers
              ? [
                  { x: x[4], label: "fixture ".repeat(40), record_type: "event-one", color: "" },
                  { x: x[6], label: "Event Two", record_type: "event-two", color: "" },
                  { x: x[14], label: "fixture".repeat(40), record_type: "event-one", color: "" },
                ]
              : [],
            promptMarkers: withMarkers
              ? [
                  { x: x[6], title: "fixture ".repeat(40), kind: "asked" },
                  { x: x[12], title: "fixture".repeat(40), kind: "answered" },
                ]
              : [],
            legend: { summaries: series.map((item) => ({ series_id: item.key, count: 10 })), pending: false },
          }),
        ),
      );
      await settled();
    },
    counts: () => [...document.querySelectorAll("canvas")].map((canvas) => painted.get(canvas)?.size ?? 0),
    async toggle(index) {
      const row = document.querySelectorAll("tbody tr")[index];
      if (!(row instanceof HTMLElement)) throw new Error("Legend row is missing");
      row.click();
      await settled();
    },
  };
}

startCanvasFixture();
