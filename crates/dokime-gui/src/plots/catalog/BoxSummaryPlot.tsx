/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatUnitLabel } from "@/lib/format";

import { isNumber } from "@/lib/utils";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText, plotUnavailableText } from "@/plots/plot-placeholder";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

/** One box as the backend summarized it, whiskers as its drawn extent. */
type BoxSummaryBucket = {
  key: string;
  label: string;
  minimum: number;
  q1: number;
  median: number;
  q3: number;
  maximum: number;
  outliers: number[];
};

/* === Components === */

export function BoxSummarySeriesPlot({ plot }: { plot: PlotCardSpec }) {
  const buckets = plotBoxSummaryBuckets(plot);
  const totalSamples = plot.analysis?.kind === "box_summary" ? plot.analysis.total : 0;
  const values = buckets
    .flatMap((bucket) => [bucket.minimum, bucket.q1, bucket.median, bucket.q3, bucket.maximum, ...bucket.outliers])
    .filter(isNumber);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = Math.max(max - min, 0.001);

  if (plot.analysis?.kind === "error") {
    return <PlotPlaceholder title={plot.title} {...plotUnavailableText(plot.analysis.message)} />;
  }
  if (!buckets.length) {
    return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  }

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-4 rounded-lg bg-muted/20 p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold" title={formatUnitLabel(plot.title, plot.unit)}>
            {formatUnitLabel(plot.title, plot.unit)}
          </h4>
          <p className="text-xs text-muted-foreground">
            {buckets.length} categor{buckets.length === 1 ? "y" : "ies"}, {totalSamples} samples
          </p>
        </div>
      </header>
      <div className="grid gap-3 pt-1">
        {buckets.map((bucket) => (
          <div
            className="grid min-h-14 min-w-0 grid-cols-[minmax(--spacing(26),--spacing(40))_minmax(--spacing(40),1fr)_max-content] items-center gap-3"
            key={bucket.key}
          >
            <span className="min-w-0 truncate text-xs font-semibold tabular-nums" title={bucket.label}>
              {bucket.label}
            </span>
            <div className="relative h-12 min-w-0 overflow-hidden rounded-lg bg-background ring-1 ring-border">
              {[...new Set(bucket.outliers)].map((value) => (
                <span
                  key={value}
                  className="absolute top-1/2 h-1 w-1 rounded-full bg-foreground"
                  style={{ left: `${position(value, min, range)}%` }}
                  title={`Outlier: ${formatValue(value, plot.unit ?? "")}`}
                />
              ))}
              <span className="absolute left-2 right-2 top-1/2 h-px -translate-y-1/2 bg-border" />
              <span
                className="absolute top-1/2 h-px -translate-y-1/2 bg-foreground"
                style={{
                  left: `${position(bucket.minimum, min, range)}%`,
                  right: `${100 - position(bucket.maximum, min, range)}%`,
                }}
              />
              <span
                className="absolute top-1/2 h-7 -translate-y-1/2 rounded border border-primary/50 bg-primary/20"
                style={{
                  left: `${position(bucket.q1, min, range)}%`,
                  right: `${100 - position(bucket.q3, min, range)}%`,
                }}
              />
              <span
                className="absolute top-1/2 h-9 w-px -translate-y-1/2 bg-primary"
                style={{ left: `${position(bucket.median, min, range)}%` }}
              />
            </div>
            <span className="whitespace-nowrap text-right text-xs font-semibold tabular-nums">
              {formatValue(bucket.median, plot.unit ?? "value")}
            </span>
          </div>
        ))}
      </div>
    </article>
  );
}

/* === Private Functions === */

function position(value: number | null | undefined, min: number, range: number): number {
  return value === null || value === undefined ? 0 : ((value - min) / range) * 100;
}

function formatValue(value: number | null | undefined, unit: string): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  const precision = Math.abs(value) >= 100 ? 0 : 2;
  return `${value.toFixed(precision)} ${unit}`;
}

function plotBoxSummaryBuckets(plot: PlotCardSpec): BoxSummaryBucket[] {
  if (plot.analysis?.kind !== "box_summary") return [];

  return plot.analysis.series
    .map((summary) => ({
      key: summary.key,
      label: summary.label,
      minimum: summary.lower_whisker,
      maximum: summary.upper_whisker,
      q1: summary.q1,
      median: summary.median,
      q3: summary.q3,
      outliers: summary.outliers,
    }))
    .sort(compareBuckets);
}

function compareBuckets(left: BoxSummaryBucket, right: BoxSummaryBucket): number {
  const leftNumber = Number(left.label);
  const rightNumber = Number(right.label);
  if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
    return leftNumber - rightNumber;
  }
  return left.label.localeCompare(right.label, undefined, { numeric: true });
}
