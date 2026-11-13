/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { scaleLinear } from "d3-scale";

/* === Local Imports === */

import { formatNumber, formatUnitLabel } from "@/lib/format";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText, plotUnavailableText } from "@/plots/plot-placeholder";
import { categoryFractions, categoryTotals } from "@/plots/categories";
import { plotColor } from "@/plots/colors";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

type BarReading = {
  key: string;
  color?: string;
  label: string;
  value: number;
};

/* === Components === */

export function BarPlot({ plot }: { plot: PlotCardSpec }) {
  if (plot.pareto) return <ParetoPlot plot={plot} />;
  if (plot.arrangement) return <ArrangedBarPlot plot={plot} />;
  const bars = latestBars(plot);
  const minimum = Math.min(0, ...bars.map((bar) => bar.value));
  const maximum = Math.max(0, ...bars.map((bar) => bar.value));
  const span = maximum - minimum || 1;
  const baseline = (-minimum / span) * 100;
  const summary = `${bars.length} categor${bars.length === 1 ? "y" : "ies"}.`;

  if (!bars.length) {
    return (
      <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot, "No categorical samples are available in the selected window.")} />
    );
  }

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-4 rounded-lg bg-muted/20 p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold" title={formatUnitLabel(plot.title, plot.unit)}>
            {formatUnitLabel(plot.title, plot.unit)}
          </h4>
          <p className="text-xs text-muted-foreground">{summary}</p>
        </div>
      </header>
      <div className="grid gap-3 pt-1">
        {bars.map((bar) => (
          <div
            className="grid min-h-10 min-w-0 grid-cols-[minmax(--spacing(26),--spacing(40))_minmax(--spacing(32),1fr)_max-content] items-center gap-3"
            key={bar.key}
          >
            <span className="min-w-0 truncate text-xs font-semibold tabular-nums" title={bar.label}>
              {bar.label}
            </span>
            <div className="relative h-8 min-w-0 overflow-hidden rounded-lg bg-background ring-1 ring-border">
              <div
                className="absolute h-full rounded-lg"
                style={{
                  left: `${((Math.min(0, bar.value) - minimum) / span) * 100}%`,
                  width: `${(Math.abs(bar.value) / span) * 100}%`,
                  backgroundColor: bar.color,
                }}
              />
              <span className="absolute inset-y-0 w-px bg-foreground" style={{ left: `${baseline}%` }} />
            </div>
            <span className="whitespace-nowrap text-right text-xs font-semibold tabular-nums">{formatBarValue(bar.value, plot.unit)}</span>
          </div>
        ))}
      </div>
    </article>
  );
}

function ArrangedBarPlot({ plot }: { plot: PlotCardSpec }) {
  const categories = new Map<string, Map<number, number>>();
  plot.series.forEach((series, seriesIndex) => {
    for (const point of series.points) {
      if (typeof point.y !== "number" || !Number.isFinite(point.y)) continue;
      const key = String(point.x);
      if (!categories.has(key)) categories.set(key, new Map());
      categories.get(key)?.set(seriesIndex, point.y);
    }
  });
  if (!categories.size) return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  const stacked = plot.arrangement === "stacked";
  let minimum = 0;
  let maximum = 0;
  for (const values of categories.values()) {
    let negative = 0;
    let positive = 0;
    for (const value of values.values()) {
      if (stacked) {
        negative += Math.min(0, value);
        positive += Math.max(0, value);
      } else {
        minimum = Math.min(minimum, value);
        maximum = Math.max(maximum, value);
      }
    }
    if (stacked) {
      minimum = Math.min(minimum, negative);
      maximum = Math.max(maximum, positive);
    }
  }
  const x = scaleLinear([minimum, maximum === minimum ? minimum + 1 : maximum], [150, 650]);
  const rowHeight = stacked ? 36 : Math.max(36, plot.series.length * 24);
  const height = 44 + categories.size * rowHeight + 32;
  const rows = [...categories].map(([category, values], rowIndex) => {
    let positive = 0;
    let negative = 0;
    const bars = [...values].map(([seriesIndex, value]) => {
      const start = stacked ? (value < 0 ? negative : positive) : 0;
      const end = start + value;
      if (stacked) {
        if (value < 0) negative = end;
        else positive = end;
      }
      return { seriesIndex, value, start, end, top: 20 + rowIndex * rowHeight + (stacked ? 0 : seriesIndex * 24) };
    });
    return { category, rowIndex, bars };
  });
  return (
    <article
      className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3"
      data-testid={`${plot.arrangement}-bar-plot`}
    >
      <h4 className="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <svg role="img" aria-label={plot.title} viewBox={`0 0 720 ${height}`} className="w-full min-w-0">
        {rows.map(({ category, rowIndex, bars }) => (
          <g key={category}>
            <text x={140} y={20 + rowIndex * rowHeight + 14} textAnchor="end" fill="currentColor" className="text-xs">
              {category}
            </text>
            {bars.map((bar) => (
              <rect
                key={bar.seriesIndex}
                data-category={category}
                data-series={plot.series[bar.seriesIndex].key}
                x={Math.min(x(bar.start), x(bar.end))}
                y={bar.top}
                width={Math.abs(x(bar.end) - x(bar.start))}
                height={20}
                fill={plot.colors[bar.seriesIndex] ?? plotColor(bar.seriesIndex)}
              >
                <title>{`${plot.series[bar.seriesIndex].label}: ${formatUnitLabel(formatNumber(bar.value, plot.yFormat ?? null), plot.unit)}`}</title>
              </rect>
            ))}
          </g>
        ))}
        <line x1={x(0)} x2={x(0)} y1={14} y2={height - 30} stroke="currentColor" />
        {x.ticks(5).map((value) => (
          <text key={value} x={x(value)} y={height - 10} textAnchor="middle" fill="currentColor" className="text-xs">
            {formatNumber(value, plot.yFormat ?? null)}
          </text>
        ))}
      </svg>
      <ul className="flex flex-wrap gap-x-4 gap-y-1 text-xs">
        {plot.series.map((series, index) => (
          <li key={series.key} className="flex items-center gap-2">
            <span className="size-2" style={{ backgroundColor: plot.colors[index] ?? plotColor(index) }} />
            {series.label}
          </li>
        ))}
      </ul>
    </article>
  );
}

