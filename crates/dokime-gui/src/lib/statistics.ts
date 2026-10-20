/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

/** One labelled distribution's backend statistics; `null` until they arrive. */
export type StatsSummaryRow = {
  label: string;
  unit?: string;
  statistics: StatsSummary | null;
  count: number;
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
  xCursor: string;
  yCursor: string;
  cells: Record<SeriesStatistic, string>;
};

export type SeriesStatistic = (typeof SERIES_STATISTICS)[number][0];

export type SeriesLegendColumn = { id: "trace" | "xCursor" | "yCursor" | SeriesStatistic; header: string };

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
 * The legend table's columns, the same on a live plot and in a report: the
 * cursor columns read the pointer over either; the unit sits in each value
 * column's header.
 */
export function seriesLegendColumns(unit?: string | null): SeriesLegendColumn[] {
  const withUnit = (title: string) => (unit ? `${title} (${unit})` : title);
  return [
    { id: "trace", header: "Trace" },
    { id: "xCursor", header: "X Cursor" },
    { id: "yCursor", header: withUnit("Y Cursor") },
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
  cursorPoints,
  formatX = formatSummaryValue,
  hidden,
  summaries,
  traces,
}: {
  cursorPoints?: readonly ({ x: number; y: number } | null)[] | null;
  /** Prints the x cursor as the plot's x axis does; a plain number otherwise. */
  formatX?: (x: number) => string;
  hidden: ReadonlySet<string>;
  summaries: readonly SeriesSummary[];
  traces: readonly SeriesLegendTrace[];
}): SeriesLegendRow[] {
  const byKey = new Map(summaries.map((summary) => [summary.series_id, summary]));
  return traces.map((trace, index) => {
    const summary = byKey.get(trace.key);
    const cells = Object.fromEntries(
      SERIES_STATISTICS.map(([id]) => [id, id === "count" ? (summary ? String(summary.count) : "-") : formatSummaryValue(summary?.[id])]),
    ) as Record<SeriesStatistic, string>;
    const cursor = cursorPoints?.[index];
    return {
      ...trace,
      hidden: hidden.has(trace.key),
      xCursor: cursor ? formatX(cursor.x) : "",
      yCursor: cursor ? formatSummaryValue(cursor.y) : "",
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
 * The drawn point nearest `at` on x, skipping gaps; `null` when the
 * trace draws no point. A scatter's x need not ascend, so every point is read.
 */
export function nearestDrawnPoint(
  x: readonly number[],
  y: readonly (number | null | undefined)[],
  at: number,
): { x: number; y: number } | null {
  if (!Number.isFinite(at)) return null;
  let best: { x: number; y: number } | null = null;
  let distance = Number.POSITIVE_INFINITY;
  for (let index = 0; index < x.length; index += 1) {
    const value = y[index];
    const gap = Math.abs(x[index] - at);
    if (value != null && Number.isFinite(value) && gap < distance) {
      best = { x: x[index], y: value };
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

export function formatStatValue(value: number | undefined, unit?: string): string {
  const text = formatSummaryValue(value);
  return unit && text !== "-" ? `${text} ${unit}` : text;
}

/**
 * A distribution's statistics as the summary grid prints them, in order. The
 * values are the backend's; a statistic it did not deliver prints a dash.
 */
export function statsSummaryCells(row: StatsSummaryRow): [string, string][] {
  const stats = row.statistics ?? {};
  return [
    ["Mean", formatStatValue(stats.mean, row.unit)],
    ["Median", formatStatValue(stats.median, row.unit)],
    ["Std", formatStatValue(stats.std, row.unit)],
    ["RMS", formatStatValue(stats.rms, row.unit)],
    ["Min", formatStatValue(stats.min, row.unit)],
    ["Max", formatStatValue(stats.max, row.unit)],
    ["Count", String(row.count)],
  ];
}
