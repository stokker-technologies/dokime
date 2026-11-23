<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Pencil } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import SortableTableHeader from "../../../components/ui/SortableTableHeader.svelte";
  import TablePanel from "../../../components/ui/TablePanel.svelte";
  import TableRowActions from "../../../components/ui/TableRowActions.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../../components/ui/table-order";
  import type { SecretEntry } from "../../../lib/api/native-secrets";
  import type { NotificationFailure } from "../../../lib/notification-details";

  let {
    actions,
    busy,
    entries,
    failure,
    loading,
    onDelete,
    onEdit,
    onRetry,
  }: {
    actions?: Snippet;
    busy: boolean;
    entries: SecretEntry[];
    failure?: NotificationFailure;
    loading: boolean;
    onDelete: (entry: SecretEntry) => void;
    onEdit: (entry: SecretEntry) => void;
    onRetry?: () => void;
  } = $props();

  const sortableColumns = [
    { key: "identifier", label: "Identifier" },
    { key: "label", label: "Label" },
  ] as const;
  type Column = (typeof sortableColumns)[number]["key"];
  let order = $state.raw<TableOrder<Column>>();
  const rows = $derived(
    orderTableRows(
      entries.map((entry) => ({ entry, ...entry.metadata })),
      order,
    ),
  );
</script>

{#snippet failureRow()}
  {#if failure}
    <ErrorCard {failure} {onRetry} notificationContext={onRetry ? undefined : "action"} />
  {/if}
{/snippet}

<TablePanel
  title="Shared Secrets"
  label="Secrets"
  description="All plugins on this host can look up these identifiers. Values stay in its OS keyring."
  columnCount={5}
  rowCount={rows.length}
  {loading}
  empty="No Shared Secrets"
  {actions}
  failure={failure ? failureRow : undefined}
>
  {#snippet columns()}
    {#each sortableColumns as column (column.key)}
      <SortableTableHeader
        class="px-2 first:pl-4"
        label={column.label}
        direction={order?.column === column.key ? order.direction : undefined}
        onclick={() => (order = nextTableOrder(order, column.key))}
      />
    {/each}
    <th scope="col" class="w-28 px-2 font-medium">Value</th>
    <th scope="col" class="px-2 font-medium">References</th>
    <th scope="col" class="w-24 px-4 text-right font-medium">Actions</th>
  {/snippet}
  {#each rows as { entry, identifier, label, description } (identifier)}
    <tr class="h-(--list-row-height)" data-row-id={identifier}>
      <td class="px-4 font-mono text-xs">{identifier}</td>
      <td class="px-2">
        <div class="grid gap-1">
          <strong>{label}</strong>
          <p class="text-xs text-muted-foreground">{description}</p>
        </div>
      </td>
      <td class="px-2">
        <span aria-hidden="true">••••••••</span>
        <span class="sr-only">Masked Value</span>
      </td>
      <td class="px-2 text-xs">{entry.knownUses.join(", ") || "—"}</td>
      <td class="px-4">
        <TableRowActions label={identifier} primary="Edit" disabled={busy} onSelect={() => onEdit(entry)}>
          {#snippet icon()}
            <Pencil />
          {/snippet}
          <DropdownMenu.Item
            class="cursor-default rounded-lg px-2 py-1 text-sm text-destructive outline-none data-highlighted:bg-destructive/10 data-disabled:opacity-50"
            disabled={busy}
            onSelect={() => onDelete(entry)}
            >Delete</DropdownMenu.Item
          >
        </TableRowActions>
      </td>
    </tr>
  {/each}
</TablePanel>
