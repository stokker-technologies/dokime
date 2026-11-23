<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { SeriesLegendColumn, SeriesLegendRow, StatsSummaryRow } from "../lib/statistics";

  type SeriesLegend = {
    columns: SeriesLegendColumn[];
    rows: SeriesLegendRow[];
    pending: boolean;
    onToggle?: (key: string) => void;
  };
</script>

<script lang="ts">
  import AbbreviatedValue from "../components/AbbreviatedValue.svelte";
  import Table from "../components/ui/Table.svelte";
  import { seriesLegendColumns, seriesLegendRows } from "../lib/statistics";

  let { rows = [], legend }: { rows?: StatsSummaryRow[]; legend?: SeriesLegend } = $props();
  const columns = $derived([
    { id: "trace", header: "Metric" } satisfies SeriesLegendColumn,
    ...seriesLegendColumns(rows[0]?.unit).slice(3),
  ]);
  const displayed = $derived(
    seriesLegendRows({
      hidden: new Set(),
      summaries: rows.map((row, index) => ({ ...row.statistics, series_id: String(index), count: row.count })),
      traces: rows.map((row, index) => ({ key: String(index), label: row.label, color: "" })),
    }),
  );
  const visibleColumns = $derived(legend?.columns ?? columns);
  const visibleRows = $derived(legend?.rows ?? displayed);
  const head = "h-(--list-row-height) px-3 align-middle text-xs font-medium whitespace-nowrap text-muted-foreground";
  const rowStyle = "h-(--list-row-height) border-b transition-colors hover:bg-muted/50";
  const cell = "px-3 py-2 align-middle whitespace-nowrap";
</script>

{#snippet trace(
  row: SeriesLegendRow,
)}
  <span class={["flex min-w-0 items-center gap-2", row.hidden && "opacity-50"]} data-trace-hidden={row.hidden ? "true" : undefined}>
    <span
      aria-hidden="true"
      class="size-2.5 shrink-0 rounded-full border-2"
      style:background-color={row.hidden ? "transparent" : row.color}
      style:border-color={row.color}
    ></span>
    <span class="truncate" title={row.label}>{row.label}</span>
    {#if row.hidden}
      <span class="sr-only">(Hidden)</span>
    {/if}
  </span>
{/snippet}

<Table
  aria-label={legend ? "Trace Statistics" : "Histogram Statistics"}
  class={legend?.pending ? "transition-opacity opacity-50" : "transition-opacity"}
>
  <thead data-slot="table-header" class="[&_tr]:border-b">
    <tr data-slot="table-row" class={rowStyle}>
      {#each visibleColumns as { header, id } (id)}
        <th scope="col" data-slot="table-head" class={[head, id === "trace" ? "text-left" : "text-right"]} title={header}>{header}</th>
      {/each}
    </tr>
  </thead>
  <tbody data-slot="table-body" class="[&_tr:last-child]:border-0">
    {#each visibleRows as row (row.key)}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions (The trace button provides keyboard activation.) -->
      <tr
        data-slot="table-row"
        class={[rowStyle, legend?.onToggle && "cursor-pointer"]}
        data-row-id={row.key}
        onclick={legend?.onToggle ? () => legend?.onToggle?.(row.key) : undefined}
      >
        {#each visibleColumns as { id } (id)}
          <td data-slot="table-cell" class={[cell, id !== "trace" && "text-right tabular-nums"]}>
            {#if id === "trace"}
              {#if legend?.onToggle}
                <button type="button" aria-pressed={!row.hidden} class="block w-full text-left">{@render trace(row)}</button>
              {:else if legend}
                {@render trace(row)}
              {:else}
                {row.label}
              {/if}
            {:else}
              <AbbreviatedValue
                class={row.hidden ? "opacity-50" : undefined}
                exact={id === "xCursor" || id === "yCursor" ? row[id] : row.cells[id]}
              />
            {/if}
          </td>
        {/each}
      </tr>
    {:else}
      <tr data-slot="table-row" class={rowStyle}>
        <td data-slot="table-cell" class={cell} colspan={visibleColumns.length}>
          {legend ? "No Traces Recorded" : "No Statistics Recorded"}
        </td>
      </tr>
    {/each}
  </tbody>
</Table>
