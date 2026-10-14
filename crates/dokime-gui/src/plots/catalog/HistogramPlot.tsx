/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { HistogramChart } from "@/components/HistogramChart";
import { StatsSummaryTable } from "@/components/StatsSummaryTable";
import { histogramAxisTitle } from "@/plots/histogram-axes";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

type HistogramPlotProps = {
  bins: number;
  title: string;
  unit: string;
  values: number[];
  /** The x axis title: the binned value's label and unit. */
  xTitle?: string;
  emptyMessage?: string;
};

/* === Components === */

export function HistogramPlot({
  bins,
  title,
  unit,
  values,
  xTitle,
  emptyMessage = "No numeric samples available for this histogram.",
}: HistogramPlotProps) {
  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3">
      <HistogramChart
        bins={bins}
        emptyMessage={emptyMessage}
        sampleLabel="samples"
        title={title}
        unit={unit}
        values={values}
        xTitle={xTitle}
      />
      <StatsSummaryTable rows={[{ label: title, unit, values }]} />
    </article>
  );
}

export function HistogramSeriesPlot({ plot }: { plot: PlotCardSpec }) {
  const analysis = plot.analysis?.kind === "histogram" ? plot.analysis : null;
  const axis = plot.histogramAxes?.x;
  const unit = axis?.unit ?? plot.histogram?.unit ?? plot.unit ?? "";

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3">
      {analysis?.series.length ? (
        analysis.series.map((series) => (
          <HistogramChart
            key={series.key}
            title={
              analysis.series.length > 1
                ? `${plot.title} / ${plot.series.find((item) => item.key === series.key)?.label ?? series.key}`
                : plot.title
            }
            values={[]}
            distribution={series}
            sampleLabel="samples"
            unit={unit}
            format={axis?.format ?? null}
            xTitle={axis ? histogramAxisTitle(axis) : undefined}
          />
        ))
      ) : (
        <p>{plot.stateDetail ?? "No histogram analysis is available."}</p>
      )}
      {analysis?.series.length ? (
        <StatsSummaryTable
          rows={analysis.series.map((series) => ({
            label:
              analysis.series.length === 1
                ? plot.title
                : (plot.series.find((item) => item.key === series.key)?.label ?? series.key),
            unit,
            values: [],
            statistics: series.statistics,
            count: series.count,
          }))}
        />
      ) : null}
    </article>
  );
}
