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
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText, plotUnavailableText } from "@/plots/plot-placeholder";
import type { PlotCardSpec } from "@/plots/types";

/* === Components === */

/**
 * A histogram plot: the backend's bins per series and its statistics of each.
 * Until any series has samples it shows the same placeholder as a time series.
 */
export function HistogramSeriesPlot({ plot }: { plot: PlotCardSpec }) {
  const analysis = plot.analysis?.kind === "histogram" ? plot.analysis : null;
  const series = analysis?.series ?? [];
  const axis = plot.histogramAxes?.x;
  const unit = axis?.unit ?? plot.histogram?.unit ?? plot.unit ?? "";

  if (plot.analysis?.kind === "error") {
    return <PlotPlaceholder title={plot.title} {...plotUnavailableText(plot.analysis.message)} />;
  }
  if (!series.some((item) => item.count > 0)) {
    return <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />;
  }

  const seriesLabel = (key: string) => plot.series.find((item) => item.key === key)?.label ?? key;
  const rows = series.map((item) => ({
    label: series.length === 1 ? plot.title : seriesLabel(item.key),
    unit,
    statistics: item.statistics ?? null,
    count: item.count,
  }));

  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3">
      {series.map((item) => (
        <HistogramChart
          key={item.key}
          title={series.length > 1 ? `${plot.title} / ${seriesLabel(item.key)}` : plot.title}
          distribution={item}
          unit={unit}
          format={axis?.format ?? null}
          xTitle={axis ? histogramAxisTitle(axis) : undefined}
        />
      ))}
      <StatsSummaryTable rows={rows} />
    </article>
  );
}
