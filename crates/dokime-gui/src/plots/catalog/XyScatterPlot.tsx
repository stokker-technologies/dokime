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
  const { zoom } = usePlotInteraction();
  const setSummaryWindow = useSummaryWindowStore((state) => state.setSummaryWindow);
  // A live zoom asks the backend for the zoomed range's statistics; a report's
  // statistics stay frozen over the window it captured.
  const onViewportChange = useCallback((window: SummaryWindow | null) => setSummaryWindow(plot.key, window), [plot.key, setSummaryWindow]);
  useEffect(() => () => setSummaryWindow(plot.key, null), [plot.key, setSummaryWindow]);
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
      legend={{ summaries: plot.summaries ?? [], pending: plot.summaryPending ?? false }}
      onViewportChange={zoom ? undefined : onViewportChange}
    />
  );
}
