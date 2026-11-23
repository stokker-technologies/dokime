<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { DocumentViewTableRow } from "../../lib/types";
  import Table from "../ui/Table.svelte";

  let { rows }: { rows: DocumentViewTableRow[] } = $props();

  const number = /^(-|[-+]?\d+(\.\d+)?([eE][-+]?\d+)?)$/;
  const names = $derived([...new Set(rows.flatMap((row) => Object.keys(row.cells)))]);
  const columns = $derived(
    (names.length ? names : ["Value"]).map((name) => ({
      name,
      numeric: rows.some((row) => row.cells[name] !== undefined) && rows.every((row) => number.test(row.cells[name] ?? "-")),
    })),
  );
</script>

<section class="grid min-w-0 content-start overflow-hidden rounded-lg border bg-card text-card-foreground">
  <header class="grid min-w-0 gap-1 border-b px-4 py-2">
    <strong>Recorded Values</strong>
    <p class="text-xs text-muted-foreground">Rows retained in this document table.</p>
  </header>
  <Table aria-label="Recorded Values">
    <thead data-slot="table-header">
      <tr class="border-b" data-slot="table-row">
        {#each columns as { name, numeric } (name)}
          <th
            class={[
              "h-(--list-row-height) px-3 align-middle text-xs font-medium whitespace-nowrap text-muted-foreground",
              numeric ? "text-right" : "text-left",
            ]}
            scope="col"
            data-slot="table-head"
          >
            {name}
          </th>
        {/each}
      </tr>
    </thead>
    <tbody data-slot="table-body">
      {#each rows as row (row.row_id)}
        <tr class="h-(--list-row-height) border-b last:border-0 hover:bg-muted/50" data-slot="table-row" data-row-id={row.row_id}>
          {#each columns as { name, numeric } (name)}
            <td class={["px-3 py-2 align-middle whitespace-nowrap", numeric && "text-right tabular-nums"]} data-slot="table-cell">
              {row.cells[name] ?? "-"}
            </td>
          {/each}
        </tr>
      {:else}
        <tr data-slot="table-row">
          <td colspan={columns.length} class="h-24 text-center text-muted-foreground" data-slot="table-cell">No Table Rows Recorded</td>
        </tr>
      {/each}
    </tbody>
  </Table>
</section>
