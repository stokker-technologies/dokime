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
import { scaleSequential } from "d3-scale";
import { interpolateYlOrRd } from "d3-scale-chromatic";

/* === Local Imports === */

import type { GridAxisEntry } from "@host/model";
import { DataTable } from "../components/data-table/DataTable";
import { dataTableColumns } from "../components/data-table/data-table-model";
import { formatNumber, formatUnitLabel } from "../lib/format";
import type { Wire } from "../lib/rpc/canonical";
import { PlotPlaceholder } from "../PlotPlaceholder";
import { plotPlaceholderText } from "../plot-placeholder";
import { orderedInstants, PlotPlayback, usePlotPlayback } from "../playback";
import type { PlotCardSpec } from "../types";

/* === Types === */

type GridRow = Wire<GridAxisEntry> & { index: number };

/* === Constants === */

/* === Components === */

export function GridChartPlot({ plot }: { plot: PlotCardSpec }) {
  const data = plot.analysis?.kind === "grid_chart" ? plot.analysis.data : null;
  const instants = useMemo(() => orderedInstants(data?.frames.map((frame) => frame.instant) ?? []), [data]);
  const playback = usePlotPlayback(data, instants);
  const frame = playback.data?.frames.find((frame) => frame.instant === playback.instant);
  const shown = playback.data;
  if (!shown || !frame)
    return (
      <div className="grid min-w-0 gap-2">
        <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
        <PlotPlayback title={plot.title} playback={playback} />
      </div>
    );
  const values = frame.values.filter((value, index) => frame.valid[index] && Number.isFinite(value));
  let min = values.length ? Number.POSITIVE_INFINITY : 0;
  let max = values.length ? Number.NEGATIVE_INFINITY : 1;
  for (const value of values) {
    min = Math.min(min, value);
    max = Math.max(max, value);
  }
  const color = scaleSequential(interpolateYlOrRd).domain(min === max ? [min - 1, max + 1] : [min, max]);
  const column = dataTableColumns<GridRow>();
  const columns = column.columns([
    column.accessor("label", {
      header: "Row",
      meta: { filter: "text" },
      cell: ({ row }) => (
        <span title={row.original.group ?? undefined}>
          {row.original.group ? `${row.original.group} · ` : ""}
          {row.original.label}
        </span>
      ),
    }),
    ...shown.columns.map((entry, columnIndex) =>
      column.accessor(
        (row) => {
          const index = row.index * shown.columns.length + columnIndex;
          return frame.valid[index] && Number.isFinite(frame.values[index]) ? frame.values[index] : undefined;
        },
        {
          id: entry.key,
          header: formatUnitLabel(entry.group ? `${entry.group} · ${entry.label}` : entry.label, plot.unit),
          meta: { align: "end" },
          cell: ({ row, getValue }) => {
            const value = getValue();
            const present = value !== undefined;
            const highlights = frame.highlights.filter((cell) => cell.row === row.original.index && cell.column === columnIndex);
            const label = present ? formatUnitLabel(formatNumber(value, plot.yFormat ?? null), plot.unit) : "No Value";
            return (
              <div
                data-cell={`${row.original.key}/${entry.key}`}
                data-valid={present ? "true" : "false"}
                data-highlight={highlights.length ? "true" : undefined}
                title={[row.original.label, entry.label, label, ...highlights.map((cell) => cell.label)].join(" · ")}
                className={`px-2 tabular-nums ${present && max > min && (value - min) / (max - min) > 0.72 ? "text-white" : "text-zinc-950"} ${highlights.length ? "ring-2 ring-inset ring-primary" : ""}`}
                style={present ? { backgroundColor: color(value) } : undefined}
              >
                {present ? formatNumber(value, plot.yFormat ?? null) : null}
              </div>
            );
          },
        },
      ),
    ),
  ]);
  const rows = shown.rows.map((row, index) => ({ ...row, index }));

  return (
    <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="grid-chart-plot">
      <DataTable
        aria-label={plot.title}
        title={plot.title}
        description="Values and highlighted cells at the selected instant."
        columns={columns}
        data={rows}
        getRowId={(row) => row.key}
        emptyText="No Grid Rows Recorded"
      />
      {shown.summaryLabel ? (
        <output className="text-xs text-muted-foreground">
          {formatUnitLabel(shown.summaryLabel, plot.unit)}:{" "}
          {formatNumber(
            values.reduce((sum, value) => sum + value, 0),
            plot.yFormat ?? null,
          )}
        </output>
      ) : null}
      <PlotPlayback title={plot.title} playback={playback} />
    </article>
  );
}

/* === Public Functions === */

/* === Private Functions === */
