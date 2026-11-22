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
import { type Component, mount, unmount } from "svelte";

// Local Imports
import type { DocumentPlotStyle, PlotCartesianFrame, PlotSeries } from "@/lib/types";
import CartesianPlot from "@/plots/CartesianPlot.svelte";
import { plotInteractionContext } from "@/plots/plot-interaction";
import HistogramChart from "@/plots/HistogramChart.svelte";
import PiePlot from "@/plots/catalog/PiePlot.svelte";
import BarPlot from "@/plots/catalog/BarPlot.svelte";
import BoxPlot from "@/plots/catalog/BoxPlot.svelte";
import ContourPlot from "@/plots/catalog/ContourPlot.svelte";
import XyIntervalPlot from "@/plots/catalog/XyIntervalPlot.svelte";
import TrajectoryPlot from "@/plots/catalog/TrajectoryPlot.svelte";
import { documentPlotSpec } from "@/plots/grouping";
import type { DocumentViewPlot, PlotPayload } from "@/lib/types";
import GridChartPlot from "@/plots/catalog/GridChartPlot.svelte";
import type { PlotCardSpec } from "@/plots/types";
import "../../../src/desktop.css";

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
      playback: (frozen: boolean) => Promise<void>;
    };
  }
}

/* === Components === */

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

async function settled() {
  for (let frame = 0; frame < 8; frame += 1) {
    await new Promise(requestAnimationFrame);
  }
}

