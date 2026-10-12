/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { StatsSummaryRow } from "@/lib/statistics";
import { describeValues, formatStatValue } from "@/lib/statistics";

type StatsSummaryTableProps = {
  rows: StatsSummaryRow[];
};

/* === Components === */

export function StatsSummaryTable({ rows }: StatsSummaryTableProps) {
  if (rows.length === 1) {
    return <StatsSummaryGrid row={rows[0]} />;
  }

  return (
    <div className="max-w-full overflow-x-auto rounded-md bg-muted/20">
      <table className="w-full min-w-[760px] table-fixed text-sm">
        <colgroup>
          <col className="w-[24%]" />
          <col className="w-[11%]" />
          <col className="w-[11%]" />
          <col className="w-[11%]" />
          <col className="w-[11%]" />
          <col className="w-[11%]" />
          <col className="w-[11%]" />
          <col className="w-[10%]" />
        </colgroup>
        <thead className="bg-muted/50 text-xs uppercase text-muted-foreground">
          <tr>
            <th className="px-3 py-2 text-left">Metric</th>
            <th className="px-3 py-2 text-right">Mean</th>
            <th className="px-3 py-2 text-right">Median</th>
            <th className="px-3 py-2 text-right">Std</th>
            <th className="px-3 py-2 text-right">RMS</th>
            <th className="px-3 py-2 text-right">Min</th>
            <th className="px-3 py-2 text-right">Max</th>
            <th className="px-3 py-2 text-right">Count</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <StatsSummaryTableRow key={row.label} row={row} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function StatsSummaryTableRow({ row }: { row: StatsSummaryRow }) {
  const stats = row.statistics ?? describeValues(row.values);
  return (
    <tr className="border-t">
      <th className="px-3 py-2 text-left font-medium">{row.label}</th>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.mean, row.unit)}</td>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.median, row.unit)}</td>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.std, row.unit)}</td>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.rms, row.unit)}</td>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.min, row.unit)}</td>
      <td className="px-3 py-2 text-right">{formatStatValue(stats.max, row.unit)}</td>
      <td className="px-3 py-2 text-right">{row.count ?? row.values.length}</td>
    </tr>
  );
}

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
