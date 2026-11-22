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
import { scaleSequential } from "d3-scale";
import { interpolateYlOrRd } from "d3-scale-chromatic";

// Local Imports
import { cutKey } from "../lib/rpc/canonical";
import { formatUnitLabel } from "../lib/format";
import { PlotPlaceholder } from "../PlotPlaceholder";
import { plotPlaceholderText } from "../plot-placeholder";
import type { PlotCardSpec } from "../types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function HeatStripPlot({ plot }: { plot: PlotCardSpec }) {
  if (!plot.series.some((series) => series.points.length)) {
    return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  }
  let min = Number.POSITIVE_INFINITY;
  let max = Number.NEGATIVE_INFINITY;
  for (const series of plot.series) {
    for (const point of series.points) {
      if (point.y === null || !Number.isFinite(point.y)) continue;
      min = Math.min(min, point.y);
      max = Math.max(max, point.y);
    }
  }
  const present = Number.isFinite(min);
  const color = scaleSequential(interpolateYlOrRd).domain(present ? (min === max ? [min - 1, max + 1] : [min, max]) : [0, 1]);
  return (
    <article className="plot-card grid min-w-0 gap-3 rounded-lg bg-muted/20 p-3" data-testid="heat-strip-plot">
      {!plot.titleRepeated && <strong>{formatUnitLabel(plot.title, plot.unit)}</strong>}
      <div className="grid gap-2 overflow-x-auto">
        {plot.series.map((series) => (
          <div key={series.key} className="grid gap-1" data-facet={series.key}>
            {series.label && <span className="text-sm">{series.label}</span>}
            <div className="flex w-max gap-1">
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
                    className="grid shrink-0 place-items-center rounded border break-all text-center text-xs tabular-nums"
                    style={{
                      width: "3rem",
                      height: "3rem",
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
    </article>
  );
}

/* === Private Functions === */

/* === Entry Point === */
