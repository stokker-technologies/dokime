/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { CartesianPlot } from "@/plots/CartesianPlot";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";

/* === Components === */

export function FrequencySeriesPlot({
  plot,
  resetKey,
  viewportMode = "fit",
}: {
  plot: PlotCardSpec;
  resetKey?: string;
  viewportMode?: PlotViewportMode;
}) {
  return (
    <CartesianPlot
      badge={frequencyBadge(plot)}
      cartesian={plot.cartesian}
      colors={plot.colors}
      referenceBands={plot.referenceBands}
      referenceLines={plot.referenceLines}
      resetKey={resetKey}
      series={plot.series}
      stateDetail={plot.stateDetail}
      stateLabel={plot.stateLabel}
      style="line"
      timeAxis={false}
      title={plot.title}
      unit={plot.fft.level_unit === "linear" ? plot.unit : plot.fft.level_unit}
      liveWindowS={plot.liveWindowS}
      viewportMode={viewportMode}
      xScale={plot.xScale}
      yScale={plot.yScale}
      xLabel="Frequency (Hz)"
      xFormat={{ kind: "normal" }}
      yFormat={{ kind: "normal" }}
    />
  );
}

/* === Private Functions === */

function frequencyBadge(plot: PlotCardSpec): string {
  const unit = plot.fft.level_unit === "linear" ? "FFT · Hz" : `FFT · Hz · ${plot.fft.level_unit}`;
  return plot.xScale === "log" || plot.yScale === "log" ? `${unit} log` : unit;
}
