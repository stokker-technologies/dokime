/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { PlotCartesianFrame, PlotSeries } from "@/lib/types";
import { CartesianPlot } from "@/plots/CartesianPlot";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";

/* === Components === */

export function XyIntervalPlot({
  plot,
  resetKey,
  viewportMode = "fit",
}: {
  plot: PlotCardSpec;
  resetKey?: string;
  viewportMode?: PlotViewportMode;
}) {
  const series = intervalSeries(plot);
  const frame: PlotCartesianFrame = {
    mode: "faceted",
    series: series.map((item) => ({
      x: item.points.map((point) => Number(point.x)),
      y: item.points.map((point) => point.y),
    })),
    point_count: series.reduce((count, item) => count + item.points.length, 0),
    signature: JSON.stringify(plot.analysis),
  };

  return (
    <CartesianPlot
      badge="XY Interval"
      intervals
      cartesian={frame}
      colors={plot.colors}
      referenceBands={plot.referenceBands}
      referenceLines={plot.referenceLines}
      resetKey={resetKey}
      series={series}
      stateDetail={plot.stateDetail}
      stateLabel={plot.stateLabel}
      style="line_points"
      timeAxis={false}
      title={plot.title}
      unit={plot.unit}
      liveWindowS={plot.liveWindowS}
      viewportMode={viewportMode}
      xScale={plot.xScale}
      yScale={plot.yScale}
      xFormat={plot.xFormat}
      yFormat={plot.yFormat}
    />
  );
}

/* === Public Functions === */

export function intervalSeries(plot: PlotCardSpec): PlotSeries[] {
  if (plot.analysis?.kind !== "sigma_interval") return [];

  return plot.analysis.series.map((summary) => ({
    key: summary.key,
    label: plot.series.find((series) => series.key === summary.key)?.label ?? summary.key,
    raw_points: summary.bins.reduce((count, bin) => count + bin.count, 0),
    downsampled: false,
    points: summary.bins
      .filter(
        (bin) =>
          Number.isFinite(bin.x) &&
          Number.isFinite(bin.mean) &&
          (plot.xScale !== "log" || bin.x > 0) &&
          (plot.yScale !== "log" || bin.mean > 0),
      )
      .map((bin) => ({
        x: bin.x,
        y: bin.mean,
        low: bin.lower,
        high: bin.upper,
        sample_count: bin.count,
      })),
  }));
}
