/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { contours } from "d3-contour";
import { geoPath } from "d3-geo";
import { scaleLinear, scaleLog, scaleSequential } from "d3-scale";
import { interpolateYlOrRd } from "d3-scale-chromatic";
import { useEffect, useId, useRef } from "react";

/* === Local Imports === */

import { formatNumber, formatUnitLabel } from "@/lib/format";
import type { ContourGrid } from "@/lib/types";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText } from "@/plots/plot-placeholder";
import type { PlotCardSpec } from "@/plots/types";
import { useSummaryWindowStore } from "@/stores/summary-window-store";

/* === Types === */

/* === Constants === */

const WIDTH = 720;
const HEIGHT = 420;
const MARGIN = { left: 58, right: 24, top: 20, bottom: 52 };

/* === Components === */

export function ContourPlot({ plot }: { plot: PlotCardSpec }) {
  const clip = useId();
  const element = useRef<SVGSVGElement>(null);
  const setPanelSize = useSummaryWindowStore((store) => store.setPanelSize);
  const hasGrid = plot.analysis?.kind === "contour" && plot.analysis.grid.valid.some(Boolean);
  useEffect(() => {
    if (!hasGrid || !element.current || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (!entry || entry.contentRect.width <= 0 || entry.contentRect.height <= 0) return;
      let width = Math.max(1, Math.round((entry.contentRect.width * (WIDTH - MARGIN.left - MARGIN.right)) / WIDTH));
      let height = Math.max(1, Math.round((entry.contentRect.height * (HEIGHT - MARGIN.top - MARGIN.bottom)) / HEIGHT));
      const factor = Math.min(1, Math.sqrt(262_144 / (width * height)), 512 / height);
      width = Math.max(1, Math.floor(width * factor));
      height = Math.max(1, Math.floor(height * factor));
      setPanelSize(plot.key, { width, height });
    });
    observer.observe(element.current);
    return () => {
      observer.disconnect();
      setPanelSize(plot.key, null);
    };
  }, [plot.key, setPanelSize, hasGrid]);
  const grid = plot.analysis?.kind === "contour" ? plot.analysis.grid : null;
  if (!grid || !validContourGrid(grid) || !grid.valid.some(Boolean)) {
    return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  }
  const width = WIDTH - MARGIN.left - MARGIN.right;
  const height = HEIGHT - MARGIN.top - MARGIN.bottom;
  const values = grid.values.filter((_, index) => grid.valid[index]);
  let min = Number.POSITIVE_INFINITY;
  let max = Number.NEGATIVE_INFINITY;
  for (const value of values) {
    min = Math.min(min, value);
    max = Math.max(max, value);
  }
  const domain: [number, number] = min === max ? [min - 1, max + 1] : [min, max];
  const color = scaleSequential(interpolateYlOrRd).domain(domain);
  const x = (plot.xScale === "log" ? scaleLog : scaleLinear)(
    [grid.x_edges[0], grid.x_edges[grid.columns]],
    [MARGIN.left, MARGIN.left + width],
  );
  const y = (plot.yScale === "log" ? scaleLog : scaleLinear)([grid.y_edges[0], grid.y_edges[grid.rows]], [MARGIN.top + height, MARGIN.top]);
  const path = geoPath();
  const shapes =
    plot.render === "bands"
      ? contours().size([grid.columns, grid.rows]).smooth(false).thresholds(8)(
          grid.values.map((value, index) => (grid.valid[index] ? value : Number.NaN)),
        )
      : [];
  const cells = grid.values.flatMap((value, index) =>
    grid.valid[index] ? [{ value, index, column: index % grid.columns, row: Math.floor(index / grid.columns) }] : [],
  );

  return (
    <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="contour-plot">
      <h4 className="truncate text-sm font-semibold" title={formatUnitLabel(plot.title, plot.unit)}>
        {formatUnitLabel(plot.title, plot.unit)}
      </h4>
      <svg
        ref={element}
        role="img"
        aria-label={formatUnitLabel(plot.title, plot.unit)}
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        className="aspect-[12/7] w-full min-w-0 rounded-lg bg-background"
      >
        <defs>
          <clipPath id={clip} clipPathUnits="userSpaceOnUse">
            {cells.map((cell) => (
              <rect key={cell.index} data-valid-bin={cell.index} x={cell.column} y={cell.row} width={1} height={1} />
            ))}
          </clipPath>
        </defs>
        <g transform={`translate(${MARGIN.left},${MARGIN.top + height}) scale(${width / grid.columns},${-height / grid.rows})`}>
          {cells.map((cell) => (
            <rect key={cell.index} data-bin={cell.index} x={cell.column} y={cell.row} width={1} height={1} fill={color(cell.value)}>
              <title>{`${formatNumber(grid.x_edges[cell.column])}–${formatNumber(grid.x_edges[cell.column + 1])}, ${formatNumber(grid.y_edges[cell.row])}–${formatNumber(grid.y_edges[cell.row + 1])}: ${formatUnitLabel(formatNumber(cell.value), plot.unit)}`}</title>
            </rect>
          ))}
          {plot.render === "bands" ? (
            <g clipPath={`url(#${clip})`} data-contour-mask>
              {shapes.map((shape) => (
                <path
                  key={shape.value}
                  d={path(shape) ?? undefined}
                  fill={color(shape.value)}
                  stroke="var(--foreground)"
                  strokeOpacity={0.35}
                  strokeWidth={0.6}
                  vectorEffect="non-scaling-stroke"
                />
              ))}
            </g>
          ) : null}
          <g clipPath={`url(#${clip})`}>
            {plot.referenceBands.map((band) => {
              if (typeof band.min !== "number" || typeof band.max !== "number") return null;
              const bottom = ((MARGIN.top + height - y(band.min)) * grid.rows) / height;
              const top = ((MARGIN.top + height - y(band.max)) * grid.rows) / height;
              if (!Number.isFinite(bottom) || !Number.isFinite(top)) return null;
              return (
                <rect
                  key={JSON.stringify([band.label, band.min, band.max, band.color])}
                  x={0}
                  width={grid.columns}
                  y={Math.min(bottom, top)}
                  height={Math.abs(top - bottom)}
                  fill={band.color}
                  fillOpacity={0.12}
                />
              );
            })}
            {plot.referenceLines.map((line) => {
              if (typeof line.value !== "number") return null;
              const row = ((MARGIN.top + height - y(line.value)) * grid.rows) / height;
              return Number.isFinite(row) ? (
                <line
                  key={JSON.stringify([line.label, line.value, line.color])}
                  x1={0}
                  x2={grid.columns}
                  y1={row}
                  y2={row}
                  stroke={line.color}
                  vectorEffect="non-scaling-stroke"
                />
              ) : null;
            })}
          </g>
        </g>
        {x.ticks(5).map((value) => (
          <text key={value} x={x(value)} y={MARGIN.top + height + 22} textAnchor="middle" fill="currentColor" className="text-xs">
            {formatNumber(value)}
          </text>
        ))}
        {y.ticks(5).map((value) => (
          <text
            key={value}
            x={MARGIN.left - 8}
            y={y(value)}
            dominantBaseline="middle"
            textAnchor="end"
            fill="currentColor"
            className="text-xs"
          >
            {formatNumber(value)}
          </text>
        ))}
      </svg>
    </article>
  );
}

/* === Public Functions === */

export function validContourGrid(grid: ContourGrid): boolean {
  const count = grid.columns * grid.rows;
  return (
    Number.isSafeInteger(count) &&
    grid.columns > 0 &&
    grid.rows > 0 &&
    grid.values.length === count &&
    grid.valid.length === count &&
    grid.x_edges.length === grid.columns + 1 &&
    grid.y_edges.length === grid.rows + 1 &&
    grid.x_edges.every(Number.isFinite) &&
    grid.y_edges.every(Number.isFinite) &&
    grid.values.every((value, index) => !grid.valid[index] || Number.isFinite(value))
  );
}

/* === Private Functions === */
