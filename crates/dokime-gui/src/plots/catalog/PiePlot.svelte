<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatNumber, formatUnitLabel } from "../../lib/format";
  import { categoryFractions, categoryTotals, formatCategoryTotal } from "../categories";
  import { plotUnavailableText } from "../plot-placeholder";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import type { PlotCardSpec } from "../types";

  const CENTER = 160;
  const RADIUS = 130;

  let { plot }: { plot: PlotCardSpec } = $props();

  const totals = $derived(categoryTotals(plot));
  const hole = $derived(plot.hole ?? 0);
  const sectors = $derived.by(() => {
    const entries = totals.entries ?? [];
    const fractions = categoryFractions(entries);
    let angle = -Math.PI / 2;
    return entries.map((entry, index) => {
      const start = angle;
      const fraction = fractions[index];
      angle += fraction * Math.PI * 2;
      return { ...entry, fraction, path: sectorPath(start, angle, hole) };
    });
  });

  function sectorPath(start: number, end: number, hole: number): string {
    const middle = (start + end) / 2;
    const position = (angle: number, radius: number) => `${CENTER + Math.cos(angle) * radius},${CENTER + Math.sin(angle) * radius}`;
    const outer = `M${position(start, RADIUS)} A${RADIUS},${RADIUS} 0 0 1 ${position(middle, RADIUS)} A${RADIUS},${RADIUS} 0 0 1 ${position(end, RADIUS)}`;
    if (hole === 0) {
      return `${outer} L${CENTER},${CENTER} Z`;
    }
    const inner = RADIUS * hole;
    return `${outer} L${position(end, inner)} A${inner},${inner} 0 0 0 ${position(middle, inner)} A${inner},${inner} 0 0 0 ${position(start, inner)} Z`;
  }
</script>

{#if totals.entries === undefined}
  <PlotPlaceholder title={plot.title} {...plotUnavailableText(totals.error)} />
{:else if !Number.isFinite(hole) || hole < 0 || hole >= 1}
  <PlotPlaceholder title={plot.title} {...plotUnavailableText("Pie hole must be finite and in [0, 1).")} />
{:else if !sectors.some((sector) => sector.fraction > 0)}
  <PlotPlaceholder title={plot.title} label="No Positive Total" detail="Category totals sum to zero." />
{:else}
  <article class="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="pie-plot">
    <h4 class="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
    <svg viewBox="0 0 320 320" class="mx-auto aspect-square w-full max-w-80" role="img" aria-label={plot.title}>
      {#each sectors.filter((sector) => sector.fraction > 0) as sector (sector.key)}
        {@const total = formatUnitLabel(formatCategoryTotal(sector, plot.yFormat ?? null), plot.unit)}
        {@const percent = formatNumber(sector.fraction * 100, null)}
        <path data-category={sector.key} d={sector.path} fill={sector.color} stroke="var(--background)">
          <title>{`${sector.key}: ${total} · ${percent}%`}</title>
        </path>
      {/each}
    </svg>
    <ul class="flex flex-wrap gap-x-4 gap-y-1 text-xs">
      {#each sectors as sector (sector.key)}
        <li class="flex items-center gap-2">
          <span class="size-2 shrink-0" style:background-color={sector.color}></span>
          {sector.key}: {formatUnitLabel(formatCategoryTotal(sector, plot.yFormat ?? null), plot.unit)}
        </li>
      {/each}
    </ul>
  </article>
{/if}
