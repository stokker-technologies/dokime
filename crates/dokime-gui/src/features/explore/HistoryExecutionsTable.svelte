<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Eye } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import SortableTableHeader from "../../components/ui/SortableTableHeader.svelte";
  import TablePanel from "../../components/ui/TablePanel.svelte";
  import TableRowActions from "../../components/ui/TableRowActions.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../components/ui/table-order";
  import type { NotificationFailure } from "../../lib/notification-details";
  import type { ExecutionBrowseEntry } from "../../lib/types/browse";

  import { isDeletableExecution } from "./explore-contracts";

  let {
    rows,
    emptyCycle,
    failure,
    loading,
    onInspect,
    onDelete,
  }: {
    rows: ExecutionBrowseEntry[];
    emptyCycle: boolean;
    failure?: NotificationFailure;
    loading: boolean;
    onInspect: (entry: ExecutionBrowseEntry) => void;
    onDelete: (entry: ExecutionBrowseEntry) => void;
  } = $props();

  let order = $state.raw<TableOrder<"title" | "status">>();
  const executions = $derived(
    orderTableRows(
      rows.map((entry) => ({
        entry,
        title: entry.metadata.title,
        status: entry.availability === "interrupted" ? "Interrupted" : entry.metadata.status,
      })),
      order,
    ),
  );
</script>

{#snippet failureRow()}
  {#if failure}
    <ErrorCard {failure} />
  {/if}
{/snippet}

<TablePanel
  title="Executions"
  label="History Executions"
  description={emptyCycle ? "This sealed cycle contains no executions." : "Executions retained in this history entry."}
  columnCount={3}
  rowCount={executions.length}
  {loading}
  empty="No Executions in the Loaded History"
  failure={failure ? failureRow : undefined}
>
  {#snippet columns()}
    <SortableTableHeader
      class="px-4"
      label="Execution"
      direction={order?.column === "title" ? order.direction : undefined}
      onclick={() => (order = nextTableOrder(order, "title"))}
    />
    <SortableTableHeader
      class="w-36 px-2"
      label="Status"
      direction={order?.column === "status" ? order.direction : undefined}
      onclick={() => (order = nextTableOrder(order, "status"))}
    />
    <th scope="col" class="w-24 px-4 text-right font-medium">Actions</th>
  {/snippet}
  {#each executions as { entry, title, status } (entry.metadata.execution_id)}
    <tr data-row-id={entry.metadata.execution_id} class="h-(--list-row-height)">
      <td class="px-4">
        <span data-execution-id={entry.metadata.execution_id}>{title}</span>
      </td>
      <td class="px-2">{status}</td>
      <td class="px-4">
        <TableRowActions label={title} primary="Inspect" disabled={!entry.checkpoint} onSelect={() => onInspect(entry)}>
          {#snippet icon()}
            <Eye />
          {/snippet}
          <DropdownMenu.Item
            class="cursor-default rounded-lg px-2 py-1 text-sm text-destructive outline-none data-highlighted:bg-destructive/10 data-disabled:opacity-50"
            disabled={!isDeletableExecution(entry)}
            onSelect={() => onDelete(entry)}
            >Delete</DropdownMenu.Item
          >
        </TableRowActions>
      </td>
    </tr>
  {/each}
</TablePanel>
