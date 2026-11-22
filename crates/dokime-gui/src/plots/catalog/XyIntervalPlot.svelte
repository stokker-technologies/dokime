<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

  import type { PlotCartesianFrame, PlotSeries } from "../../lib/types";
  import CartesianPlot from "../CartesianPlot.svelte";
  import type { PlotCardSpec, PlotViewportMode } from "../types";

  let {
    actions,
    plot,
    resetKey,
    viewportMode = "fit",
  }: {
    actions?: Snippet;
    plot: PlotCardSpec;
    resetKey?: string;
    viewportMode?: PlotViewportMode;
  } = $props();
  const series = $derived(intervalSeries(plot));
  const frame: PlotCartesianFrame = $derived({
    mode: "faceted",
    series: series.map((item) => ({
      x: item.points.map((point) => Number(point.x)),
      y: item.points.map((point) => point.y),
    })),
    point_count: series.reduce((count, item) => count + item.points.length, 0),
    signature: JSON.stringify(plot.analysis),
  });

  /* === Public Functions === */

  function intervalSeries(plot: PlotCardSpec): PlotSeries[] {
    if (plot.analysis?.kind !== "sigma_interval") {
      return [];
    }

    return plot.analysis.series.map((summary) => ({
      key: summary.key,
      label: plot.series.find((series) => series.key === summary.key)?.label ?? summary.key,
      raw_points: summary.raw_points,
      downsampled: false,
      points: summary.bins
        .filter(
          (bin) =>
            Number.isFinite(bin.x) &&
            Number.isFinite(bin.mean) &&
            (plot.xScale !== "log" || bin.x > 0) &&
            (plot.yScale !== "log" || bin.mean > 0),
        )
        .map((bin) => ({
          x: bin.x,
          y: bin.mean,
          low: bin.lower,
          high: bin.upper,
          sample_count: bin.count,
        })),
    }));
  }
</script>

<CartesianPlot
  plotId={plot.key}
  badge="XY Interval"
  intervals={true}
  cartesian={frame}
  colors={plot.colors}
  referenceBands={plot.referenceBands}
  referenceLines={plot.referenceLines}
  {actions}
  {resetKey}
  {series}
  stateDetail={plot.stateDetail}
  stateLabel={plot.stateLabel}
  style="line_points"
  timeAxis={false}
  title={plot.title}
  unit={plot.unit}
  windowS={plot.windowS}
  {viewportMode}
  xScale={plot.xScale}
  yScale={plot.yScale}
  xFormat={plot.xFormat}
  yFormat={plot.yFormat}
/>
