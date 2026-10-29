/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { contours, geoPath, interpolateYlOrRd, scaleLinear, scaleLog, scaleSequential } from "d3";

/* === Local Imports === */

import { formatNumber } from "@/lib/format";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

type HeatmapPoint = {
  x: number;
  y: number;
  z: number;
};

type Domain = {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
};

type SurfaceCell = {
  x: number;
  y: number;
  z: number;
};

/* === Constants === */

const GRID_COLUMNS = 36;
const GRID_ROWS = 28;
const CONTOUR_MIN_POINTS = 4;
const SVG_WIDTH = 720;
const SVG_HEIGHT = 420;
const MARGIN = { top: 72, right: 24, bottom: 52, left: 58 };
const LEGEND_WIDTH = 160;
const LEGEND_HEIGHT = 8;

/* === Components === */

export function HeatmapPlot({ plot }: { plot: PlotCardSpec }) {
  const points = heatmapPoints(plot);
  const domain = heatmapDomain(points);
  if (plot.xScale === "log" && points.length) domain.minX = Math.max(domain.minX, Math.min(...points.map((point) => point.x)) / 2);
  if (plot.yScale === "log" && points.length) domain.minY = Math.max(domain.minY, Math.min(...points.map((point) => point.y)) / 2);
  const zValues = points.map((point) => point.z);
  const minZ = zValues.length ? Math.min(...zValues) : null;
  const maxZ = zValues.length ? Math.max(...zValues) : null;
  const zDomain = normalizedZDomain(minZ, maxZ);
  const plotWidth = SVG_WIDTH - MARGIN.left - MARGIN.right;
  const plotHeight = SVG_HEIGHT - MARGIN.top - MARGIN.bottom;
  const xScale = (plot.xScale === "log" ? scaleLog : scaleLinear)([domain.minX, domain.maxX], [MARGIN.left, MARGIN.left + plotWidth]);
  const yScale = (plot.yScale === "log" ? scaleLog : scaleLinear)([domain.minY, domain.maxY], [MARGIN.top + plotHeight, MARGIN.top]);
  const colorScale = scaleSequential(interpolateYlOrRd).domain(zDomain);
  const surface =
    points.length >= CONTOUR_MIN_POINTS && plot.xScale !== "log" && plot.yScale !== "log" ? griddedSurface(points, domain) : null;
  const contourPath = geoPath();
  const contourShapes = surface
    ? contours().size([GRID_COLUMNS, GRID_ROWS]).thresholds(contourThresholds(zDomain))(surface.map((cell) => cell.z))
    : [];
  const legendId = `heatmap-legend-${plot.key.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
  const clipId = `${legendId}-clip`;
  const referenceCoordinate = (axis: "x" | "y", value: number | null | undefined) => {
    const logarithmic = (axis === "x" ? plot.xScale : plot.yScale) === "log";
    if (typeof value !== "number" || !Number.isFinite(value) || (logarithmic && value <= 0)) {
      return null;
    }

    const coordinate = (axis === "x" ? xScale : yScale)(value);
    return Number.isFinite(coordinate) ? coordinate : null;
  };
  const summary = points.length
    ? `${points.length} points, value ${formatScore(minZ)}-${formatScore(maxZ)}${plot.unit ?? ""}`
    : (plot.stateDetail ?? "No XY samples with numeric Z values are available.");

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3 rounded-lg bg-muted/20 p-3" data-testid="heatmap-plot">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold">{plot.title}</h4>
          {plot.description ? <p className="text-xs text-muted-foreground">{plot.description}</p> : null}
          <p className="text-xs text-muted-foreground">{summary}</p>
        </div>
        <span className="rounded-lg border bg-background px-2 py-1 text-xs font-semibold">Heatmap</span>
      </header>

      {points.length ? (
        <div className="grid min-w-0 max-w-full gap-2 overflow-hidden">
          <svg
            aria-label={`${plot.title}. ${summary}`}
            className="aspect-[12/7] min-h-64 min-w-0 max-w-full rounded-lg bg-background"
            role="img"
            viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
          >
            <defs>
              <clipPath id={clipId}>
                <rect x={MARGIN.left} y={MARGIN.top} width={plotWidth} height={plotHeight} />
              </clipPath>
              <linearGradient id={legendId} x1="0%" x2="100%" y1="0%" y2="0%">
                {Array.from({ length: 9 }, (_, index) => {
                  const t = index / 8;
                  const value = zDomain[0] + (zDomain[1] - zDomain[0]) * t;
                  return <stop key={`legend-${value}`} offset={`${t * 100}%`} stopColor={colorScale(value)} />;
                })}
              </linearGradient>
            </defs>
            <rect
              fill={colorScale(zDomain[0])}
              fillOpacity={0.16}
              height={plotHeight}
              stroke="var(--border)"
              width={plotWidth}
              x={MARGIN.left}
              y={MARGIN.top}
            />
            {xScale.ticks(6).map((tick) => (
              <g key={`x-${tick}`} transform={`translate(${xScale(tick)} 0)`}>
                <line stroke="var(--border)" x1={0} x2={0} y1={MARGIN.top} y2={MARGIN.top + plotHeight} />
                <text fill="var(--muted-foreground)" fontSize={11} textAnchor="middle" x={0} y={MARGIN.top + plotHeight + 20}>
                  {formatNumber(tick, plot.xFormat ?? null)}
                </text>
              </g>
            ))}
            {yScale.ticks(5).map((tick) => (
              <g key={`y-${tick}`} transform={`translate(0 ${yScale(tick)})`}>
                <line stroke="var(--border)" x1={MARGIN.left} x2={MARGIN.left + plotWidth} y1={0} y2={0} />
                <text fill="var(--muted-foreground)" fontSize={11} textAnchor="end" x={MARGIN.left - 10} y={4}>
                  {formatNumber(tick, plot.yFormat ?? null)}
                </text>
              </g>
            ))}
            {contourShapes.length ? (
              <g
                opacity={0.88}
                transform={`translate(${MARGIN.left} ${MARGIN.top}) scale(${plotWidth / GRID_COLUMNS} ${plotHeight / GRID_ROWS})`}
              >
                {contourShapes.map((shape) => (
                  <path d={contourPath(shape) ?? undefined} fill={colorScale(shape.value)} key={shape.value} stroke="none">
                    <title>
                      Value contour {formatScore(shape.value)}
                      {plot.unit ?? ""}
                    </title>
                  </path>
                ))}
              </g>
            ) : (
              <g>
                {points.map((point) => {
                  const size = sparseCellSize(points.length);
                  return (
                    <rect
                      aria-label={`Value ${formatScore(point.z)}${plot.unit ?? ""} At ${formatAxis(point.x)}, ${formatAxis(point.y)}`}
                      fill={colorScale(point.z)}
                      height={size}
                      key={`${point.x}-${point.y}-${point.z}`}
                      opacity={0.92}
                      rx={3}
                      stroke="var(--foreground)"
                      strokeOpacity={0.32}
                      width={size}
                      x={xScale(point.x) - size / 2}
                      y={yScale(point.y) - size / 2}
                    >
                      <title>
                        {formatScore(point.z)}
                        {plot.unit ?? ""} value at X {formatAxis(point.x)}, Y {formatAxis(point.y)}
                      </title>
                    </rect>
                  );
                })}
              </g>
            )}
            {points.map((point) => (
              <circle
                cx={xScale(point.x)}
                cy={yScale(point.y)}
                fill={colorScale(point.z)}
                key={`sample-${point.x}-${point.y}-${point.z}`}
                r={points.length < CONTOUR_MIN_POINTS ? 4 : 2.8}
                stroke="var(--foreground)"
                strokeOpacity={0.42}
                strokeWidth={0.8}
              >
                <title>
                  {formatScore(point.z)}
                  {plot.unit ?? ""} value at X {formatAxis(point.x)}, Y {formatAxis(point.y)}
                </title>
              </circle>
            ))}
            <g clipPath={`url(#${clipId})`} data-testid="heatmap-references">
              {plot.referenceBands.map((band) => {
                const first = referenceCoordinate(band.axis, band.min);
                const second = referenceCoordinate(band.axis, band.max);
                if (first === null || second === null) return null;

                const start = Math.max(Math.min(first, second), band.axis === "x" ? MARGIN.left : MARGIN.top);
                const end = Math.min(Math.max(first, second), band.axis === "x" ? MARGIN.left + plotWidth : MARGIN.top + plotHeight);
                if (end <= start) return null;

                const x = band.axis === "x" ? start : MARGIN.left;
                const y = band.axis === "y" ? start : MARGIN.top;
                const color = band.color || "var(--foreground)";
                const label = band.label ?? `${band.axis.toUpperCase()} reference band`;

                return (
                  <g key={JSON.stringify(band)} data-reference-band={band.axis}>
                    <title>{label}</title>
                    <rect
                      x={x}
                      y={y}
                      width={band.axis === "x" ? end - start : plotWidth}
                      height={band.axis === "y" ? end - start : plotHeight}
                      fill={color}
                      fillOpacity={0.15}
                    />
                    {band.label ? (
                      <text x={x + 4} y={Math.min(y + 14, MARGIN.top + plotHeight - 4)} fill={color} fontSize={11}>
                        {band.label}
                      </text>
                    ) : null}
                  </g>
                );
              })}
              {plot.referenceLines.map((line) => {
                const coordinate = referenceCoordinate(line.axis, line.value);
                if (coordinate === null) return null;

                const minimum = line.axis === "x" ? MARGIN.left : MARGIN.top;
                const maximum = minimum + (line.axis === "x" ? plotWidth : plotHeight);
                if (coordinate < minimum || coordinate > maximum) return null;

                const color = line.color || "var(--foreground)";
                const label =
                  line.label ??
                  `${line.axis.toUpperCase()} = ${formatNumber(line.value as number, (line.axis === "x" ? plot.xFormat : plot.yFormat) ?? null)}`;

                return (
                  <g key={JSON.stringify(line)} data-reference-line={line.axis}>
                    <title>{label}</title>
                    <line
                      x1={line.axis === "x" ? coordinate : MARGIN.left}
                      x2={line.axis === "x" ? coordinate : MARGIN.left + plotWidth}
                      y1={line.axis === "y" ? coordinate : MARGIN.top}
                      y2={line.axis === "y" ? coordinate : MARGIN.top + plotHeight}
                      stroke={color}
                      strokeWidth={1}
                      strokeDasharray="5 3"
                    />
                    {line.label ? (
                      <text
                        x={line.axis === "x" ? Math.min(coordinate + 4, MARGIN.left + plotWidth - 4) : MARGIN.left + 4}
                        y={line.axis === "y" ? Math.max(coordinate - 4, MARGIN.top + 14) : MARGIN.top + 14}
                        fill={color}
                        fontSize={11}
                        textAnchor={line.axis === "x" && coordinate > MARGIN.left + plotWidth / 2 ? "end" : "start"}
                      >
                        {line.label}
                      </text>
                    ) : null}
                  </g>
                );
              })}
            </g>
            <text
              fill="var(--foreground)"
              fontSize={12}
              fontWeight={600}
              textAnchor="middle"
              x={MARGIN.left + plotWidth / 2}
              y={SVG_HEIGHT - 16}
            >
              X
            </text>
            <text
              fill="var(--foreground)"
              fontSize={12}
              fontWeight={600}
              textAnchor="middle"
              transform={`translate(18 ${MARGIN.top + plotHeight / 2}) rotate(-90)`}
            >
              Y
            </text>
            <g transform={`translate(${SVG_WIDTH - MARGIN.right - LEGEND_WIDTH} 18)`}>
              <text fill="var(--muted-foreground)" fontSize={11} x={0} y={0}>
                Value {plot.unit ?? ""}
              </text>
              <rect fill={`url(#${legendId})`} height={LEGEND_HEIGHT} rx={4} width={LEGEND_WIDTH} y={12} />
              <text fill="var(--muted-foreground)" fontSize={10} x={0} y={34}>
                {formatScore(minZ)}
              </text>
              <text fill="var(--muted-foreground)" fontSize={10} textAnchor="end" x={LEGEND_WIDTH} y={34}>
                {formatScore(maxZ)}
              </text>
            </g>
          </svg>
        </div>
      ) : (
        <div className="grid min-h-56 place-content-center rounded-lg bg-background text-center text-sm text-muted-foreground">
          {plot.stateLabel ?? "Waiting For Numeric Samples"}
        </div>
      )}
    </article>
  );
}

