<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import HistogramChart from "../HistogramChart.svelte";
  import StatsSummaryTable from "../StatsSummaryTable.svelte";
  import { histogramAxisTitle } from "../histogram-axes";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import { plotPlaceholderText, plotUnavailableText } from "../plot-placeholder";
  import type { PlotCardSpec } from "../types";

  let { plot }: { plot: PlotCardSpec } = $props();

  const analysis = $derived(plot.analysis?.kind === "histogram" ? plot.analysis : null);

  const series = $derived(analysis?.series ?? []);

  const axis = $derived(plot.histogramAxes?.x);

  const unit = $derived(axis?.unit ?? plot.histogram?.unit ?? plot.unit ?? "");

  const seriesLabel = $derived((key: string) => plot.series.find((item) => item.key === key)?.label ?? key);

  const rows = $derived(
    series.map((item) => ({
      label: series.length === 1 ? plot.title : seriesLabel(item.key),
      unit,
      statistics: item.statistics ?? null,
      count: item.count,
    })),
  );

  const single = $derived(series.length === 1);
</script>

{#if plot.analysis?.kind === "error"}
  <PlotPlaceholder title={plot.title} {...plotUnavailableText(plot.analysis.message)} />
{:else if !series.some((item) => item.count > 0)}
  <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
{:else}
  <article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-3 rounded-lg border bg-card p-4 text-card-foreground">
    {#each series as item (item.key)}
      <HistogramChart
        title={single ? plot.title : `${plot.title} / ${seriesLabel(item.key)}`}
        distribution={item}
        {unit}
        format={axis?.format ?? null}
        xTitle={axis ? histogramAxisTitle(axis) : undefined}
      />
    {/each}
    {#if plot.description}
      <p class="text-xs text-muted-foreground">{plot.description}</p>
    {/if}
    <StatsSummaryTable {rows} />
  </article>
{/if}
