/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type ReactNode, useState } from "react";

/* === Local Imports === */

import { formatNumber } from "@/lib/format";
import type { ValueFormat } from "@/lib/types";
import { HISTOGRAM_COUNT_LABEL } from "@/plots/histogram-axes";
import { type HistogramDistribution, histogramBars } from "@/plots/histogram-bins";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText } from "@/plots/plot-placeholder";

/* === Types === */

type HistogramChartProps = {
  title: string;
  /** The backend's bins of this series; the browser does not bin samples. */
  distribution: HistogramDistribution;
  unit: string;
  /** The x axis title: the binned value's label and unit. */
  xTitle?: string;
  /** How a bin edge prints; `null` keeps the plain number. */
  format?: ValueFormat | null;
  sampleLabel?: string;
  /** Content the card holds under the chart, such as its statistics, under the one title. */
  children?: ReactNode;
};

/* === Components === */

export function HistogramChart({
  children,
  title,
  distribution,
  unit,
  xTitle,
  format = null,
  sampleLabel = "samples",
}: HistogramChartProps) {
  const [activeBinIndex, setActiveBinIndex] = useState<number | null>(null);

  if (!distribution.count) {
    return <PlotPlaceholder title={title} {...plotPlaceholderText({})} />;
  }

  const bins = histogramBars(distribution);
  const maxCount = Math.max(1, ...bins.map((bin) => bin.count));
  const activeBin = activeBinIndex === null ? undefined : bins[activeBinIndex];

  return (
    <section className="grid min-h-44 min-w-0 max-w-full overflow-hidden gap-3 rounded-lg bg-muted/20 p-3">
      <div className="flex min-w-0 items-center justify-between gap-3">
        <strong className="min-w-0 truncate text-sm">{title}</strong>
        <span className="shrink-0 text-xs text-muted-foreground">
          {distribution.count} {sampleLabel}
        </span>
      </div>
      <div className="grid min-w-0 gap-2">
        <span className="text-xs text-muted-foreground">{HISTOGRAM_COUNT_LABEL}</span>
        <div className="relative grid h-24 items-end gap-1" style={{ gridTemplateColumns: `repeat(${bins.length}, minmax(0, 1fr))` }}>
          {activeBinIndex !== null && activeBin ? (
            <>
              <span
                className="pointer-events-none absolute bottom-0 top-0 w-px bg-foreground/60"
                style={{ left: `${binCentre(activeBinIndex, bins.length)}%` }}
              />
              <span
                className="pointer-events-none absolute top-0 z-10 max-w-full -translate-x-1/2 truncate rounded-lg border bg-popover px-2 py-1 text-xs whitespace-nowrap text-popover-foreground shadow-sm"
                role="tooltip"
                style={{ left: `${Math.min(85, Math.max(15, binCentre(activeBinIndex, bins.length)))}%` }}
              >
                {`${formatHistogramValue(activeBin.min, unit, format)} - ${formatHistogramValue(activeBin.max, unit, format)}: ${activeBin.count}`}
              </span>
            </>
          ) : null}
          {bins.map((bin, index) => (
            <button
              aria-label={`${formatHistogramValue(bin.min, unit, format)} To ${formatHistogramValue(bin.max, unit, format)}: ${bin.count}`}
              className="grid h-full min-w-0 items-end text-left"
              key={bin.slot}
              type="button"
              onBlur={() => setActiveBinIndex(null)}
              onFocus={() => setActiveBinIndex(index)}
              onMouseEnter={() => setActiveBinIndex(index)}
              onMouseLeave={() => setActiveBinIndex(null)}
            >
              <span
                className="block w-full rounded-t bg-primary transition-colors hover:bg-primary/85"
                style={{
                  height: bin.count > 0 ? `${(bin.count / maxCount) * 96}px` : 0,
                }}
              />
            </button>
          ))}
        </div>
        <div className="grid gap-1" style={{ gridTemplateColumns: `repeat(${bins.length}, minmax(0, 1fr))` }}>
          {bins.map((bin) => (
            <span
              className="block truncate text-center text-xs text-muted-foreground"
              key={`${bin.slot}-label`}
              title={formatHistogramValue(bin.min, unit, format)}
            >
              {formatHistogramValue(bin.min, unit, format)}
            </span>
          ))}
        </div>
        {xTitle ? <span className="truncate text-center text-xs text-muted-foreground">{xTitle}</span> : null}
      </div>
      {children}
    </section>
  );
}

/* === Private Functions === */

/** Where bin `index` of `count` centres along the chart, in percent. */
function binCentre(index: number, count: number): number {
  return ((index + 0.5) / Math.max(1, count)) * 100;
}

function formatHistogramValue(value: number | undefined, unit: string, format: ValueFormat | null): string {
  if (value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  const text = format ? formatNumber(value, format) : value.toFixed(Math.abs(value) >= 100 ? 0 : 2);
  return unit ? `${text} ${unit}` : text;
}
