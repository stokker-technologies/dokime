<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { Snippet } from "svelte";

  export type PluginRow = {
    alert?: Snippet;
    badge?: Snippet;
    enabled?: { checked: boolean; disabled?: boolean; onChange: (next: boolean) => void };
    key: string;
    name: string;
    onOpenSettings?: () => void;
    placement?: "Managed" | "Unmanaged";
    source: string;
    status?: string;
    trailing?: Snippet;
  };
</script>

<script lang="ts">
  import { Settings2 } from "lucide-svelte";
  import Button from "../../../components/ui/Button.svelte";
  import SortableTableHeader from "../../../components/ui/SortableTableHeader.svelte";
  import Switch from "../../../components/ui/Switch.svelte";
  import TablePanel from "../../../components/ui/TablePanel.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../../components/ui/table-order";
  import { activateTableRow } from "../../../components/ui/table-row";

  let {
    actions,
    failure,
    label,
    loading,
    rows,
  }: {
    actions?: Snippet;
    failure?: Snippet;
    label: string;
    loading?: boolean;
    rows: PluginRow[];
  } = $props();
  const sortableColumns = [
    { key: "name", label: "Plugin", width: "pl-4" },
    { key: "status", label: "Status", width: "w-48" },
    { key: "placement", label: "Placement", width: "w-28" },
    { key: "source", label: "Source", width: "w-[34%]" },
  ] as const;
  type Column = (typeof sortableColumns)[number]["key"];
  let order = $state.raw<TableOrder<Column>>();
  const plugins = $derived(orderTableRows(rows, order));
</script>

<TablePanel
  title={label}
  description="Installed plugins and their current activation state."
  columnCount={6}
  rowCount={plugins.length}
  {loading}
  empty="No Plugins"
  {actions}
  {failure}
>
  {#snippet columns()}
    {#each sortableColumns as column (column.key)}
      <SortableTableHeader
        class={`px-2 ${column.width}`}
        label={column.label}
        direction={order?.column === column.key ? order.direction : undefined}
        onclick={() => (order = nextTableOrder(order, column.key))}
      />
    {/each}
    <th scope="col" class="w-24 px-2 font-medium">Enabled</th>
    <th scope="col" class="w-32 px-4 text-right font-medium">Actions</th>
  {/snippet}
  {#each plugins as row (row.key)}
    <tr
      data-row-id={row.key}
      class={["h-(--list-row-height) hover:bg-muted/50", row.onOpenSettings && "cursor-pointer"]}
      {@attach (element) => activateTableRow(element, () => row.onOpenSettings?.())}
    >
      <td class="truncate px-4 font-medium" title={row.name}>{row.name}</td>
      <td class="px-2">{@render row.badge?.()}</td>
      <td class="px-2 text-xs">{row.placement ?? ""}</td>
      <td class="truncate px-2 text-xs text-muted-foreground" title={row.source}>{row.source}</td>
      <td class="px-2">
        {#if row.enabled}
          <Switch
            aria-label={`${row.enabled.checked ? "Disable" : "Enable"} ${row.name}`}
            checked={row.enabled.checked}
            disabled={row.enabled.disabled}
            onCheckedChange={row.enabled.onChange}
          />
        {/if}
      </td>
      <td class="px-4">
        <div class="flex items-center justify-end gap-1">
          <Button
            aria-label="Settings"
            title="Settings"
            size="icon-sm"
            variant="ghost"
            disabled={!row.onOpenSettings}
            onclick={row.onOpenSettings}
          >
            <Settings2 />
          </Button>
          {@render row.trailing?.()}
        </div>
      </td>
    </tr>
    {#if row.alert}
      <tr>
        <td colspan="6" class="px-4 py-3">{@render row.alert()}</td>
      </tr>
    {/if}
  {/each}
</TablePanel>
