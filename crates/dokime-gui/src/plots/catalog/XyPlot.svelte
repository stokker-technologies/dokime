<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

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
</script>

<CartesianPlot
  plotId={plot.key}
  badge="XY"
  cartesian={plot.cartesian}
  colors={plot.colors}
  referenceBands={plot.referenceBands}
  referenceLines={plot.referenceLines}
  {actions}
  {resetKey}
  series={plot.series}
  stateDetail={plot.stateDetail}
  stateLabel={plot.stateLabel}
  style={plot.style}
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
