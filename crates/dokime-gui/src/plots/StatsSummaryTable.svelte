<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import AbbreviatedValue from "../components/AbbreviatedValue.svelte";
  import Table from "../components/ui/Table.svelte";
  import { type SeriesLegendColumn, seriesLegendColumns, seriesLegendRows, type StatsSummaryRow } from "../lib/statistics";

  let { rows }: { rows: StatsSummaryRow[] } = $props();

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
  const head = "h-(--list-row-height) px-3 align-middle text-xs font-medium whitespace-nowrap text-muted-foreground";
  const rowStyle = "h-(--list-row-height) border-b transition-colors hover:bg-muted/50";
  const cell = "px-3 py-2 align-middle whitespace-nowrap";
</script>

<Table aria-label="Histogram Statistics">
  <thead data-slot="table-header" class="[&_tr]:border-b">
    <tr data-slot="table-row" class={rowStyle}>
      {#each columns as { header, id } (id)}
        <th scope="col" data-slot="table-head" class={[head, id === "trace" ? "text-left" : "text-right"]} title={header}>{header}</th>
      {/each}
    </tr>
  </thead>
  <tbody data-slot="table-body" class="[&_tr:last-child]:border-0">
    {#each displayed as row (row.key)}
      <tr data-slot="table-row" class={rowStyle} data-row-id={row.key}>
        {#each columns as { id } (id)}
          <td data-slot="table-cell" class={[cell, id !== "trace" && "text-right tabular-nums"]}>
            {#if id === "trace"}
              {row.label}
            {:else}
              <AbbreviatedValue exact={id === "xCursor" || id === "yCursor" ? row[id] : row.cells[id]} />
            {/if}
          </td>
        {/each}
      </tr>
    {:else}
      <tr data-slot="table-row" class={rowStyle}>
        <td data-slot="table-cell" class={cell} colspan={columns.length}>No Statistics Recorded</td>
      </tr>
    {/each}
  </tbody>
</Table>
