/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One table shape for every plugin the Plugins page lists.
 *
 * An installed bundle and a program the host or the launch configuration
 * brought in are the same columns — name, source, enabled, settings, actions —
 * at the same widths, so the page reads as one list rather than as differently
 * shaped ones. The row itself opens the plugin's page; the switch and the
 * settings button keep their own hit areas inside it.
 */

import { Settings2 } from "lucide-react";
import type { ReactNode } from "react";

import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Switch } from "@/components/ui/switch";

export type PluginTableRow = {
  /** Rendered under the name when the plugin reports a problem. */
  alert?: ReactNode;
  /** Trailing content on the name column, such as a state pill. */
  badge?: ReactNode;
  /** Absent when this plugin cannot be enabled or disabled here. */
  enabled?: { checked: boolean; disabled?: boolean; onChange: (next: boolean) => void };
  key: string;
  name: string;
  /** Opens the plugin's own settings page; absent when it contributes none. */
  onOpenSettings?: () => void;
  /** Where the plugin came from, with its version. One muted line. */
  source: string;
  /** The row's own actions, on the shared trailing column. */
  trailing?: ReactNode;
};

const column = dataTableColumns<PluginTableRow>();

const COLUMNS: DataTableColumns<PluginTableRow> = column.columns([
  column.accessor("name", {
    cell: ({ row }) => (
      <div className="grid min-w-0 gap-1">
        <div className="flex min-w-0 items-center gap-2">
          <span className="truncate text-sm font-medium" title={row.original.name}>
            {row.original.name}
          </span>
          {row.original.badge}
        </div>
        {row.original.alert}
      </div>
    ),
    header: "Plugin",
    meta: { filter: "text", label: "Plugin" },
  }),
  column.accessor("source", {
    cell: ({ getValue }) => <span className="text-xs text-muted-foreground">{getValue()}</span>,
    header: "Source",
    meta: { label: "Source", truncate: true, width: "w-[34%]" },
  }),
  column.display({
    cell: ({ row }) => {
      const enabled = row.original.enabled;
      return enabled ? (
        <Switch
          aria-label={`${enabled.checked ? "Disable" : "Enable"} ${row.original.name}`}
          checked={enabled.checked}
          disabled={enabled.disabled}
          onCheckedChange={enabled.onChange}
        />
      ) : null;
    },
    header: "Enabled",
    id: "enabled",
    meta: { label: "Enabled", width: "w-24" },
  }),
  column.display({
    id: "actions",
    header: "Actions",
    cell: ({ row }) => (
      <DataTableRowActions
        label={row.original.name}
        primary={{
          label: "Settings",
          icon: Settings2,
          disabled: !row.original.onOpenSettings,
          onSelect: () => row.original.onOpenSettings?.(),
        }}
      >
        {row.original.trailing}
      </DataTableRowActions>
    ),
  }),
]);

export function PluginTable({ label, rows, actions }: { label: string; rows: PluginTableRow[]; actions?: ReactNode }) {
  return (
    <DataTable
      actions={actions}
      aria-label={label}
      columns={COLUMNS}
      data={rows}
      emptyText="No Plugins"
      getRowId={rowKey}
      onRowClick={(row) => row.onOpenSettings?.()}
    />
  );
}

function rowKey(row: PluginTableRow): string {
  return row.key;
}
