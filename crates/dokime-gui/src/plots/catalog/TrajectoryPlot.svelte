<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  /* === External Imports === */

  import type { Snippet } from "svelte";

  /* === Local Imports === */

  import type { PlotCartesianFrame } from "../../lib/types";
  import CartesianPlot from "../CartesianPlot.svelte";
  import { instantTime, orderedInstants } from "../instants";
  import PlotPlayback from "../PlotPlayback.svelte";
  import { plotPlayback } from "../playback.svelte";
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
  const instants = $derived(
    orderedInstants(plot.series.flatMap((series) => series.points.flatMap((point) => (point.instant ? [point.instant] : [])))),
  );
  const playback = plotPlayback(
    () => plot.series,
    () => instants,
  );
  const time = $derived(playback.instant ? instantTime(playback.instant) : null);
  const series = $derived(
    playback.data.map((series) => ({
      ...series,
      points: series.points
        .filter((point) => {
          const stamp = point.instant ? instantTime(point.instant) : null;
          return time !== null && stamp !== null && stamp <= time;
        })
        .sort((a, b) => {
          const left = instantTime(a.instant ?? "") ?? 0n;
          const right = instantTime(b.instant ?? "") ?? 0n;
          return left < right ? -1 : left > right ? 1 : 0;
        }),
    })),
  );
  const frame: PlotCartesianFrame = $derived({
    mode: "faceted",
    series: series.map((item) => ({ x: item.points.map((point) => Number(point.x)), y: item.points.map((point) => point.y) })),
    point_count: series.reduce((count, item) => count + item.points.length, 0),
    signature: JSON.stringify(series),
  });
</script>

<div class="min-w-0" data-trajectory-instant={playback.instant ?? ""}>
  <CartesianPlot
    plotId={plot.key}
    badge="Trajectory"
    cartesian={frame}
    currentPosition={true}
    localViewport={true}
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
  >
    {#snippet controls()}
      <PlotPlayback title={plot.title} {playback} />
    {/snippet}
  </CartesianPlot>
</div>
