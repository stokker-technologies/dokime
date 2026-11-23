<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  export type PluginRow = {
    failure?: NotificationFailure;
    badge?: "destructive" | "outline" | "success" | "warning";
    enabled?: { checked: boolean; disabled?: boolean; onChange: (next: boolean) => void };
    key: string;
    name: string;
    onOpenSettings?: () => void;
    placement?: "Managed" | "Unmanaged";
    source: string;
    status?: string;
    reload?: { disabled?: boolean; select: () => void };
    remove?: { disabled?: boolean; select: () => void };
  };
</script>

<script lang="ts">
  import { PluginState } from "@generated/host/model";
  import { RotateCw, Settings2, Trash2 } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Badge from "../../../components/ui/Badge.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import SortableTableHeader from "../../../components/ui/SortableTableHeader.svelte";
  import Switch from "../../../components/ui/Switch.svelte";
  import TablePanel from "../../../components/ui/TablePanel.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../../components/ui/table-order";
  import { activateTableRow } from "../../../components/ui/table-row";
  import type { ManagedPlugin } from "../../../lib/api/native-plugins";
  import type { NotificationFailure } from "../../../lib/notification-details";
  import { pluginStartFailure } from "./plugin-queries";
  import { entryPlacement, entrySource, statusPresentation } from "./plugin-rows";

  let {
    actions,
    failure,
    loading,
    entries,
    programs = [],
    generation,
    reloading,
    removingId,
    changingEnabled,
    onReload,
    onRemove,
    onSetEnabled,
    openSettings,
    settingsSectionOf,
  }: {
    actions?: Snippet;
    failure?: Snippet;
    loading?: boolean;
    entries: ManagedPlugin[];
    programs?: PluginRow[];
    generation: string;
    reloading: boolean;
    removingId: string | null;
    changingEnabled: boolean;
    onReload: (id: string) => void;
    onRemove: (entry: ManagedPlugin) => void;
    onSetEnabled: (entry: ManagedPlugin, enabled: boolean) => void;
    openSettings: (id: string) => void;
    settingsSectionOf: (entry: ManagedPlugin) => string | null;
  } = $props();
  const sortableColumns = [
    { key: "name", label: "Plugin", width: "pl-4" },
    { key: "status", label: "Status", width: "w-48" },
    { key: "placement", label: "Placement", width: "w-28" },
    { key: "source", label: "Source", width: "w-[34%]" },
  ] as const;
  type Column = (typeof sortableColumns)[number]["key"];
  let order = $state.raw<TableOrder<Column>>();
  const rows = $derived(
    entries.map((entry): PluginRow => {
      const status = statusPresentation(entry);
      const section = settingsSectionOf(entry);
      return {
        key: entry.name,
        name: entry.description?.id || entry.name,
        source: entrySource(entry),
        placement: entryPlacement(entry),
        status: status.label,
        badge: status.variant,
        enabled: entry.value
          ? { checked: entry.value.enabled, disabled: changingEnabled, onChange: (next) => onSetEnabled(entry, next) }
          : undefined,
        onOpenSettings: section ? () => openSettings(section) : undefined,
        reload: {
          disabled: !entry.value?.enabled || entry.state === PluginState.STOPPING || reloading,
          select: () => onReload(entry.name),
        },
        remove: { disabled: removingId === entry.name, select: () => onRemove(entry) },
        failure: entry.failure ? pluginStartFailure(entry, generation) : undefined,
      };
    }),
  );
  const plugins = $derived(orderTableRows([...programs, ...rows], order));
</script>

<TablePanel
  title="Plugins"
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
      <td class="px-2">
        {#if row.badge}
          <Badge variant={row.badge}>{row.status}</Badge>
        {/if}
      </td>
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
          {#if row.reload}
            <Button
              aria-label={`Reload ${row.name}`}
              title="Reload"
              size="icon-sm"
              variant="ghost"
              disabled={row.reload.disabled}
              onclick={row.reload.select}
            >
              <RotateCw />
            </Button>
          {/if}
          {#if row.remove}
            <Button
              aria-label={`Delete ${row.name}`}
              title="Delete"
              size="icon-sm"
              variant="ghost"
              disabled={row.remove.disabled}
              onclick={row.remove.select}
            >
              <Trash2 />
            </Button>
          {/if}
        </div>
      </td>
    </tr>
    {#if row.failure}
      <tr>
        <td colspan="6" class="px-4 py-3">
          <ErrorCard
            failure={row.failure}
            onRetry={row.enabled?.checked ? row.reload?.select : undefined}
            retryLabel="Restart Plugin"
            retryDisabled={reloading}
          />
        </td>
      </tr>
    {/if}
  {/each}
</TablePanel>
