/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type StatsSummaryRow = {
  label: string;
  unit?: string;
  values: number[];
  statistics?: StatsSummary | null;
  count?: number;
};

/** The backend's statistics of one trace over the rows of the shown window. */
export type SeriesSummary = {
  series_id: string;
  count: number;
  min?: number | null;
  max?: number | null;
  mean?: number | null;
  median?: number | null;
  std?: number | null;
  rms?: number | null;
  /** The x range the statistics cover, in the plot's cartesian x space. */
  window?: { from: number; to: number } | null;
};

/** One trace of a time series or scatter plot, as its legend table lists it. */
export type SeriesLegendTrace = {
  key: string;
  label: string;
  color: string;
};

/** A legend table row: the trace, its visibility, and its formatted cells. */
export type SeriesLegendRow = SeriesLegendTrace & {
  hidden: boolean;
  cursor: string;
  cells: Record<SeriesStatistic, string>;
};

export type SeriesStatistic = (typeof SERIES_STATISTICS)[number][0];

export type SeriesLegendColumn = { id: "trace" | "cursor" | SeriesStatistic; header: string };

/** What happened to a plot's summary request on its stream. */
export type SummaryRequestEvent = "requested" | "delivered" | "failed" | "ended";

export type StatsSummary = {
  mean?: number;
  median?: number;
  std?: number;
  rms?: number;
  min?: number;
  max?: number;
};

/* === Constants === */

/** The legend table's statistics, in column order; count alone has no unit. */
export const SERIES_STATISTICS = [
  ["count", "Count"],
  ["min", "Min"],
  ["max", "Max"],
  ["mean", "Mean"],
  ["median", "Median"],
  ["std", "Std"],
  ["rms", "Rms"],
] as const;

/* === Public Functions === */

/**
 * The legend table's columns. The Cursor column reads the pointer over a live
 * plot, so a report omits it; the unit sits in each value column's header.
 */
export function seriesLegendColumns(live: boolean, unit?: string | null): SeriesLegendColumn[] {
  const withUnit = (title: string) => (unit ? `${title} (${unit})` : title);
  return [
    { id: "trace", header: "Trace" },
    ...(live ? [{ id: "cursor" as const, header: withUnit("Cursor") }] : []),
    ...SERIES_STATISTICS.map(([id, title]) => ({
      id,
      header: id === "count" ? title : withUnit(title),
    })),
  ];
}

/**
 * One row per trace, in trace order. Statistics come from the backend summary
 * with the trace's key; a trace without one shows dashes rather than numbers
 * the browser computed from its reduced points.
 */
export function seriesLegendRows({
  cursorValues,
  hidden,
  summaries,
  traces,
}: {
  cursorValues?: readonly (number | null)[] | null;
  hidden: ReadonlySet<string>;
  summaries: readonly SeriesSummary[];
  traces: readonly SeriesLegendTrace[];
}): SeriesLegendRow[] {
  const byKey = new Map(summaries.map((summary) => [summary.series_id, summary]));
  return traces.map((trace, index) => {
    const summary = byKey.get(trace.key);
    const cells = Object.fromEntries(
      SERIES_STATISTICS.map(([id]) => [
        id,
        id === "count"
          ? summary
            ? String(summary.count)
            : "-"
          : formatSummaryValue(summary?.[id]),
      ]),
    ) as Record<SeriesStatistic, string>;
    const cursor = cursorValues?.[index];
    return {
      ...trace,
      hidden: hidden.has(trace.key),
      cursor: cursor === undefined || cursor === null ? "" : formatSummaryValue(cursor),
      cells,
    };
  });
}

/**
 * Whether a plot's summary request is in flight after `event`. Only a new
 * request dims the table: a delivered frame, an error frame, or a stream that
 * ends with its run leaves the shown numbers final.
 */
export function summaryInFlight(event: SummaryRequestEvent): boolean {
  return event === "requested";
}

/** The hidden trace set after a click on one trace's row. */
export function toggleTrace(hidden: ReadonlySet<string>, key: string): Set<string> {
  const next = new Set(hidden);
  if (!next.delete(key)) next.add(key);
  return next;
}

/**
 * The y of the drawn point nearest `at` on x, skipping gaps; `null` when the
 * trace draws no point. A scatter's x need not ascend, so every point is read.
 */
export function nearestDrawnValue(
  x: readonly number[],
  y: readonly (number | null | undefined)[],
  at: number,
): number | null {
  if (!Number.isFinite(at)) return null;
  let best: number | null = null;
  let distance = Number.POSITIVE_INFINITY;
  for (let index = 0; index < x.length; index += 1) {
    const value = y[index];
    const gap = Math.abs(x[index] - at);
    if (value != null && Number.isFinite(value) && gap < distance) {
      best = value;
      distance = gap;
    }
  }
  return best;
}

/** One statistic as the legend table prints it; the unit is in the header. */
export function formatSummaryValue(value: number | null | undefined): string {
  if (value === undefined || value === null || !Number.isFinite(value)) {
    return "-";
  }
  return value.toFixed(Math.abs(value) >= 100 ? 0 : 2);
}

export function describeValues(values: number[]): StatsSummary {
  if (!values.length) {
    return {};
  }
  const ordered = [...values].sort((left, right) => left - right);
  const mean = values.reduce((total, value) => total + value, 0) / values.length;
  const variance = values.reduce((total, value) => total + (value - mean) ** 2, 0) / values.length;
  const middle = Math.floor(ordered.length / 2);
  const median =
    ordered.length % 2
      ? ordered[middle]
      : ((ordered[middle - 1] ?? 0) + (ordered[middle] ?? 0)) / 2;

  return {
    mean,
    median,
    std: Math.sqrt(variance),
    rms: Math.sqrt(values.reduce((total, value) => total + value ** 2, 0) / values.length),
    min: ordered[0],
    max: ordered[ordered.length - 1],
  };
}

export function formatStatValue(value: number | undefined, unit?: string): string {
  const text = formatSummaryValue(value);
  return unit && text !== "-" ? `${text} ${unit}` : text;
}
