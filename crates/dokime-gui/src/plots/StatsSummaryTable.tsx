/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { AbbreviatedValue } from "../components/AbbreviatedValue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import type { SeriesLegendColumn, SeriesLegendRow, StatsSummaryRow } from "../lib/statistics";
import { seriesLegendColumns, seriesLegendRows } from "../lib/statistics";
import { cn } from "../lib/utils";

/* === Types === */

/**
 * One clickable row per trace or marker kind, with counts of the shown window
 * and the backend's trace statistics.
 */
export type SeriesLegend = {
  columns: SeriesLegendColumn[];
  rows: SeriesLegendRow[];
  /** A new window's statistics are in flight: the old range's numbers dim. */
  pending: boolean;
  onToggle?: (key: string) => void;
};

type StatsSummaryTableProps =
  | {
      rows: StatsSummaryRow[];
      legend?: undefined;
    }
  | { legend: SeriesLegend; rows?: undefined };

/* === Components === */

export function StatsSummaryTable(props: StatsSummaryTableProps) {
  const legend: SeriesLegend = props.legend ?? {
    columns: [{ id: "trace", header: "Metric" }, ...seriesLegendColumns(props.rows[0]?.unit).slice(3)],
    rows: seriesLegendRows({
      hidden: new Set(),
      summaries: props.rows.map((row, index) => ({ ...row.statistics, series_id: String(index), count: row.count })),
      traces: props.rows.map((row, index) => ({ key: String(index), label: row.label, color: "" })),
    }),
    pending: false,
  };
  const onToggle = legend.onToggle;
  return (
    <Table
      aria-label={props.legend ? "Trace Statistics" : "Histogram Statistics"}
      className={cn("transition-opacity", legend.pending && "opacity-50")}
    >
      <TableHeader>
        <TableRow>
          {legend.columns.map(({ header, id }) => (
            <TableHead className={id === "trace" ? undefined : "text-right"} key={id} title={header}>
              {header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {legend.rows.map((row) => (
          <TableRow
            className={onToggle ? "cursor-pointer" : undefined}
            data-row-id={row.key}
            key={row.key}
            onClick={onToggle ? () => onToggle(row.key) : undefined}
          >
            {legend.columns.map(({ id }) => (
              <TableCell className={id === "trace" ? undefined : "text-right tabular-nums"} key={id}>
                {id === "trace" ? (
                  onToggle ? (
                    <button type="button" aria-pressed={!row.hidden} className="block w-full text-left">
                      <TraceCell row={row} />
                    </button>
                  ) : props.legend ? (
                    <TraceCell row={row} />
                  ) : (
                    row.label
                  )
                ) : (
                  <AbbreviatedValue
                    className={cn(row.hidden && "opacity-50")}
                    exact={id === "xCursor" || id === "yCursor" ? row[id] : row.cells[id]}
                  />
                )}
              </TableCell>
            ))}
          </TableRow>
        ))}
        {legend.rows.length === 0 ? (
          <TableRow>
            <TableCell colSpan={legend.columns.length}>{props.legend ? "No Traces Recorded" : "No Statistics Recorded"}</TableCell>
          </TableRow>
        ) : null}
      </TableBody>
    </Table>
  );
}

/* === Private Components === */

/** The trace's colour, filled while it draws and outlined while hidden. */
function TraceCell({ row }: { row: SeriesLegendRow }) {
  return (
    <span className={cn("flex min-w-0 items-center gap-2", row.hidden && "opacity-50")} data-trace-hidden={row.hidden ? "true" : undefined}>
      <span
        aria-hidden="true"
        className="size-2.5 shrink-0 rounded-full border-2"
        style={{ backgroundColor: row.hidden ? "transparent" : row.color, borderColor: row.color }}
      />
      <span className="truncate" title={row.label}>
        {row.label}
      </span>
      {row.hidden ? <span className="sr-only">(Hidden)</span> : null}
    </span>
  );
}
