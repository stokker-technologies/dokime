<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Table from "../../components/ui/Table.svelte";
  import { formatNumber, formatUnitLabel } from "../../lib/format";
  import { sequentialColorScale } from "../colors";
  import { orderedInstants } from "../instants";
  import { plotPlaceholderText } from "../plot-placeholder";
  import { plotPlayback } from "../playback.svelte";
  import PlotPlaceholder from "../PlotPlaceholder.svelte";
  import PlotPlayback from "../PlotPlayback.svelte";
  import type { PlotCardSpec } from "../types";

  let { plot }: { plot: PlotCardSpec } = $props();

  const data = $derived(plot.analysis?.kind === "grid_chart" ? plot.analysis.data : null);
  const instants = $derived(orderedInstants(data?.frames.map((frame) => frame.instant) ?? []));
  const playback = plotPlayback(
    () => data,
    () => instants,
  );
  const shown = $derived(playback.data);
  const frame = $derived(shown?.frames.find((frame) => frame.instant === playback.instant));
  const values = $derived(frame?.values.filter((value, index) => frame?.valid[index] && Number.isFinite(value)) ?? []);
  const { min, max, color } = $derived(sequentialColorScale(values));
  const total = $derived(values.reduce((sum, value) => sum + value, 0));
</script>

{#if shown && frame}
  <article class="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="grid-chart-plot">
    <div class="min-w-0 rounded-lg border bg-card text-card-foreground" data-slot="data-table">
      <div class="border-b px-3 py-2">
        <h4 class="text-sm font-semibold">{plot.title}</h4>
        <p class="text-xs text-muted-foreground">Values and highlighted cells at the selected instant.</p>
      </div>
      <Table aria-label={plot.title}>
        <thead class="border-b text-xs text-muted-foreground" data-slot="table-header">
          <tr class="h-(--list-row-height)" data-slot="table-row">
            <th scope="col" class="whitespace-nowrap px-3 text-left font-medium" data-slot="table-head">Row</th>
            {#each shown.columns as column (column.key)}
              <th scope="col" class="whitespace-nowrap px-3 text-right font-medium" data-slot="table-head">
                {formatUnitLabel(column.group ? `${column.group} · ${column.label}` : column.label, plot.unit)}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="[&_tr:last-child]:border-0" data-slot="table-body">
          {#each shown.rows as row, rowIndex (row.key)}
            <tr class="h-(--list-row-height) border-b hover:bg-muted/50" data-slot="table-row">
              <th scope="row" class="whitespace-nowrap px-3 py-2 text-left font-normal" title={row.group} data-slot="table-cell">
                {row.group ? `${row.group} · ` : ""}{row.label}
              </th>
              {#each shown.columns as column, columnIndex (column.key)}
                {@const index = rowIndex * shown.columns.length + columnIndex}
                {@const value = frame.values[index]}
                {@const present = frame.valid[index] && Number.isFinite(value)}
                {@const highlights = frame.highlights.filter((cell) => cell.row === rowIndex && cell.column === columnIndex)}
                {@const label = present ? formatNumber(value, plot.yFormat ?? null) : ""}
                {@const description = present ? formatUnitLabel(label, plot.unit) : "No Value"}
                <td class="whitespace-nowrap px-3 py-2 text-right" data-slot="table-cell">
                  <div
                    class="px-2 tabular-nums {present && max > min && (value - min) / (max - min) > 0.72
                      ? "text-white"
                      : "text-zinc-950"} {highlights.length ? "ring-2 ring-inset ring-primary" : ""}"
                    data-cell={`${row.key}/${column.key}`}
                    data-valid={present ? "true" : "false"}
                    data-highlight={highlights.length ? "true" : undefined}
                    title={[row.label, column.label, description, ...highlights.map((cell) => cell.label)].join(" · ")}
                    style={present ? `background-color: ${color(value)}` : undefined}
                  >
                    {label}
                  </div>
                </td>
              {/each}
            </tr>
          {:else}
            <tr>
              <td colspan={shown.columns.length + 1} class="px-3 py-2 text-muted-foreground">No Grid Rows Recorded</td>
            </tr>
          {/each}
        </tbody>
      </Table>
    </div>
    {#if shown.summaryLabel}
      <output class="text-xs text-muted-foreground"
        >{formatUnitLabel(shown.summaryLabel, plot.unit)}: {formatNumber(total, plot.yFormat ?? null)}</output
      >
    {/if}
    <PlotPlayback title={plot.title} {playback} />
  </article>
{:else}
  <div class="grid min-w-0 gap-2">
    <PlotPlaceholder title={plot.title} {...plotPlaceholderText(plot)} />
    <PlotPlayback title={plot.title} {playback} />
  </div>
{/if}
