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

export function XyScatterPlot({
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
      badge="XY Scatter"
      cartesian={plot.cartesian}
      colors={plot.colors}
      referenceBands={plot.referenceBands}
      referenceLines={plot.referenceLines}
      resetKey={resetKey}
      series={plot.series}
      stateDetail={plot.stateDetail}
      stateLabel={plot.stateLabel}
      style="points"
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
