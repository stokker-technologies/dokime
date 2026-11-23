<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Table from "../../components/ui/Table.svelte";
  import { formatDuration } from "../../lib/format";
  import { exploreEntryKey, type ExploreEntry } from "./explore-contracts";

  import { formatHistoryDate } from "./history-format";

  let { cycles }: { cycles: ExploreEntry[] } = $props();
  const metrics: { label: string; value: (cycle: ExploreEntry) => string }[] = [
    { label: "Owner", value: (cycle) => cycle.owner ?? "—" },
    { label: "Source", value: (cycle) => cycle.source.label },
    { label: "Cases", value: (cycle) => String(cycle.case_count) },
    { label: "Passed", value: (cycle) => String(cycle.passed_count) },
    { label: "Failed", value: (cycle) => String(cycle.failed_count) },
    { label: "Duration", value: (cycle) => formatDuration(cycle.duration_s) ?? "—" },
    { label: "Updated", value: (cycle) => formatHistoryDate(cycle.updated_at) },
  ];
</script>

{#if cycles.length < 2}
  <p role="status" class="text-sm text-muted-foreground">Select at least two entries in the library to compare them.</p>
{:else}
  <div data-slot="data-table" class="grid min-w-0 content-start overflow-hidden rounded-lg border bg-card text-card-foreground">
    <header class="border-b px-4 py-3">
      <h2 class="text-sm font-semibold">Comparison</h2>
      <p class="mt-1 text-xs text-muted-foreground">Comparison covers the summary metrics above.</p>
    </header>
    <Table aria-label="Comparison" class="table-fixed">
      <thead class="border-b bg-muted/50 text-left">
        <tr class="h-(--list-row-height)">
          <th scope="col" class="w-32 px-4 font-medium">Metric</th>
          {#each cycles as cycle (exploreEntryKey(cycle.target))}
            <th scope="col" class="truncate px-2 text-left font-medium last:pr-4" title={cycle.title}>{cycle.title}</th>
          {/each}
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each metrics as metric (metric.label)}
          <tr class="h-(--list-row-height)">
            <th scope="row" class="px-4 text-left font-normal text-muted-foreground">{metric.label}</th>
            {#each cycles as cycle (exploreEntryKey(cycle.target))}
              {@const value = metric.value(cycle)}
              <td class="truncate px-2 last:pr-4" title={value}>{value}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </Table>
  </div>
{/if}
