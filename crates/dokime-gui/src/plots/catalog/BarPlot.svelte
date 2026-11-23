<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatNumber, formatUnitLabel } from "../../lib/format";
  import { categoryTotals, formatCategoryTotal } from "../categories";
  import { plotColor } from "../colors";
  import { plotPlaceholderText, plotUnavailableText } from "../plot-placeholder";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import type { PlotCardSpec } from "../types";
  import { arrangedBars, latestBars, paretoBars } from "./bar-layout";

  let { plot }: { plot: PlotCardSpec } = $props();
  const totals = $derived(categoryTotals(plot));
  const entries = $derived([...(totals.entries ?? [])].sort((left, right) => right.value - left.value || left.order - right.order));
  const arrangement = $derived(arrangedBars(plot));
  const bars = $derived(latestBars(plot));
  const minimum = $derived(Math.min(0, ...bars.map((bar) => bar.value)));
  const maximum = $derived(Math.max(0, ...bars.map((bar) => bar.value)));
  const span = $derived(maximum - minimum || 1);
  const baseline = $derived((-minimum / span) * 100);
  const summary = $derived(bars.length + (bars.length === 1 ? " category." : " categories."));
</script>

{#if plot.pareto}
  {#if plot.arrangement || plot.series.length > 1}
    <PlotPlaceholder title={plot.title} {...plotUnavailableText("Pareto cannot have an arrangement or series binding.")} />
  {:else if totals.entries === undefined}
    <PlotPlaceholder title={plot.title} {...plotUnavailableText(totals.error)} />
  {:else if !entries.length}
    <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
  {:else}
    {@const { bars, maximum, valueAxis } = paretoBars(entries)}
    <article class="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="pareto-plot">
      <h4 class="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <svg role="img" aria-label={plot.title} viewBox="0 0 720 360" class="w-full min-w-0">
        {#each bars as bar (bar.key)}
          <g>
            <rect
              data-category={bar.key}
              x={bar.x - 240 / entries.length}
              y={280 - bar.height}
              width={480 / entries.length}
              height={bar.height}
              fill={plot.colors[0] ?? plotColor(0)}
            >
              <title>{`${bar.key}: ${formatUnitLabel(formatCategoryTotal(bar, plot.yFormat ?? null), plot.unit)}`}</title>
            </rect>
            <text x={bar.x} y={300} text-anchor="middle" fill="currentColor" class="text-xs">{bar.key}</text>
            <text x={bar.x} y={Math.max(15, 274 - bar.height)} text-anchor="middle" fill="currentColor" class="text-xs">
              {formatCategoryTotal(bar, plot.yFormat ?? null)}
            </text>
          </g>
        {/each}
        <line x1={50} x2={50} y1={20} y2={280} stroke="currentColor"></line>
        {#each valueAxis.ticks(5) as value (value)}
          <text x={44} y={valueAxis(value)} text-anchor="end" dominant-baseline="middle" fill="currentColor" class="text-xs">
            {formatNumber(value, plot.yFormat ?? null)}
          </text>
        {/each}
        {#if maximum > 0}
          <g>
            <polyline
              data-cumulative
              fill="none"
              stroke={plotColor(1)}
              stroke-width={2}
              points={bars.map((bar) => `${bar.x},${280 - bar.cumulative * 2.6}`).join(" ")}
            ></polyline>
            {#each bars as bar (bar.key)}
              <circle data-percent={bar.cumulative} cx={bar.x} cy={280 - bar.cumulative * 2.6} r={3} fill={plotColor(1)}></circle>
            {/each}
            {#each [0, 25, 50, 75, 100] as percent (percent)}
              <text x={660} y={280 - percent * 2.6} fill="currentColor" dominant-baseline="middle" class="text-xs">{percent}%</text>
            {/each}
          </g>
        {/if}
      </svg>
    </article>
  {/if}
{:else if plot.arrangement}
  {#if arrangement}
    {@const { rows, x, height, rowHeight } = arrangement}
    <article class="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid={`${plot.arrangement}-bar-plot`}>
      <h4 class="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <svg role="img" aria-label={plot.title} viewBox={`0 0 720 ${height}`} class="w-full min-w-0">
        {#each rows as { category, rowIndex, bars } (category)}
          <g>
            <text x={140} y={20 + rowIndex * rowHeight + 14} text-anchor="end" fill="currentColor" class="text-xs">{category}</text>
            {#each bars as bar (bar.seriesIndex)}
              <rect
                data-category={category}
                data-series={plot.series[bar.seriesIndex].key}
                x={Math.min(x(bar.start), x(bar.end))}
                y={bar.top}
                width={Math.abs(x(bar.end) - x(bar.start))}
                height={20}
                fill={plot.colors[bar.seriesIndex] ?? plotColor(bar.seriesIndex)}
              >
                <title>
                  {`${plot.series[bar.seriesIndex].label}: ${formatUnitLabel(formatCategoryTotal(bar, plot.yFormat ?? null), plot.unit)}`}
                </title>
              </rect>
            {/each}
          </g>
        {/each}
        <line x1={x(0)} x2={x(0)} y1={14} y2={height - 30} stroke="currentColor"></line>
        {#each x.ticks(5) as value (value)}
          <text x={x(value)} y={height - 10} text-anchor="middle" fill="currentColor" class="text-xs">
            {formatNumber(value, plot.yFormat ?? null)}
          </text>
        {/each}
      </svg>
      <ul class="flex flex-wrap gap-x-4 gap-y-1 text-xs">
        {#each plot.series as series, index (series.key)}
          <li class="flex items-center gap-2">
            <span class="size-2" style:background-color={plot.colors[index] ?? plotColor(index)}></span>{series.label}
          </li>
        {/each}
      </ul>
    </article>
  {:else}
    <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
  {/if}
{:else if !bars.length}
  <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot, "No categorical samples are available in the selected window.")} />
{:else}
  <article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-4 rounded-lg bg-muted/20 p-4">
    <header class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <h4 class="truncate text-sm font-semibold" title={formatUnitLabel(plot.title, plot.unit)}>
          {formatUnitLabel(plot.title, plot.unit)}
        </h4>
        <p class="text-xs text-muted-foreground">{summary}</p>
      </div>
    </header>
    <div class="grid gap-3 pt-1">
      {#each bars as bar (bar.key)}
        <div
          class="grid min-h-10 min-w-0 grid-cols-[minmax(--spacing(26),--spacing(40))_minmax(--spacing(32),1fr)_max-content] items-center gap-3"
        >
          <span class="min-w-0 truncate text-xs font-semibold tabular-nums" title={bar.label}>{bar.label}</span>
          <div class="relative h-8 min-w-0 overflow-hidden rounded-lg bg-background ring-1 ring-border">
            <div
              class="absolute h-full rounded-lg"
              style:left={`${((Math.min(0, bar.value) - minimum) / span) * 100}%`}
              style:width={`${(Math.abs(bar.value) / span) * 100}%`}
              style:background-color={bar.color}
            ></div>
            <span class="absolute inset-y-0 w-px bg-foreground" style:left={`${baseline}%`}></span>
          </div>
          <span class="whitespace-nowrap text-right text-xs font-semibold tabular-nums"
            >{formatUnitLabel(
              formatCategoryTotal(bar, { kind: { kind: { case: "normal", value: { maxFractionDigits: 2 } } } }),
              plot.unit,
            )}</span
          >
        </div>
      {/each}
    </div>
  </article>
{/if}
