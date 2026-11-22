<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatUnitLabel } from "../../lib/format";
  import { cutKey } from "../../lib/rpc/canonical";
  import { sequentialColorScale } from "../colors";
  import { orderedInstants } from "../instants";
  import { plotPlaceholderText } from "../plot-placeholder";
  import { plotPlayback } from "../playback.svelte";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import PlotPlayback from "../PlotPlayback.svelte";
  import type { PlotCardSpec } from "../types";
  import { heatStripSample } from "./heat-strip";

  let { plot }: { plot: PlotCardSpec } = $props();

  const instants = $derived(
    orderedInstants(plot.series.flatMap((series) => series.points.flatMap((point) => (point.instant ? [point.instant] : [])))),
  );
  const playback = plotPlayback(
    () => plot,
    () => instants,
  );
  const facets = $derived(heatStripSample(playback.data.series, playback.instant));
  const { min, max, present, color } = $derived(
    sequentialColorScale(playback.data.series.flatMap((series) => series.points.map((point) => point.y))),
  );
  const hasPoints = $derived(playback.data.series.some((series) => series.points.length));
  const gradient = $derived(Array.from({ length: 11 }, (_, index) => color(min + ((max - min) * index) / 10)).join(", "));
</script>

{#if hasPoints}
  <article class="plot-card grid min-w-0 gap-3 rounded-lg bg-muted/20 p-3" data-testid="heat-strip-plot">
    {#if !plot.titleRepeated}
      <strong>{formatUnitLabel(plot.title, plot.unit)}</strong>
    {/if}
    <div class="grid min-w-0 gap-2">
      {#each facets as facet (facet.key)}
        <div class="grid gap-1" data-facet={facet.key}>
          {#if facet.label}
            <span class="text-sm">{facet.label}</span>
          {/if}
          <div
            class="grid w-full min-w-0 gap-1"
            style:grid-template-columns={facet.points.length > 16
              ? "repeat(auto-fit, minmax(min(100%, 2.5rem), 1fr))"
              : `repeat(${facet.points.length}, minmax(0, 1fr))`}
          >
            {#each facet.points as point (cutKey(point.x))}
              {@const value = point.y !== null && Number.isFinite(point.y) ? point.y : null}
              {@const label = String(point.x ?? "")}
              {@const title = [label, facet.label, value === null ? "No Value" : String(point.exact_y ?? value), point.instant]
                .filter(Boolean)
                .join(" · ")}
              {@const bright = value !== null && max > min && (value - min) / (max - min) > 0.72}
              <div
                class="grid min-w-0 place-items-center rounded border px-1 break-all text-center text-xs tabular-nums"
                data-cell={label}
                data-valid={String(value !== null)}
                {title}
                style:min-height="3rem"
                style:background-color={value === null ? undefined : color(value)}
                style:color={value === null ? undefined : bright ? "#fff" : "#18181b"}
              >
                {label}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
    {#if present}
      <div class="flex items-center gap-2 text-xs tabular-nums" data-heat-legend="true" data-min={min} data-max={max}>
        <span>{String(min)}</span>
        <span class="h-3 w-32 rounded" style:background={`linear-gradient(to right, ${gradient})`}></span>
        <span>{String(max)}</span>
        {#if plot.unit}
          <span>[{plot.unit}]</span>
        {/if}
      </div>
    {/if}
    <PlotPlayback title={plot.title} {playback} />
  </article>
{:else}
  <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
{/if}
