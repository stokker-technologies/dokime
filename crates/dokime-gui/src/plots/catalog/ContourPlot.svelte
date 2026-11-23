<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatNumber, formatUnitLabel } from "../../lib/format";
  import { plotPlaceholderText } from "../plot-placeholder";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import type { PlotCardSpec } from "../types";
  import { contourLayout, HEIGHT, MARGIN, validContourGrid, WIDTH } from "./contour-layout";

  let {
    plot,
    setPanelSize,
  }: { plot: PlotCardSpec; setPanelSize?: (plotId: string, size: { width: number; height: number } | null) => void } = $props();

  const clip = $props.id();
  const grid = $derived(plot.analysis?.kind === "contour" ? plot.analysis.grid : null);
  const layout = $derived(grid && validContourGrid(grid) && grid.valid.some(Boolean) ? contourLayout(grid, plot) : null);
  const title = $derived(formatUnitLabel(plot.title, plot.unit));
  let element: SVGSVGElement | undefined = $state();
  // Derived, so a revision of the props keeps the measured panel and its plot request instead of resetting them.
  const plotKey = $derived(plot.key);
  const measurePanel = $derived(setPanelSize);

  $effect(() => {
    const measure = measurePanel;
    const key = plotKey;
    if (!element || !measure || typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(([entry]) => {
      if (!entry || entry.contentRect.width <= 0 || entry.contentRect.height <= 0) {
        return;
      }
      let width = Math.max(1, Math.round((entry.contentRect.width * (WIDTH - MARGIN.left - MARGIN.right)) / WIDTH));
      let height = Math.max(1, Math.round((entry.contentRect.height * (HEIGHT - MARGIN.top - MARGIN.bottom)) / HEIGHT));
      const factor = Math.min(1, Math.sqrt(262_144 / (width * height)), 512 / height);
      width = Math.max(1, Math.floor(width * factor));
      height = Math.max(1, Math.floor(height * factor));
      measure(key, { width, height });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
      measure(key, null);
    };
  });
</script>

{#if grid && layout}
  {@const { width, height, color, x, y, path, shapes, cells } = layout}
  <article class="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="contour-plot">
    <h4 class="truncate text-sm font-semibold" {title}>{title}</h4>
    <svg
      bind:this={element}
      role="img"
      aria-label={title}
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      class="aspect-[12/7] w-full min-w-0 rounded-lg bg-background"
    >
      <defs>
        <clipPath id={clip} clipPathUnits="userSpaceOnUse">
          {#each cells as cell (cell.index)}
            <rect data-valid-bin={cell.index} x={cell.left} y={cell.top} width={cell.right - cell.left} height={cell.bottom - cell.top} />
          {/each}
        </clipPath>
      </defs>
      <g>
        {#each cells as cell (cell.index)}
          {@const xStart = formatNumber(grid.xEdges[cell.column], plot.xFormat ?? null)}
          {@const xEnd = formatNumber(grid.xEdges[cell.column + 1], plot.xFormat ?? null)}
          {@const yStart = formatNumber(grid.yEdges[cell.row], plot.yFormat ?? null)}
          {@const yEnd = formatNumber(grid.yEdges[cell.row + 1], plot.yFormat ?? null)}
          {@const value = formatUnitLabel(formatNumber(cell.value, null), plot.unit)}
          <rect
            data-bin={cell.index}
            x={cell.left}
            y={cell.top}
            width={cell.right - cell.left}
            height={cell.bottom - cell.top}
            fill={color(cell.value)}
          >
            <title>{`${xStart}–${xEnd}, ${yStart}–${yEnd}: ${value}`}</title>
          </rect>
        {/each}
        {#if plot.render === "bands"}
          <g clip-path={`url(#${clip})`} data-contour-mask>
            {#each shapes as shape (shape.value)}
              <path
                d={path(shape) ?? undefined}
                fill={color(shape.value)}
                stroke="var(--foreground)"
                stroke-opacity={0.35}
                stroke-width={0.6}
                vector-effect="non-scaling-stroke"
              />
            {/each}
          </g>
        {/if}
        <g clip-path={`url(#${clip})`}>
          {#each plot.referenceBands as band}
            {#if typeof band.min === "number" && typeof band.max === "number"}
              {@const bottom = y(band.min)}
              {@const top = y(band.max)}
              {#if Number.isFinite(bottom) && Number.isFinite(top)}
                <rect
                  x={MARGIN.left}
                  {width}
                  y={Math.min(bottom, top)}
                  height={Math.abs(top - bottom)}
                  fill={band.color}
                  fill-opacity={0.12}
                />
              {/if}
            {/if}
          {/each}
          {#each plot.referenceLines as line}
            {#if typeof line.value === "number"}
              {@const row = y(line.value)}
              {#if Number.isFinite(row)}
                <line x1={MARGIN.left} x2={MARGIN.left + width} y1={row} y2={row} stroke={line.color} vector-effect="non-scaling-stroke" />
              {/if}
            {/if}
          {/each}
        </g>
      </g>
      {#each x.ticks(5) as value (value)}
        <text x={x(value)} y={MARGIN.top + height + 22} text-anchor="middle" fill="currentColor" class="text-xs">
          {formatNumber(value, plot.xFormat ?? null)}
        </text>
      {/each}
      {#each y.ticks(5) as value (value)}
        <text x={MARGIN.left - 8} y={y(value)} dominant-baseline="middle" text-anchor="end" fill="currentColor" class="text-xs">
          {formatNumber(value, plot.yFormat ?? null)}
        </text>
      {/each}
    </svg>
  </article>
{:else}
  <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
{/if}