function ParetoPlot({ plot }: { plot: PlotCardSpec }) {
  if (plot.arrangement || plot.series.length > 1)
    return <PlotPlaceholder title={plot.title} {...plotUnavailableText("Pareto cannot have an arrangement or series binding.")} />;
  const totals = categoryTotals(plot);
  if (totals.entries === undefined) return <PlotPlaceholder title={plot.title} {...plotUnavailableText(totals.error)} />;
  const entries = [...totals.entries].sort((left, right) => right.value - left.value || left.order - right.order);
  if (!entries.length) return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  const fractions = categoryFractions(entries);
  const maximum = entries[0].value;
  const valueAxis = scaleLinear([0, maximum || 1], [280, 20]);
  let cumulative = 0;
  const bars = entries.map((entry, index) => {
    cumulative += fractions[index] * 100;
    return {
      ...entry,
      x: 50 + ((index + 0.5) * 600) / entries.length,
      height: maximum > 0 ? (entry.value / maximum) * 260 : 0,
      cumulative: Math.min(100, cumulative),
    };
  });
  return (
    <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="pareto-plot">
      <h4 className="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <svg role="img" aria-label={plot.title} viewBox="0 0 720 360" className="w-full min-w-0">
        {bars.map((bar) => (
          <g key={bar.key}>
            <rect
              data-category={bar.key}
              x={bar.x - 240 / entries.length}
              y={280 - bar.height}
              width={480 / entries.length}
              height={bar.height}
              fill={plot.colors[0] ?? plotColor(0)}
            >
              <title>{`${bar.key}: ${formatUnitLabel(formatNumber(bar.value, plot.yFormat ?? null), plot.unit)}`}</title>
            </rect>
            <text x={bar.x} y={300} textAnchor="middle" fill="currentColor" className="text-xs">
              {bar.key}
            </text>
            <text x={bar.x} y={Math.max(15, 274 - bar.height)} textAnchor="middle" fill="currentColor" className="text-xs">
              {formatNumber(bar.value, plot.yFormat ?? null)}
            </text>
          </g>
        ))}
        <line x1={50} x2={50} y1={20} y2={280} stroke="currentColor" />
        {valueAxis.ticks(5).map((value) => (
          <text key={value} x={44} y={valueAxis(value)} textAnchor="end" dominantBaseline="middle" fill="currentColor" className="text-xs">
            {formatNumber(value, plot.yFormat ?? null)}
          </text>
        ))}
        {maximum > 0 ? (
          <g>
            <polyline
              data-cumulative
              fill="none"
              stroke={plotColor(1)}
              strokeWidth={2}
              points={bars.map((bar) => `${bar.x},${280 - bar.cumulative * 2.6}`).join(" ")}
            />
            {bars.map((bar) => (
              <circle key={bar.key} data-percent={bar.cumulative} cx={bar.x} cy={280 - bar.cumulative * 2.6} r={3} fill={plotColor(1)} />
            ))}
            {[0, 25, 50, 75, 100].map((percent) => (
              <text key={percent} x={660} y={280 - percent * 2.6} fill="currentColor" dominantBaseline="middle" className="text-xs">
                {percent}%
              </text>
            ))}
          </g>
        ) : null}
      </svg>
    </article>
  );
}

/* === Public Functions === */

export function latestBars(plot: PlotCardSpec): BarReading[] {
  return plot.series
    .flatMap((series, index) => {
      const values = new Map<string, number>();
      series.points.forEach((point) => {
        if (Number.isFinite(point.y)) values.set(String(point.x), Number(point.y));
      });

      return [...values.entries()].map(([category, value]) => ({
        key: JSON.stringify([series.key, category]),
        label: plot.series.length > 1 ? `${series.label} / ${category}` : category,
        value,
        color: plot.colors[index] ?? `var(--chart-${(index % 8) + 1})`,
      }));
    })
    .sort(compareBarLabels);
}

/* === Private Functions === */

function compareBarLabels(left: BarReading, right: BarReading): number {
  const leftNumber = Number(left.label);
  const rightNumber = Number(right.label);
  if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
    return leftNumber - rightNumber;
  }
  return left.label.localeCompare(right.label, undefined, { numeric: true });
}

function formatBarValue(value: number, unit: string | null | undefined): string {
  const formatted = formatNumber(value, { kind: "normal", max_fraction_digits: 2 });
  return unit ? `${formatted} ${unit}` : formatted;
}
