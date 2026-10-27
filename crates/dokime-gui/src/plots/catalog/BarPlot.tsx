/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatNumber } from "@/lib/format";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText } from "@/plots/plot-placeholder";
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
  const bars = latestBars(plot);
  const minimum = Math.min(0, ...bars.map((bar) => bar.value));
  const maximum = Math.max(0, ...bars.map((bar) => bar.value));
  const span = maximum - minimum || 1;
  const baseline = (-minimum / span) * 100;
  const summary = `${bars.length} categor${bars.length === 1 ? "y" : "ies"}.`;
  const badge = plot.unit ?? "value";

  if (!bars.length) {
    return (
      <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot, "No categorical samples are available in the selected window.")} />
    );
  }

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-4 rounded-lg bg-muted/20 p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold">{plot.title}</h4>
          <p className="text-xs text-muted-foreground">{summary}</p>
        </div>
        <span className="shrink-0 text-xs text-muted-foreground">{badge}</span>
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