function startCanvasFixture() {
  const rootElement = document.getElementById("root");
  if (!rootElement) {
    throw new Error("Canvas fixture root is missing");
  }
  const mounted: Record<string, unknown>[] = [];

  function render(content: () => HTMLElement) {
    for (const instance of mounted.splice(0)) {
      void unmount(instance);
    }
    rootElement!.replaceChildren(content());
  }

  function chart<Props extends Record<string, unknown>>(component: Component<Props>, props: Props, zoom: boolean): HTMLElement {
    const target = document.createElement("div");
    target.className = "contents";
    const context = new Map([[plotInteractionContext, { zoom }]]);
    const instance = mount(component, { target, props, context });
    mounted.push(instance);
    return target;
  }

  function group(attributes: Record<string, string | number>, children: HTMLElement[]): HTMLDivElement {
    const element = document.createElement("div");
    for (const [name, value] of Object.entries(attributes)) {
      element.setAttribute(name, String(value));
    }
    element.append(...children);
    return element;
  }
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
    if (!points) {
      return;
    }
    const samples = painted.get(this.canvas) ?? new Set<string>();
    for (const [x, y] of points) {
      if (x < 0 || y < 0 || x >= this.canvas.width || y >= this.canvas.height) {
        continue;
      }
      const pixel = this.getImageData(Math.floor(x), Math.floor(y), 1, 1).data;
      if (pixel[3] > 0) {
        samples.add(`${x}:${y}`);
      }
    }
    painted.set(this.canvas, samples);
  };
  CanvasRenderingContext2D.prototype.clearRect = function (x, y, width, height) {
    painted.delete(this.canvas);
    return nativeClear.call(this, x, y, width, height);
  };

  window.canvasFixture = {
    async playback(frozen) {
      const instants = ["2026-01-01T00:00:00Z", "2026-01-01T00:00:02Z", "2026-01-01T00:00:09Z"];
      const declaration = {
        plot_id: "track",
        title: "Trajectory",
        stream: "readings",
        plot_type: "trajectory",
        x: "x",
        y: ["y"],
      } as DocumentViewPlot;
      const payload: PlotPayload = {
        stream: "readings",
        x_key: "x",
        max_points: 100,
        generated_at: instants[2],
        series: [
          {
            key: "track",
            label: "Track",
            raw_points: 3,
            downsampled: false,
            points: instants.map((instant, i) => ({ x: i % 2, y: i, instant })),
          },
        ],
      };
      const trajectory = documentPlotSpec({ plot: declaration, payload });
      const grid = documentPlotSpec({
        plot: { ...declaration, plot_id: "grid", title: "Grid", plot_type: "grid_chart" },
        payload: {
          ...payload,
          series: [],
          analysis: {
            kind: "grid_chart",
            data: {
              rows: [{ key: "sensor", label: "Sensor", order: 0 }],
              columns: [{ key: "site", label: "Site", order: 0 }],
              frames: instants.map((instant, i) => ({ instant, values: [i], valid: [true], highlights: [] })),
              summaryLabel: "Readings",
            },
          },
        },
      });
      render(() => group({}, [chart(TrajectoryPlot, { plot: trajectory }, frozen), chart(GridChartPlot, { plot: grid }, frozen)]));
      await settled();
    },
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
          kind: "box_plot",
          total: 12,
          series: [{ key: "A", label: "A", lower_whisker: 0, q1: 1, median: 2, q3: 3, upper_whisker: 4, outliers: [5] }],
        },
      } as unknown as PlotCardSpec;
      render(() =>
        group({}, [
          group({ "data-plot-kind": "histogram" }, [
            chart(
              HistogramChart,
              {
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
              },
              false,
            ),
          ]),
          group({ "data-plot-kind": "bar" }, [chart(BarPlot, { plot }, false)]),
          group({ "data-plot-kind": "box" }, [chart(BoxPlot, { plot }, false)]),
          group({ "data-plot-kind": "grid" }, [
            chart(
              GridChartPlot,
              {
                plot: {
                  ...plot,
                  analysis: {
                    kind: "grid_chart",
                    data: {
                      rows: [{ key: "sensor", label: "Sensor", order: 0 }],
                      columns: [
                        { key: "left", label: "Left", order: 0 },
                        { key: "right", label: "Right", order: 1 },
                      ],
                      frames: [{ instant: "2026-01-01T00:00:00Z", values: [2, 3], valid: [true, true], highlights: [] }],
                      summaryLabel: "Readings",
                    },
                  },
                },
              },
              false,
            ),
          ]),
          ...["cells", "bands"].map((render) =>
            chart(
              ContourPlot,
              {
                plot: documentPlotSpec({
                  plot: {
                    plot_id: `contour.${render}`,
                    title: "Surface",
                    stream: "readings",
                    plot_type: "contour",
                    y: ["y"],
                    render,
                  } as DocumentViewPlot,
                  payload: {
                    stream: "readings",
                    x_key: "x",
                    series: [],
                    max_points: 4,
                    generated_at: "2026-01-01T00:00:00Z",
                    analysis: {
                      kind: "contour",
                      grid: {
                        columns: 2,
                        rows: 2,
                        xEdges: [0, 1, 2],
                        yEdges: [0, 1, 2],
                        values: [0, 0, 0, 4],
                        valid: [true, false, false, true],
                      },
                    },
                  },
                }),
              },
              false,
            ),
          ),
          ...["grouped", "stacked", "pareto"].map((option) =>
            chart(
              BarPlot,
              {
                plot: {
                  ...plot,
                  plotType: "bar",
                  arrangement: option === "pareto" ? null : (option as "grouped" | "stacked"),
                  pareto: option === "pareto",
                  analysis: null,
                  series: (option === "pareto" ? [0] : [0, 1]).map((index) => ({
                    key: `series${index}`,
                    label: `Series ${index}`,
                    raw_points: 3,
                    downsampled: false,
                    points: [
                      { x: "a", y: 2 + index },
                      { x: "b", y: 3 + index },
                      { x: "c", y: 3 + index },
                    ],
                  })),
                },
              },
              false,
            ),
          ),
          chart(
            XyIntervalPlot,
            {
              plot: documentPlotSpec({
                plot: {
                  plot_id: "interval",
                  title: "Interval",
                  stream: "readings",
                  plot_type: "xy",
                  xy_interval: "sigma",
                  y: ["value"],
                } as DocumentViewPlot,
                payload: {
                  stream: "readings",
                  x_key: "x",
                  max_points: 10,
                  generated_at: "2026-01-01T00:00:00Z",
                  series: [],
                  analysis: {
                    kind: "sigma_interval",
                    series: [{ key: "value", raw_points: 5, bins: [{ x: 1, count: 5, mean: 4, lower: 2, upper: 6 }] }],
                  },
                },
              }),
            },
            false,
          ),
          ...[0, 0.5].map((hole) =>
            group({ "data-pie-hole": hole }, [
              chart(
                PiePlot,
                {
                  plot: {
                    ...plot,
                    plotType: "pie",
                    hole,
                    analysis: null,
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
                  },
                },
                false,
              ),
            ]),
          ),
        ]),
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
      render(() =>
        chart(
          CartesianPlot,
          {
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
          },
          zoom,
        ),
      );
      await settled();
    },
    counts: () => [...document.querySelectorAll("canvas")].map((canvas) => painted.get(canvas)?.size ?? 0),
    async toggle(index) {
      const row = document.querySelectorAll("tbody tr")[index];
      if (!(row instanceof HTMLElement)) {
        throw new Error("Legend row is missing");
      }
      row.click();
      await settled();
    },
  };
}

startCanvasFixture();
