/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Heat Strip Plot

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useMemo } from "react";

// Local Imports
import { sequentialColorScale } from "../colors";
import { cutKey } from "../lib/rpc/canonical";
import { formatUnitLabel } from "../lib/format";
import type { PlotPoint, PlotSeries } from "../lib/types";
import { PlotPlaceholder } from "../PlotPlaceholder";
import { plotPlaceholderText } from "../plot-placeholder";
import { instantTime, orderedInstants, PlotPlayback, usePlotPlayback } from "../playback";
import type { PlotCardSpec } from "../types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function HeatStripPlot({ plot }: { plot: PlotCardSpec }) {
  const instants = useMemo(
    () => orderedInstants(plot.series.flatMap((series) => series.points.flatMap((point) => (point.instant ? [point.instant] : [])))),
    [plot.series],
  );
  const playback = usePlotPlayback(plot.series, instants);
  const series = useMemo(() => heatStripSample(playback.data, playback.instant), [playback.data, playback.instant]);
  if (!playback.data.some((series) => series.points.length)) {
    return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  }
  const { min, max, present, color } = sequentialColorScale(playback.data.flatMap((series) => series.points.map((point) => point.y)));
  return (
    <article className="plot-card grid min-w-0 gap-3 rounded-lg bg-muted/20 p-3" data-testid="heat-strip-plot">
      {!plot.titleRepeated && <strong>{formatUnitLabel(plot.title, plot.unit)}</strong>}
      <div className="grid min-w-0 gap-2">
        {series.map((series) => (
          <div key={series.key} className="grid gap-1" data-facet={series.key}>
            {series.label && <span className="text-sm">{series.label}</span>}
            <div
              className="grid w-full min-w-0 gap-1"
              style={{
                gridTemplateColumns:
                  series.points.length > 16
                    ? "repeat(auto-fit, minmax(min(100%, 2.5rem), 1fr))"
                    : `repeat(${series.points.length}, minmax(0, 1fr))`,
              }}
            >
              {series.points.map((point) => {
                const valid = point.y !== null && Number.isFinite(point.y);
                const label = String(point.x ?? "");
                const title = [label, series.label, valid ? String(point.exact_y ?? point.y) : "No Value", point.instant]
                  .filter(Boolean)
                  .join(" · ");
                return (
                  <div
                    key={cutKey(point.x)}
                    data-cell={label}
                    data-valid={valid}
                    title={title}
                    className="grid min-w-0 place-items-center rounded border px-1 break-all text-center text-xs tabular-nums"
                    style={{
                      minHeight: "3rem",
                      backgroundColor: valid && point.y !== null ? color(point.y) : undefined,
                      color: valid
                        ? max > min && point.y !== null && (point.y - min) / (max - min) > 0.72
                          ? "#fff"
                          : "#18181b"
                        : undefined,
                    }}
                  >
                    {label}
                  </div>
                );
              })}
            </div>
          </div>
        ))}
      </div>
      {present && (
        <div className="flex items-center gap-2 text-xs tabular-nums" data-heat-legend="true" data-min={min} data-max={max}>
          <span>{String(min)}</span>
          <span
            className="h-3 w-32 rounded"
            style={{
              background: `linear-gradient(to right, ${Array.from({ length: 11 }, (_, index) => color(min + ((max - min) * index) / 10)).join(", ")})`,
            }}
          />
          <span>{String(max)}</span>
          {plot.unit && <span>[{plot.unit}]</span>}
        </div>
      )}
      <PlotPlayback title={plot.title} playback={playback} />
    </article>
  );
}

export function heatStripSample(series: PlotSeries[], instant: string | null): PlotSeries[] {
  const time = instant ? instantTime(instant) : null;
  return series.map((series) => {
    const cells = new Map<string, PlotPoint>();
    for (const point of series.points) {
      const key = cutKey(point.x);
      if (!cells.has(key)) {
        cells.set(key, { x: point.x, y: null, instant });
      }
      const stamp = point.instant ? instantTime(point.instant) : null;
      if (time !== null && stamp === time) {
        cells.set(key, point);
      }
    }
    return { ...series, points: [...cells.values()] };
  });
}

/* === Private Functions === */

/* === Entry Point === */
