<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

  import CartesianPlot from "../CartesianPlot.svelte";
  import PlotPlayback from "../PlotPlayback.svelte";
  import { plotPlayback } from "../playback.svelte";
  import type { PlotCardSpec, PlotViewportMode } from "../types";
  import { xyTimeline } from "./xy-playback";

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
  // A revision that keeps its frame, such as a statistics update, neither reads nor cuts it again.
  const cartesian = $derived(plot.cartesian);
  const timeline = $derived(xyTimeline(cartesian));
  const incoming = $derived({ plot, timeline });
  const playback = plotPlayback(
    () => incoming,
    () => timeline.instants,
  );
  // The held series and frame come from one revision, so their columns stay paired.
  const shown = $derived(playback.data.plot);
  const shownTimeline = $derived(playback.data.timeline);
  const frame = $derived(shownTimeline.until(playback.instant));
  const timed = $derived(playback.instants.length > 0);
</script>

<CartesianPlot
  plotId={plot.key}
  badge="XY"
  cartesian={frame}
  colors={plot.colors}
  currentPosition={timed}
  referenceBands={plot.referenceBands}
  referenceLines={plot.referenceLines}
  {actions}
  {resetKey}
  series={shown.series}
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
>
  {#snippet controls()}
    {#if timed}
      <PlotPlayback title={plot.title} {playback} />
    {/if}
  {/snippet}
</CartesianPlot>
