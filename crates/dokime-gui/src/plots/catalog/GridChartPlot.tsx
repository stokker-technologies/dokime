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

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { formatNumber, formatUnitLabel } from "@/lib/format";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText } from "@/plots/plot-placeholder";
import { orderedInstants, PlotPlayback, usePlotPlayback } from "@/plots/playback";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

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

  return (
    <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="grid-chart-plot">
      <h4 className="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <Table aria-label={plot.title}>
        <TableHeader>
          <TableRow>
            <TableHead />
            {shown.columns.map((column) => (
              <TableHead key={column.key} className="text-center" title={column.group ?? undefined}>
                {column.group ? `${column.group} · ` : ""}
                {column.label}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {shown.rows.map((row, rowIndex) => (
            <TableRow key={row.key}>
              <TableHead scope="row" title={row.group ?? undefined}>
                {row.group ? `${row.group} · ` : ""}
                {row.label}
              </TableHead>
              {shown.columns.map((column, columnIndex) => {
                const index = rowIndex * shown.columns.length + columnIndex;
                const present = frame.valid[index] && Number.isFinite(frame.values[index]);
                const highlights = frame.highlights.filter((cell) => cell.row === rowIndex && cell.column === columnIndex);
                const value = present ? formatUnitLabel(formatNumber(frame.values[index], plot.yFormat ?? null), plot.unit) : "No Value";
                return (
                  <TableCell
                    key={column.key}
                    data-cell={`${row.key}/${column.key}`}
                    data-valid={present ? "true" : "false"}
                    data-highlight={highlights.length ? "true" : undefined}
                    title={[row.label, column.label, value, ...highlights.map((cell) => cell.label)].join(" · ")}
                    className={`text-center tabular-nums ${present && max > min && (frame.values[index] - min) / (max - min) > 0.72 ? "text-white" : "text-zinc-950"} ${highlights.length ? "ring-2 ring-inset ring-primary" : ""}`}
                    style={present ? { backgroundColor: color(frame.values[index]) } : undefined}
                  >
                    {present ? formatNumber(frame.values[index], plot.yFormat ?? null) : null}
                  </TableCell>
                );
              })}
            </TableRow>
          ))}
        </TableBody>
      </Table>
      {shown.summary_label ? (
        <output className="text-xs text-muted-foreground">
          {formatUnitLabel(shown.summary_label, plot.unit)}:{" "}
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
