/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { DocumentBoxSummaryPayload } from "@/lib/report-types";
import { isNumber } from "@/lib/utils";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

type BoxSummaryPlotProps = {
  summary: DocumentBoxSummaryPayload;
};

type BoxSummaryBucket = Pick<
  DocumentBoxSummaryPayload,
  "minimum" | "q1" | "median" | "q3" | "maximum" | "outliers"
> & {
  key: string;
  label: string;
  sample_count: number;
};

/* === Components === */

export function BoxSummaryPlot({ summary }: BoxSummaryPlotProps) {
  const hasData =
    summary.sample_count > 0 && summary.median !== null && summary.median !== undefined;
  const values = [summary.minimum, summary.q1, summary.median, summary.q3, summary.maximum].filter(
    isNumber,
  );
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = Math.max(max - min, 0.001);

  return (
    <article className="grid min-h-44 gap-3 rounded-md bg-muted/20 p-3">
      <div className="flex items-center justify-between gap-3">
        <strong className="text-sm">{summary.title}</strong>
        <span className="text-xs text-muted-foreground">
          {summary.sample_count ? `${summary.sample_count} samples` : "No samples"}
        </span>
      </div>
      {hasData ? (
        <div className="grid gap-3">
          <div className="relative h-16">
            <span className="absolute left-0 right-0 top-1/2 h-px -translate-y-1/2 bg-border" />
            <span
              className="absolute top-1/2 h-px -translate-y-1/2 bg-foreground"
              style={{
                left: `${position(summary.minimum, min, range)}%`,
                right: `${100 - position(summary.maximum, min, range)}%`,
              }}
            />
            <span
              className="absolute top-1/2 h-9 -translate-y-1/2 rounded border bg-primary/20"
              style={{
                left: `${position(summary.q1, min, range)}%`,
                right: `${100 - position(summary.q3, min, range)}%`,
              }}
            />
            <span
              className="absolute top-1/2 h-11 w-px -translate-y-1/2 bg-primary"
              style={{ left: `${position(summary.median, min, range)}%` }}
            />
          </div>
          <dl className="grid grid-cols-5 gap-2 text-xs">
            <BoxValue label="Min" unit={summary.display_unit} value={summary.minimum} />
            <BoxValue label="Q1" unit={summary.display_unit} value={summary.q1} />
            <BoxValue label="Median" unit={summary.display_unit} value={summary.median} />
            <BoxValue label="Q3" unit={summary.display_unit} value={summary.q3} />
            <BoxValue label="Max" unit={summary.display_unit} value={summary.maximum} />
          </dl>
          <p className="text-xs text-muted-foreground">
            {summary.outliers.length} outlier{summary.outliers.length === 1 ? "" : "s"}
          </p>
        </div>
      ) : (
        <div className="grid min-h-28 place-content-center text-sm text-muted-foreground">
          No numeric samples available for this summary.
        </div>
      )}
    </article>
  );
}

export function BoxSummarySeriesPlot({ plot }: { plot: PlotCardSpec }) {
  const buckets = plotBoxSummaryBuckets(plot);
  const totalSamples = buckets.reduce((total, bucket) => total + bucket.sample_count, 0);
  const values = buckets
    .flatMap((bucket) => [
      bucket.minimum,
      bucket.q1,
      bucket.median,
      bucket.q3,
      bucket.maximum,
      ...bucket.outliers,
    ])
    .filter(isNumber);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = Math.max(max - min, 0.001);

  if (!buckets.length) {
    return (
      <div className="plot-empty grid min-h-44 place-content-center gap-2 rounded-md bg-muted/20 p-4 text-center text-muted-foreground">
        <strong>{plot.title}</strong>
        <span>{plot.stateLabel ?? "Waiting For Samples"}</span>
        <span className="text-xs">
          {plot.stateDetail ?? "No numeric samples are available in the selected window."}
        </span>
      </div>
    );
  }

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-5 rounded-md bg-muted/20 p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold">{plot.title}</h4>
          <p className="text-xs text-muted-foreground">
            {buckets.length} categor{buckets.length === 1 ? "y" : "ies"}, {totalSamples} samples
          </p>
        </div>
        <span className="shrink-0 text-xs text-muted-foreground">{plot.unit ?? "value"}</span>
      </header>
      <div className="grid gap-3 pt-1">
        {buckets.map((bucket) => (
          <div
            className="grid min-h-14 min-w-0 grid-cols-[minmax(6.5rem,10rem)_minmax(10rem,1fr)_max-content] items-center gap-3"
            key={bucket.key}
          >
            <span
              className="min-w-0 truncate text-xs font-semibold tabular-nums"
              title={bucket.label}
            >
              {bucket.label}
            </span>
            <div className="relative h-12 min-w-0 overflow-hidden rounded-md bg-background ring-1 ring-border">
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

function BoxValue({ label, unit, value }: { label: string; unit: string; value?: number | null }) {
  return (
    <div className="grid gap-1">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="font-medium">{formatValue(value, unit)}</dd>
    </div>
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
      sample_count: summary.count,
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
