/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { ReactNode } from "react";

import { CartesianPlot } from "@/plots/CartesianPlot";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";

export function XyLinePlot({
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
      badge="XY Line"
      cartesian={plot.cartesian}
      colors={plot.colors}
      referenceBands={plot.referenceBands}
      referenceLines={plot.referenceLines}
      actions={actions}
      resetKey={resetKey}
      series={plot.series}
      stateDetail={plot.stateDetail}
      stateLabel={plot.stateLabel}
      style="line"
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
