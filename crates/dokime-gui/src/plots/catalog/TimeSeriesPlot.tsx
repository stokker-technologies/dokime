/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type ReactNode, useCallback, useEffect } from "react";

/* === Local Imports === */

import { CartesianPlot } from "@/plots/CartesianPlot";
import { usePlotInteraction } from "@/plots/interaction";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";
import { type SummaryWindow, useSummaryWindowStore } from "@/stores/summary-window-store";

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
  const { zoom } = usePlotInteraction();
  const setSummaryWindow = useSummaryWindowStore((state) => state.setSummaryWindow);
  // A live zoom asks the backend for the zoomed range's statistics; a report's
  // statistics stay frozen over the window it captured.
  const onViewportChange = useCallback((window: SummaryWindow | null) => setSummaryWindow(plot.key, window), [plot.key, setSummaryWindow]);
  useEffect(() => () => setSummaryWindow(plot.key, null), [plot.key, setSummaryWindow]);
  return (
    <CartesianPlot
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
      liveWindowS={plot.liveWindowS}
      viewportMode={viewportMode}
      xScale={plot.xScale}
      yScale={plot.booleanAxis ? "linear" : plot.yScale}
      xFormat={plot.xFormat}
      xElapsedUnit={plot.xElapsedUnit}
      yFormat={plot.yFormat}
      runStartedAt={plot.runStartedAt}
      cycleStartedAt={plot.cycleStartedAt}
      legend={plot.booleanAxis ? null : { summaries: plot.summaries ?? [], pending: plot.summaryPending ?? false }}
      onViewportChange={zoom || plot.booleanAxis ? undefined : onViewportChange}
    />
  );
}
