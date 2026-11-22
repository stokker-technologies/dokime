<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Table from "./Table.svelte";

  let {
    title,
    label = title,
    description,
    columnCount,
    columns,
    rowCount,
    loading,
    empty,
    actions,
    failure,
    children,
  }: {
    title: string;
    label?: string;
    description?: string;
    columnCount: number;
    columns: Snippet;
    rowCount: number;
    loading?: boolean;
    empty: string;
    actions?: Snippet;
    failure?: Snippet;
    children?: Snippet;
  } = $props();
</script>

<section data-slot="data-table" class="grid min-w-0 content-start overflow-hidden rounded-lg border bg-card text-card-foreground">
  <header class="flex flex-wrap items-center gap-2 border-b px-4 py-3">
    <div class="min-w-48 flex-1">
      <h2 class="text-sm font-semibold">{title}</h2>
      {#if description}
        <p class="mt-1 text-xs text-muted-foreground">{description}</p>
      {/if}
    </div>
    {@render actions?.()}
  </header>
  <Table aria-label={label} aria-busy={Boolean(loading)} class="table-fixed">
    <thead class="border-b bg-muted/50 text-left">
      <tr class="h-(--list-row-height)">
        {@render columns()}
      </tr>
    </thead>
    <tbody class="divide-y">
      {#if failure}
        <tr>
          <td colspan={columnCount} class="px-4 py-3">{@render failure()}</td>
        </tr>
      {/if}
      {@render children?.()}
      {#if !rowCount && !failure}
        <tr>
          <td colspan={columnCount} class="h-24 px-4 text-center text-muted-foreground">
            <span role={loading ? "status" : undefined}>{loading ? "Loading Rows…" : empty}</span>
          </td>
        </tr>
      {/if}
    </tbody>
  </Table>
</section>
