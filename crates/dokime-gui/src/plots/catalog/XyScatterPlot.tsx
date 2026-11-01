/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { CartesianPlot } from "@/plots/CartesianPlot";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";

/* === Components === */

export function XyScatterPlot({
  actions,
  plot,
  resetKey,
  viewportMode = "fit",
}: {
  actions?: ReactNode;
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
      actions={actions}
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
