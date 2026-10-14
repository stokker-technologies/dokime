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
import type { StatsSummary, StatsSummaryRow } from "@/lib/statistics";
import { describeValues, formatStatValue } from "@/lib/statistics";

/* === Types === */

type StatsSummaryTableProps = {
  rows: StatsSummaryRow[];
};

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

export function StatsSummaryTable({ rows }: StatsSummaryTableProps) {
  const described = useMemo(
    () =>
      rows.map((row) => ({
        count: row.count ?? row.values.length,
        label: row.label,
        stats: row.statistics ?? describeValues(row.values),
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

/* === Private Functions === */

function rowLabel(row: DescribedRow): string {
  return row.label;
}

/* === Private Components === */

function StatsSummaryGrid({ row }: { row: StatsSummaryRow }) {
  const stats = row.statistics ?? describeValues(row.values);
  const items = [
    ["Mean", formatStatValue(stats.mean, row.unit)],
    ["Median", formatStatValue(stats.median, row.unit)],
    ["Std", formatStatValue(stats.std, row.unit)],
    ["RMS", formatStatValue(stats.rms, row.unit)],
    ["Min", formatStatValue(stats.min, row.unit)],
    ["Max", formatStatValue(stats.max, row.unit)],
    ["Count", String(row.count ?? row.values.length)],
  ];

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
