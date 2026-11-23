<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

  import { formatUnitLabel } from "../../lib/format";

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

  /* === Private Functions === */

  function frequencyBadge(plot: PlotCardSpec): string {
    const unit = plot.fft.level_unit === "linear" ? "FFT · Hz" : `FFT · Hz · ${plot.fft.level_unit}`;
    return plot.xScale === "log" || plot.yScale === "log" ? `${unit} Log` : unit;
  }
</script>

<CartesianPlot
  badge={frequencyBadge(plot)}
  cartesian={plot.cartesian}
  colors={plot.colors}
  referenceBands={plot.referenceBands}
  referenceLines={plot.referenceLines}
  {actions}
  {resetKey}
  series={plot.series}
  stateDetail={plot.stateDetail}
  stateLabel={plot.stateLabel}
  style="line"
  timeAxis={false}
  title={plot.title}
  unit={plot.fft.level_unit === "linear" ? plot.unit : plot.fft.level_unit}
  windowS={plot.windowS}
  {viewportMode}
  xScale={plot.xScale}
  yScale={plot.yScale}
  xLabel={formatUnitLabel("Frequency", "Hz")}
  xFormat={{ kind: { kind: { case: "normal", value: {} } } }}
  yFormat={{ kind: { kind: { case: "normal", value: {} } } }}
/>
