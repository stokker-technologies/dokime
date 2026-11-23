<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import { failureOf } from "../lib/notification-details";
  import BarPlot from "./catalog/BarPlot.svelte";
  import BoxPlot from "./catalog/BoxPlot.svelte";
  import ContourPlot from "./catalog/ContourPlot.svelte";
  import FrequencySeriesPlot from "./catalog/FrequencySeriesPlot.svelte";
  import GridChartPlot from "./catalog/GridChartPlot.svelte";
  import HeatStripPlot from "./catalog/HeatStripPlot.svelte";
  import HistogramPlot from "./catalog/HistogramPlot.svelte";
  import PiePlot from "./catalog/PiePlot.svelte";
  import TimeSeriesPlot from "./catalog/TimeSeriesPlot.svelte";
  import TrajectoryPlot from "./catalog/TrajectoryPlot.svelte";
  import XyIntervalPlot from "./catalog/XyIntervalPlot.svelte";
  import XyPlot from "./catalog/XyPlot.svelte";
  import { getPlotInteraction } from "./plot-interaction";
  import { plotPlaceholderText, plotUnavailableText, STREAM_ERROR } from "./plot-placeholder";
  import PlotPlaceholder from "./PlotPlaceholder.svelte";
  import type { PlotCardSpec, PlotViewportMode } from "./types";

  let {
    plot,
    resetKey,
    viewportMode,
    actions,
  }: {
    plot: PlotCardSpec;
    resetKey?: string;
    viewportMode: PlotViewportMode;
    actions?: Snippet;
  } = $props();
  const interaction = getPlotInteraction();
</script>

{#if plot.failure && !plot.cartesian && !plot.series.some((series) => series.points.length)}
  <article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-2">
    <div class="flex items-center justify-between gap-3">
      <strong class="min-w-0 truncate" title={plot.title}>{plot.title}</strong>
      {@render actions?.()}
    </div>
    <ErrorCard failure={plot.failure} />
  </article>
{:else if plot.stateLabel === STREAM_ERROR}
  <PlotPlaceholder title={plot.title} {actions} {...plotPlaceholderText(plot)} />
{:else if plot.analysis?.kind === "error"}
  <PlotPlaceholder title={plot.title} {actions} {...plotUnavailableText(plot.analysis.message)} />
{:else if plot.plotType === "heat_strip"}
  {#key resetKey ?? plot.key}
    <HeatStripPlot {plot} />
  {/key}
{:else if plot.plotType === "grid_chart"}
  <GridChartPlot {plot} />
{:else if plot.plotType === "contour"}
  <ContourPlot {plot} setPanelSize={interaction.measurements?.setPanelSize} />
{:else if plot.plotType === "pie"}
  <PiePlot {plot} />
{:else if plot.plotType === "bar"}
  <BarPlot {plot} />
{:else if plot.plotType === "xy" && !plot.xyInterval}
  {#key resetKey ?? plot.key}
    <XyPlot {plot} {resetKey} {viewportMode} {actions} />
  {/key}
{:else if plot.plotType === "xy" && plot.xyInterval === "sigma"}
  <XyIntervalPlot {plot} {resetKey} {viewportMode} {actions} />
{:else if plot.plotType === "trajectory"}
  <TrajectoryPlot {plot} {resetKey} {viewportMode} {actions} />
{:else if plot.plotType === "frequency_series"}
  <FrequencySeriesPlot {plot} {resetKey} {viewportMode} {actions} />
{:else if plot.plotType === "histogram"}
  <HistogramPlot {plot} />
{:else if plot.plotType === "box_plot"}
  <BoxPlot {plot} />
{:else if plot.plotType === "time_series"}
  <TimeSeriesPlot {plot} {resetKey} {viewportMode} {actions} />
{:else}
  <ErrorCard
    failure={failureOf(
      `Unsupported document plot type: ${plot.plotType}`,
      "Presentation Unavailable",
      ["unsupported-plot", plot.key, plot.plotType].join(":"),
    )}
  />
{/if}
