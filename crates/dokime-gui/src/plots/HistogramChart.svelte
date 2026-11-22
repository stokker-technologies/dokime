<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ValueFormat } from "@generated/host/model";
  import type { Wire } from "../lib/rpc/canonical";
  import Button from "../components/ui/Button.svelte";
  import { formatNumber, formatUnitLabel } from "../lib/format";
  import { HISTOGRAM_COUNT_LABEL } from "./histogram-axes";
  import { type HistogramDistribution, histogramBars } from "@/plots/histogram-bins";
  import PlotPlaceholder from "./PlotPlaceholder.svelte";
  import { plotPlaceholderText } from "./plot-placeholder";
  type HistogramChartProps = {
    title: string;
    /** The backend's bins of this series; the browser does not bin samples. */
    distribution: HistogramDistribution;
    unit: string;
    /** The x axis title: the binned value's label and unit. */
    xTitle?: string;
    /** How a bin edge prints; `null` keeps the plain number. */
    format?: Wire<ValueFormat> | null;
    sampleLabel?: string;
  };
  function binCentre(index: number, count: number): number {
    return ((index + 0.5) / Math.max(1, count)) * 100;
  }
  function formatHistogramValue(value: number | undefined, unit: string, format: Wire<ValueFormat> | null): string {
    if (value === undefined || !Number.isFinite(value)) {
      return "-";
    }
    const text = format ? formatNumber(value, format) : value.toFixed(Math.abs(value) >= 100 ? 0 : 2);
    return formatUnitLabel(text, unit);
  }

  let { title, distribution, unit, xTitle, format = null, sampleLabel = "samples" }: HistogramChartProps = $props();

  let activeBinIndex = $state<number | null>(null);

  const bins = $derived(histogramBars(distribution));

  const maxCount = $derived(Math.max(1, ...bins.map((bin) => bin.count)));

  const activeBin = $derived(activeBinIndex === null ? undefined : bins[activeBinIndex]);
</script>

{#if !distribution.count}
  <PlotPlaceholder {title} {...plotPlaceholderText({})} />
{:else}
  <section class="grid min-h-44 min-w-0 max-w-full overflow-hidden gap-3">
    <div class="flex min-w-0 items-center justify-between gap-3">
      <strong class="min-w-0 truncate text-sm" title={formatUnitLabel(title, unit)}>{formatUnitLabel(title, unit)}</strong
      ><span class="shrink-0 text-xs text-muted-foreground">{distribution.count} {sampleLabel}</span>
    </div>
    <div class="grid min-w-0 gap-2">
      <span class="text-xs text-muted-foreground">{HISTOGRAM_COUNT_LABEL}</span>
      <div class="relative grid h-24 items-end gap-1" style:grid-template-columns={`repeat(${bins.length}, minmax(0, 1fr))`}>
        {#if activeBinIndex !== null && activeBin}
          <span
            class="pointer-events-none absolute bottom-0 top-0 w-px bg-foreground/60"
            style:left={`${binCentre(activeBinIndex, bins.length)}%`}
          ></span><span
            class="pointer-events-none absolute top-0 z-10 max-w-full -translate-x-1/2 truncate rounded-lg border bg-popover px-2 py-1 text-xs whitespace-nowrap text-popover-foreground shadow-sm"
            role="tooltip"
            style:left={`${Math.min(85, Math.max(15, binCentre(activeBinIndex, bins.length)))}%`}
            >{`${formatHistogramValue(activeBin.min, unit, format)} - ${formatHistogramValue(activeBin.max, unit, format)}: ${activeBin.count}`}</span
          >
        {/if}
        {#each bins as bin, index (bin.slot)}
          <Button
            variant="plain"
            size="inline"
            aria-label={`${formatHistogramValue(bin.min, unit, format)} to ${formatHistogramValue(bin.max, unit, format)}: ${bin.count}`}
            class="grid h-full min-w-0 grid-cols-1 items-end justify-stretch text-left"
            type="button"
            onblur={() => (activeBinIndex = null)}
            onfocus={() => (activeBinIndex = index)}
            onmouseenter={() => (activeBinIndex = index)}
            onmouseleave={() => (activeBinIndex = null)}
            ><span
              class="block w-full rounded-t bg-primary hover:bg-primary/85"
              style:height={bin.count > 0 ? `${(bin.count / maxCount) * 96}px` : "0"}
            ></span></Button
          >
        {/each}
      </div>
      <div class="grid gap-1" style:grid-template-columns={`repeat(${bins.length}, minmax(0, 1fr))`}>
        {#each bins as bin (`${bin.slot}-label`)}
          <span class="block truncate text-center text-xs text-muted-foreground" title={formatHistogramValue(bin.min, unit, format)}
            >{formatHistogramValue(bin.min, unit, format)}</span
          >
        {/each}
      </div>
      {#if xTitle}
        <span class="truncate text-center text-xs text-muted-foreground" title={xTitle}>{xTitle}</span>
      {/if}
    </div>
  </section>
{/if}
