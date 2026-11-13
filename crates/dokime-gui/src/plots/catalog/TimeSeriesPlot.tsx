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

export function TimeSeriesPlot({
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
      plotId={plot.key}
      badge={plot.style === "line" ? undefined : plot.style.replace("_", " ")}
      cartesian={plot.cartesian}
      colors={plot.colors}
      description={plot.description}
      contextSummary={plot.contextSummary}
      markers={plot.markers}
      promptMarkers={plot.promptMarkers}
      referenceBands={plot.referenceBands}
      referenceLines={plot.referenceLines}
      actions={actions}
      resetKey={resetKey}
      series={plot.series}
      stateDetail={plot.stateDetail}
      stateLabel={plot.stateLabel}
      style={plot.booleanAxis ? "step" : plot.style}
      timeAxis={plot.timeAxis}
      booleanAxis={plot.booleanAxis}
      title={plot.title}
      titleRepeated={plot.titleRepeated}
      unit={plot.unit}
      windowS={plot.windowS}
      viewportMode={viewportMode}
      xScale={plot.xScale}
      yScale={plot.booleanAxis ? "linear" : plot.yScale}
      xFormat={plot.xFormat}
      xElapsedUnit={plot.xElapsedUnit}
      yFormat={plot.yFormat}
      runStartedAt={plot.runStartedAt}
      cycleStartedAt={plot.cycleStartedAt}
      legend={plot.booleanAxis ? null : { summaries: plot.summaries ?? [], pending: plot.summaryPending ?? false }}
    />
  );
}
