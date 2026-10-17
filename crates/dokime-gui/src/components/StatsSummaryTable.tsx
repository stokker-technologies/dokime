/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import type {
  SeriesLegendColumn,
  SeriesLegendRow,
  StatsSummary,
  StatsSummaryRow,
} from "@/lib/statistics";
import { formatStatValue, statsSummaryCells } from "@/lib/statistics";
import { cn } from "@/lib/utils";

/* === Types === */

/**
 * A time series or scatter plot's legend: one clickable row per trace that
 * toggles the trace, with the backend's statistics of the shown window.
 */
export type SeriesLegend = {
  columns: SeriesLegendColumn[];
  rows: SeriesLegendRow[];
  /** A new window's statistics are in flight: the old range's numbers dim. */
  pending: boolean;
  onToggle?: (key: string) => void;
};

type StatsSummaryTableProps =
  | { rows: StatsSummaryRow[]; legend?: undefined }
  | { legend: SeriesLegend; rows?: undefined };

type DescribedRow = { count: number; label: string; stats: StatsSummary; unit?: string };

/* === Constants === */

const STATISTICS = [
  ["mean", "Mean"],
  ["median", "Median"],
  ["std", "Std"],
  ["rms", "RMS"],
  ["min", "Min"],
  ["max", "Max"],
] as const;

const column = dataTableColumns<DescribedRow>();

const COLUMNS: DataTableColumns<DescribedRow> = [
  column.accessor("label", {
    header: "Metric",
    meta: { label: "Metric", truncate: true },
  }),
  ...STATISTICS.map(([key, title]) =>
    column.accessor((row) => row.stats[key], {
      cell: ({ row }) => formatStatValue(row.original.stats[key], row.original.unit),
      header: title,
      id: key,
      meta: { align: "end", label: title, truncate: true, width: "w-24" },
    }),
  ),
  column.accessor("count", {
    header: "Count",
    meta: { align: "end", label: "Count", width: "w-20" },
  }),
];

/* === Components === */

export function StatsSummaryTable(props: StatsSummaryTableProps) {
  return props.legend ? (
    <SeriesLegendTable legend={props.legend} />
  ) : (
    <DescribedStatsTable rows={props.rows} />
  );
}

/* === Private Functions === */

function rowLabel(row: DescribedRow): string {
  return row.label;
}

function legendRowKey(row: SeriesLegendRow): string {
  return row.key;
}

function legendColumns(columns: SeriesLegendColumn[]): DataTableColumns<SeriesLegendRow> {
  const legend = dataTableColumns<SeriesLegendRow>();
  return columns.map(({ header, id }) => {
    if (id === "trace") {
      return legend.accessor("label", {
        cell: ({ row }) => <TraceCell row={row.original} />,
        enableSorting: false,
        header,
        id,
        meta: { label: header, primary: true },
      });
    }
    const read = (row: SeriesLegendRow) =>
      id === "xCursor" || id === "yCursor" ? row[id] : row.cells[id];
    return legend.accessor(read, {
      cell: ({ getValue, row }) => (
        <span className={cn("tabular-nums", row.original.hidden && "opacity-50")}>
          {getValue()}
        </span>
      ),
      enableSorting: false,
      header,
      id,
      meta: { align: "end", label: header, truncate: true, width: "w-24" },
    });
  }) as DataTableColumns<SeriesLegendRow>;
}

/* === Private Components === */

function DescribedStatsTable({ rows }: { rows: StatsSummaryRow[] }) {
  const described = useMemo(
    () =>
      rows.map((row) => ({
        count: row.count,
        label: row.label,
        stats: row.statistics ?? {},
        unit: row.unit,
      })),
    [rows],
  );

  if (rows.length === 1) {
    return <StatsSummaryGrid row={rows[0]} />;
  }

  return (
    <DataTable
      className="rounded-md bg-muted/20 [--table-surface:transparent]"
      columns={COLUMNS}
      data={described}
      emptyText="No statistics recorded."
      getRowId={rowLabel}
      inset="cell"
      stickyHeader={false}
    />
  );
}

function SeriesLegendTable({ legend }: { legend: SeriesLegend }) {
  const columns = useMemo(() => legendColumns(legend.columns), [legend.columns]);
  const onToggle = legend.onToggle;
  return (
    <DataTable
      aria-label="Trace statistics"
      className={cn(
        "rounded-md bg-muted/20 [--table-surface:transparent] transition-opacity",
        legend.pending && "opacity-50",
      )}
      columns={columns}
      data={legend.rows}
      emptyText="No traces recorded."
      getRowId={legendRowKey}
      inset="cell"
      onRowClick={onToggle ? (row) => onToggle(row.key) : undefined}
      stickyHeader={false}
    />
  );
}

/** The trace's colour, filled while it draws and outlined while hidden. */
function TraceCell({ row }: { row: SeriesLegendRow }) {
  return (
    <span
      className={cn("flex min-w-0 items-center gap-2", row.hidden && "opacity-50")}
      data-trace-hidden={row.hidden ? "true" : undefined}
    >
      <span
        aria-hidden="true"
        className="size-2.5 shrink-0 rounded-full border-2"
        style={{ backgroundColor: row.hidden ? "transparent" : row.color, borderColor: row.color }}
      />
      <span className="truncate" title={row.label}>
        {row.label}
      </span>
      {row.hidden ? <span className="sr-only">(hidden)</span> : null}
    </span>
  );
}

function StatsSummaryGrid({ row }: { row: StatsSummaryRow }) {
  const items = statsSummaryCells(row);

  return (
    <section
      aria-label={`${row.label} statistics`}
      className="grid min-w-0 max-w-full overflow-hidden gap-2 rounded-md bg-muted/20 p-3"
      data-testid="stats-summary-grid"
    >
      <strong className="text-sm [overflow-wrap:anywhere]">{row.label}</strong>
      <dl className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(min(100%,6.5rem),1fr))]">
        {items.map(([label, value]) => (
          <div className="min-w-0 rounded-md bg-muted/35 p-2" key={label}>
            <dt className="text-[0.68rem] font-bold uppercase text-muted-foreground">{label}</dt>
            <dd className="mt-1 truncate text-sm font-semibold tabular-nums" title={value}>
              {value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
