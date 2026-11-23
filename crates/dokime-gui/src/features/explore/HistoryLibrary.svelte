<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { ArrowRight, Cloud, Database, GitCompare, HardDrive } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Badge from "../../components/ui/Badge.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Checkbox from "../../components/ui/Checkbox.svelte";
  import SortableTableHeader from "../../components/ui/SortableTableHeader.svelte";
  import TablePanel from "../../components/ui/TablePanel.svelte";
  import TableRowActions from "../../components/ui/TableRowActions.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../components/ui/table-order";
  import { activateTableRow } from "../../components/ui/table-row";
  import type { NotificationFailure } from "../../lib/notification-details";
  import { exploreEntryKey, type ExploreEntry, type ExploreTarget } from "./explore-contracts";

  import { formatHistoryDate } from "./history-format";

  let {
    archived,
    cycles,
    failure,
    loading,
    onCompare,
    onOpenEntry,
    onRetry,
    onToggleArchived,
    onToggleSelected,
    onUnarchive,
    selectedIds,
  }: {
    archived: boolean;
    cycles: ExploreEntry[];
    failure?: NotificationFailure;
    loading: boolean;
    onCompare: () => void;
    onOpenEntry: (target: ExploreTarget) => void;
    onRetry: () => void;
    onToggleArchived: () => void;
    onToggleSelected: (entryKey: string) => void;
    onUnarchive: (entry: ExploreEntry) => void;
    selectedIds: Set<string>;
  } = $props();

  const sortableColumns = [
    { key: "title", label: "Execution / Cycle", width: "w-64" },
    { key: "failed_count", label: "Result", width: "w-36" },
    { key: "lifecycle", label: "Lifecycle", width: "w-32" },
    { key: "owner", label: "Owner", width: "w-32" },
    { key: "sourceLabel", label: "Source", width: "w-36" },
    { key: "updatedAt", label: "Updated", width: "w-44 text-right" },
  ] as const;
  type Column = (typeof sortableColumns)[number]["key"];
  let order = $state.raw<TableOrder<Column>>();
  const rows = $derived(
    orderTableRows(
      cycles.map((entry) => ({ ...entry, sourceLabel: entry.source.label, updatedAt: Date.parse(entry.updated_at) || undefined })),
      order,
    ),
  );
</script>

{#snippet failureRow()}
  {#if failure}
    <ErrorCard {failure} {onRetry} />
  {/if}
{/snippet}

<TablePanel
  title={archived ? "Archived History" : "History"}
  description="Browse saved entries or select entries to compare."
  columnCount={sortableColumns.length + 2}
  rowCount={rows.length}
  {loading}
  empty={archived ? "No Archived History" : "No History Matches"}
  failure={failure ? failureRow : undefined}
>
  {#snippet actions()}
    <Button
      size="sm"
      disabled={selectedIds.size < 2}
      onclick={onCompare}
      title={selectedIds.size < 2 ? "Select at Least Two Entries to Compare" : undefined}
    >
      <GitCompare />
      {selectedIds.size < 2 ? `Compare (Select ${2 - selectedIds.size} More)` : `Compare Selected (${selectedIds.size})`}
    </Button>
    <Button size="sm" variant={archived ? "secondary" : "outline"} aria-pressed={archived} onclick={onToggleArchived}> Archived </Button>
  {/snippet}
  {#snippet columns()}
    <th scope="col" class="w-12 px-4"><span class="sr-only">Select for Comparison</span></th>
    {#each sortableColumns as column (column.key)}
      <SortableTableHeader
        class={`px-2 ${column.width}`}
        label={column.label}
        direction={order?.column === column.key ? order.direction : undefined}
        onclick={() => (order = nextTableOrder(order, column.key))}
      />
    {/each}
    <th scope="col" class="w-24 px-4 text-right font-medium">Actions</th>
  {/snippet}
  {#each rows as entry (exploreEntryKey(entry.target))}
    {@const key = exploreEntryKey(entry.target)}
    {@const selected = selectedIds.has(key)}
    {@const Icon = entry.source.kind === "storage" ? Database : entry.source.kind === "local" ? HardDrive : Cloud}
    <tr
      data-row-id={key}
      data-state={selected ? "selected" : undefined}
      class="h-(--list-row-height) cursor-pointer hover:bg-muted/50 data-[state=selected]:bg-muted"
      {@attach (row) => activateTableRow(row, () => onOpenEntry(entry.target))}
    >
      <td class="px-4">
        <Checkbox aria-label={`Select ${entry.title} for comparison`} checked={selected} onCheckedChange={() => onToggleSelected(key)} />
      </td>
      <td class="px-2">
        <Button
          variant="plain"
          size="inline"
          class="flex w-full min-w-0 gap-2 text-left"
          title={entry.title}
          onclick={() => onOpenEntry(entry.target)}
        >
          <span class="truncate">{entry.title}</span>
          {#if entry.target.kind === "execution"}
            <Badge variant="outline">Standalone Execution</Badge>
          {/if}
        </Button>
      </td>
      <td class="px-2">
        <div class="flex items-center gap-2 text-xs font-medium">
          <span class="text-success">{entry.passed_count} Passed</span>
          <span class={entry.failed_count > 0 ? "text-destructive" : "text-muted-foreground"}>{entry.failed_count} Failed</span>
        </div>
      </td>
      <td class="px-2 capitalize">{entry.phase ?? "Unknown"} · {entry.lifecycle}</td>
      <td class="truncate px-2">{entry.owner ?? "—"}</td>
      <td class="px-2">
        <Badge
          class={[
            "h-5 gap-1 rounded px-2 text-xs font-medium [&>svg]:size-3",
            entry.source.kind === "remote" && "border-info/30 bg-info/10 text-info",
          ]}
          variant={entry.source.kind === "remote" ? "secondary" : "outline"}
        >
          <Icon />
          {entry.source.label}
        </Badge>
      </td>
      <td class="px-2 text-right text-muted-foreground">{formatHistoryDate(entry.updated_at)}</td>
      <td class="px-4">
        {#snippet icon()}
          <ArrowRight />
        {/snippet}
        {#snippet more()}
          <DropdownMenu.Item
            class="cursor-default rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent"
            onSelect={() => onUnarchive(entry)}
            >Unarchive</DropdownMenu.Item
          >
        {/snippet}
        <TableRowActions
          {icon}
          primary="Open"
          label={entry.title}
          onSelect={() => onOpenEntry(entry.target)}
          children={archived ? more : undefined}
        />
      </td>
    </tr>
  {/each}
</TablePanel>
