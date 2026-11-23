<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

  import CartesianPlot from "../CartesianPlot.svelte";
  import type { PlotCardSpec, PlotViewportMode } from "../types";

  const badges = { line: undefined, line_points: "Line Points", points: "Points", step: "Step" };

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
</script>

<CartesianPlot
  plotId={plot.key}
  badge={badges[plot.style]}
  cartesian={plot.cartesian}
  colors={plot.colors}
  description={plot.description}
  contextSummary={plot.contextSummary}
  markers={plot.markers}
  promptMarkers={plot.promptMarkers}
  referenceBands={plot.referenceBands}
  referenceLines={plot.referenceLines}
  {actions}
  {resetKey}
  series={plot.series}
  stateDetail={plot.stateDetail}
  stateLabel={plot.stateLabel}
  style={plot.booleanAxis ? "step" : plot.style}
  timeAxis={plot.timeAxis}
  booleanAxis={plot.booleanAxis}
  title={plot.title}
  titleRepeated={plot.titleRepeated}
  unit={plot.unit}
  windowS={plot.windowS}
  {viewportMode}
  xScale={plot.xScale}
  yScale={plot.booleanAxis ? "linear" : plot.yScale}
  xFormat={plot.xFormat}
  xElapsedUnit={plot.xElapsedUnit}
  yFormat={plot.yFormat}
  runStartedAt={plot.runStartedAt}
  cycleStartedAt={plot.cycleStartedAt}
  legend={plot.booleanAxis ? null : { summaries: plot.summaries ?? [], pending: plot.summaryPending ?? false }}
/>