/* === Public Functions === */

export function heatmapPoints(plot: PlotCardSpec): HeatmapPoint[] {
  return plot.series.flatMap((series) =>
    series.points
      .map((point) => {
        const x = finiteNumber(point.x);
        const y = finiteNumber(point.y);
        const z = finiteNumber(point.z);
        return x === null || y === null || z === null || (plot.xScale === "log" && x <= 0) || (plot.yScale === "log" && y <= 0)
          ? null
          : { x, y, z };
      })
      .filter((point): point is HeatmapPoint => point !== null),
  );
}

/* === Private Functions === */

function heatmapDomain(points: HeatmapPoint[]): Domain {
  if (!points.length) {
    return { minX: 0, maxX: 1, minY: 0, maxY: 1 };
  }
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  return normalizeDomain({
    minX: Math.min(...xs),
    maxX: Math.max(...xs),
    minY: Math.min(...ys),
    maxY: Math.max(...ys),
  });
}

function normalizeDomain(domain: Domain): Domain {
  const xSpan = domain.maxX - domain.minX;
  const ySpan = domain.maxY - domain.minY;
  return {
    minX: domain.minX - (xSpan === 0 ? 0.5 : xSpan * 0.04),
    maxX: domain.maxX + (xSpan === 0 ? 0.5 : xSpan * 0.04),
    minY: domain.minY - (ySpan === 0 ? 0.5 : ySpan * 0.04),
    maxY: domain.maxY + (ySpan === 0 ? 0.5 : ySpan * 0.04),
  };
}

