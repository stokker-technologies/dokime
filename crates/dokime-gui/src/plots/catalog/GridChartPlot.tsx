/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { scaleSequential } from "d3-scale";
import { interpolateYlOrRd } from "d3-scale-chromatic";

/* === Local Imports === */

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { formatNumber, formatUnitLabel } from "@/lib/format";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText } from "@/plots/plot-placeholder";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

/* === Constants === */

/* === Components === */

export function GridChartPlot({ plot }: { plot: PlotCardSpec }) {
  const data = plot.analysis?.kind === "grid_chart" ? plot.analysis.data : null;
  const frame = data?.frames.at(-1);
  if (!data || !frame) return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  const values = frame.values.filter((value, index) => frame.valid[index] && Number.isFinite(value));
  let min = values.length ? Number.POSITIVE_INFINITY : 0;
  let max = values.length ? Number.NEGATIVE_INFINITY : 1;
  for (const value of values) { min = Math.min(min, value); max = Math.max(max, value); }
  const color = scaleSequential(interpolateYlOrRd).domain(min === max ? [min - 1, max + 1] : [min, max]);

  return <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="grid-chart-plot">
    <h4 className="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
    <Table aria-label={plot.title}>
      <TableHeader><TableRow><TableHead />{data.columns.map((column) => <TableHead key={column.key} className="text-center" title={column.group ?? undefined}>{column.group ? `${column.group} · ` : ""}{column.label}</TableHead>)}</TableRow></TableHeader>
      <TableBody>{data.rows.map((row, rowIndex) => <TableRow key={row.key}>
        <TableHead scope="row" title={row.group ?? undefined}>{row.group ? `${row.group} · ` : ""}{row.label}</TableHead>
        {data.columns.map((column, columnIndex) => {
          const index = rowIndex * data.columns.length + columnIndex;
          const present = frame.valid[index] && Number.isFinite(frame.values[index]);
          const highlights = frame.highlights.filter((cell) => cell.row === rowIndex && cell.column === columnIndex);
          const value = present ? formatUnitLabel(formatNumber(frame.values[index]), plot.unit) : "No Value";
          return <TableCell key={column.key} data-cell={`${row.key}/${column.key}`} data-valid={present ? "true" : "false"} data-highlight={highlights.length ? "true" : undefined}
            title={[row.label, column.label, value, ...highlights.map((cell) => cell.label)].join(" · ")}
            className={`text-center tabular-nums ${present && max > min && (frame.values[index] - min) / (max - min) > 0.72 ? "text-white" : "text-zinc-950"} ${highlights.length ? "ring-2 ring-inset ring-primary" : ""}`}
            style={present ? { backgroundColor: color(frame.values[index]) } : undefined}>
            {present ? formatNumber(frame.values[index]) : null}
          </TableCell>;
        })}
      </TableRow>)}</TableBody>
    </Table>
    {data.summary_label ? <output className="text-xs text-muted-foreground">{formatUnitLabel(data.summary_label, plot.unit)}: {formatNumber(values.reduce((sum, value) => sum + value, 0))}</output> : null}
  </article>;
}

/* === Public Functions === */

/* === Private Functions === */
