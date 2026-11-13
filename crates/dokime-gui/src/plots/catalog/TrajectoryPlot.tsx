/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type ReactNode, useMemo } from "react";

/* === Local Imports === */

import type { PlotCartesianFrame } from "@/lib/types";
import { CartesianPlot } from "@/plots/CartesianPlot";
import { instantTime, orderedInstants, PlotPlayback, usePlotPlayback } from "@/plots/playback";
import type { PlotCardSpec, PlotViewportMode } from "@/plots/types";

/* === Components === */

export function TrajectoryPlot({
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
  const instants = useMemo(
    () => orderedInstants(plot.series.flatMap((series) => series.points.flatMap((point) => (point.instant ? [point.instant] : [])))),
    [plot.series],
  );
  const playback = usePlotPlayback(plot.series, instants);
  const time = playback.instant ? instantTime(playback.instant) : null;
  const series = useMemo(
    () =>
      playback.data.map((series) => ({
        ...series,
        points: series.points
          .filter((point) => {
            const stamp = point.instant ? instantTime(point.instant) : null;
            return time !== null && stamp !== null && stamp <= time;
          })
          .sort((a, b) => {
            const left = instantTime(a.instant ?? "") ?? 0n;
            const right = instantTime(b.instant ?? "") ?? 0n;
            return left < right ? -1 : left > right ? 1 : 0;
          }),
      })),
    [playback.data, time],
  );
  const frame: PlotCartesianFrame = {
    mode: "faceted",
    series: series.map((item) => ({ x: item.points.map((point) => Number(point.x)), y: item.points.map((point) => point.y) })),
    point_count: series.reduce((count, item) => count + item.points.length, 0),
    signature: JSON.stringify(series),
  };
  return (
    <div className="grid min-w-0 gap-2" data-trajectory-instant={playback.instant ?? ""}>
      <CartesianPlot
        plotId={plot.key}
        badge="Trajectory"
        cartesian={frame}
        currentPosition
        localViewport
        colors={plot.colors}
        referenceBands={plot.referenceBands}
        referenceLines={plot.referenceLines}
        actions={actions}
        resetKey={resetKey}
        series={series}
        stateDetail={plot.stateDetail}
        stateLabel={plot.stateLabel}
        style="line_points"
        timeAxis={false}
        title={plot.title}
        unit={plot.unit}
        windowS={plot.windowS}
        viewportMode={viewportMode}
        xScale={plot.xScale}
        yScale={plot.yScale}
        xFormat={plot.xFormat}
        yFormat={plot.yFormat}
      />
      <PlotPlayback title={plot.title} playback={playback} />
    </div>
  );
}
