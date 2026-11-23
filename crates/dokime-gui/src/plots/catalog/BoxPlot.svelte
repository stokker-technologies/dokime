<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatSummaryValue, type StatisticValue } from "../../lib/statistics";
  import { formatUnitLabel } from "../../lib/format";
  import { isNumber } from "../../lib/utils";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import { plotPlaceholderText, plotUnavailableText } from "../plot-placeholder";
  import type { PlotCardSpec } from "../types";
  type BoxSummaryBucket = {
    key: string;
    label: string;
    minimum: number;
    q1: number;
    median: number;
    exactMedian: StatisticValue;
    q3: number;
    maximum: number;
    outliers: number[];
  };
  function position(value: number | null | undefined, min: number, range: number): number {
    return value === null || value === undefined ? 0 : ((value - min) / range) * 100;
  }
  function plotBoxSummaryBuckets(plot: PlotCardSpec): BoxSummaryBucket[] {
    if (plot.analysis?.kind !== "box_plot") {
      return [];
    }

    return plot.analysis.series
      .map((summary) => ({
        key: summary.key,
        label: summary.label,
        minimum: summary.lower_whisker,
        maximum: summary.upper_whisker,
        q1: Number(summary.q1),
        median: Number(summary.median),
        exactMedian: summary.median,
        q3: Number(summary.q3),
        outliers: summary.outliers,
      }))
      .sort(compareBuckets);
  }
  function compareBuckets(left: BoxSummaryBucket, right: BoxSummaryBucket): number {
    const leftNumber = Number(left.label);
    const rightNumber = Number(right.label);
    if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
      return leftNumber - rightNumber;
    }
    return left.label.localeCompare(right.label, undefined, { numeric: true });
  }

  let { plot }: { plot: PlotCardSpec } = $props();

  const buckets = $derived(plotBoxSummaryBuckets(plot));

  const totalSamples = $derived(plot.analysis?.kind === "box_plot" ? plot.analysis.total : 0);

  const values = $derived(
    buckets.flatMap((bucket) => [bucket.minimum, bucket.q1, bucket.median, bucket.q3, bucket.maximum, ...bucket.outliers]).filter(isNumber),
  );

  const min = $derived(Math.min(...values));

  const max = $derived(Math.max(...values));

  const range = $derived(Math.max(max - min, 0.001));
</script>

{#if plot.analysis?.kind === "error"}
  <PlotPlaceholder title={plot.title} {...plotUnavailableText(plot.analysis.message)} />
{:else if !buckets.length}
  <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
{:else}
  <article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-4 rounded-lg bg-muted/20 p-4">
    <header class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <h4 class="truncate text-sm font-semibold" title={formatUnitLabel(plot.title, plot.unit)}>
          {formatUnitLabel(plot.title, plot.unit)}
        </h4>
        <p class="text-xs text-muted-foreground">{buckets.length} categor{buckets.length === 1 ? "y" : "ies"}, {totalSamples} samples</p>
      </div>
    </header>
    <div class="grid gap-3 pt-1">
      {#each buckets as bucket (bucket.key)}
        <div
          class="grid min-h-14 min-w-0 grid-cols-[minmax(--spacing(26),--spacing(40))_minmax(--spacing(40),1fr)_max-content] items-center gap-3"
        >
          <span class="min-w-0 truncate text-xs font-semibold tabular-nums" title={bucket.label}>{bucket.label}</span>
          <div class="relative h-12 min-w-0 overflow-hidden rounded-lg bg-background ring-1 ring-border">
            {#each [...new Set(bucket.outliers)] as value (value)}
              <span
                class="absolute top-1/2 h-1 w-1 rounded-full bg-foreground"
                style:left={`${position(value, min, range)}%`}
                title={`Outlier: ${`${formatSummaryValue(value)} ${plot.unit ?? ""}`}`}
              ></span>
            {/each}
            <span class="absolute left-2 right-2 top-1/2 h-px -translate-y-1/2 bg-border"></span
            ><span
              class="absolute top-1/2 h-px -translate-y-1/2 bg-foreground"
              style:left={`${position(bucket.minimum, min, range)}%`}
              style:right={`${100 - position(bucket.maximum, min, range)}%`}
            ></span><span
              class="absolute top-1/2 h-7 -translate-y-1/2 rounded border border-primary/50 bg-primary/20"
              style:left={`${position(bucket.q1, min, range)}%`}
              style:right={`${100 - position(bucket.q3, min, range)}%`}
            ></span><span
              class="absolute top-1/2 h-9 w-px -translate-y-1/2 bg-primary"
              style:left={`${position(bucket.median, min, range)}%`}
            ></span>
          </div>
          <span class="whitespace-nowrap text-right text-xs font-semibold tabular-nums">
            {plot.unit
              ? formatUnitLabel(formatSummaryValue(bucket.exactMedian), plot.unit)
              : `${formatSummaryValue(bucket.exactMedian)} value`}
          </span>
        </div>
      {/each}
    </div>
  </article>
{/if}
