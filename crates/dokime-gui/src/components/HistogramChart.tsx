/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useState } from "react";

/* === Types === */

type HistogramChartProps = {
  title: string;
  values: number[];
  distribution?: { bins: { lower: number; upper: number; count: number }[]; count: number };
  unit: string;
  bins?: number;
  sampleLabel?: string;
  emptyMessage?: string;
};

type HistogramBin = {
  slot: string;
  min: number;
  max: number;
  count: number;
};

/* === Components === */

export function HistogramChart({
  title,
  values,
  distribution,
  unit,
  bins: binCount = 8,
  sampleLabel = "records",
  emptyMessage = "Waiting for loss/recovery pairs.",
}: HistogramChartProps) {
  const [activeBinIndex, setActiveBinIndex] = useState<number | null>(null);
  const bins = distribution
    ? distribution.bins.map((bin, index) => ({
        slot: String(index),
        min: bin.lower,
        max: bin.upper,
        count: bin.count,
      }))
    : histogramBins(values, binCount);
  const sampleCount = distribution?.count ?? values.length;
  const maxCount = Math.max(1, ...bins.map((bin) => bin.count));
  const activeBin = activeBinIndex === null ? undefined : bins[activeBinIndex];

  return (
    <section className="grid min-h-44 min-w-0 max-w-full overflow-hidden gap-3 rounded-md bg-muted/20 p-3">
      <div className="flex min-w-0 items-center justify-between gap-3">
        <strong className="min-w-0 truncate text-sm">{title}</strong>
        <span className="shrink-0 text-xs text-muted-foreground">
          {sampleCount ? `${sampleCount} ${sampleLabel}` : "No samples"}
        </span>
      </div>
      {sampleCount ? (
        <div className="grid min-w-0 gap-2">
          <div
            className="relative grid h-24 items-end gap-1"
            style={{ gridTemplateColumns: `repeat(${bins.length}, minmax(0, 1fr))` }}
          >
            {activeBinIndex !== null ? (
              <span
                className="pointer-events-none absolute bottom-0 top-0 w-px bg-foreground/60"
                style={{
                  left: `${((activeBinIndex + 0.5) / Math.max(1, bins.length)) * 100}%`,
                }}
              />
            ) : null}
            {bins.map((bin, index) => (
              <button
                aria-label={`${formatHistogramValue(bin.min, unit)} to ${formatHistogramValue(
                  bin.max,
                  unit,
                )}: ${bin.count}`}
                className="grid h-full min-w-0 items-end text-left"
                key={bin.slot}
                type="button"
                onBlur={() => setActiveBinIndex(null)}
                onFocus={() => setActiveBinIndex(index)}
                onMouseEnter={() => setActiveBinIndex(index)}
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
          <div
            className="grid gap-1"
            style={{ gridTemplateColumns: `repeat(${bins.length}, minmax(0, 1fr))` }}
          >
            {bins.map((bin) => (
              <span
                className="block truncate text-center text-[0.65rem] text-muted-foreground"
                key={`${bin.slot}-label`}
                title={formatHistogramValue(bin.min, unit)}
              >
                {formatHistogramValue(bin.min, unit)}
              </span>
            ))}
          </div>
          <span className="min-h-5 text-xs text-muted-foreground">
            {activeBin
              ? `${formatHistogramValue(activeBin.min, unit)} - ${formatHistogramValue(
                  activeBin.max,
                  unit,
                )}: ${activeBin.count}`
              : "Hover a bin for range and count."}
          </span>
        </div>
      ) : (
        <div className="grid min-h-28 place-content-center text-sm text-muted-foreground">
          {emptyMessage}
        </div>
      )}
    </section>
  );
}

/* === Private Functions === */

function histogramBins(values: number[], count: number): HistogramBin[] {
  if (!values.length) {
    return [];
  }
  const min = Math.min(...values);
  const max = Math.max(...values);
  const resolvedCount = Math.max(2, count);
  const width = Math.max((max - min) / resolvedCount, 0.001);
  return Array.from({ length: resolvedCount }, (_, index) => {
    const start = min + width * index;
    const end = index === resolvedCount - 1 ? max : start + width;
    return {
      slot: `bin-${index}`,
      min: start,
      max: end,
      count: values.filter((value) =>
        index === resolvedCount - 1
          ? value >= start && value <= end
          : value >= start && value < end,
      ).length,
    };
  });
}

function formatHistogramValue(value: number | undefined, unit: string): string {
  if (value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  const precision = Math.abs(value) >= 100 ? 0 : 2;
  return `${value.toFixed(precision)} ${unit}`;
}