function normalizedZDomain(minZ: number | null, maxZ: number | null): [number, number] {
  if (minZ === null || maxZ === null) {
    return [0, 1];
  }
  if (minZ === maxZ) {
    return [minZ - 1, maxZ + 1];
  }
  return [minZ, maxZ];
}

function griddedSurface(points: HeatmapPoint[], domain: Domain): SurfaceCell[] {
  return Array.from({ length: GRID_COLUMNS * GRID_ROWS }, (_, index) => {
    const column = index % GRID_COLUMNS;
    const row = Math.floor(index / GRID_COLUMNS);
    const x = domain.minX + (column / (GRID_COLUMNS - 1)) * (domain.maxX - domain.minX);
    const y = domain.maxY - (row / (GRID_ROWS - 1)) * (domain.maxY - domain.minY);
    return {
      x,
      y,
      z: interpolatedZ(points, domain, x, y),
    };
  });
}

function interpolatedZ(points: HeatmapPoint[], domain: Domain, x: number, y: number): number {
  const xSpan = Math.max(domain.maxX - domain.minX, Number.EPSILON);
  const ySpan = Math.max(domain.maxY - domain.minY, Number.EPSILON);
  let numerator = 0;
  let denominator = 0;

  for (const point of points) {
    const dx = (point.x - x) / xSpan;
    const dy = (point.y - y) / ySpan;
    const distanceSq = dx * dx + dy * dy;
    if (distanceSq < 0.000001) {
      return point.z;
    }
    const weight = 1 / distanceSq ** 1.15;
    numerator += point.z * weight;
    denominator += weight;
  }

  return denominator > 0 ? numerator / denominator : 0;
}

function contourThresholds(domain: [number, number]): number[] {
  const [min, max] = domain;
  const span = max - min;
  return Array.from({ length: 8 }, (_, index) => min + (span * (index + 1)) / 8);
}

function sparseCellSize(pointCount: number): number {
  if (pointCount <= 1) {
    return 24;
  }
  if (pointCount <= 2) {
    return 20;
  }
  return 16;
}

function finiteNumber(value: unknown): number | null {
  if (value === null || value === undefined || value === "") return null;
  const number = typeof value === "number" ? value : Number(value);
  return Number.isFinite(number) ? number : null;
}

function formatScore(value: number | null): string {
  return value === null ? "-" : value.toFixed(value >= 10 ? 0 : 1);
}

function formatAxis(value: number): string {
  return Math.abs(value) >= 100 ? value.toFixed(0) : value.toFixed(2);
}
