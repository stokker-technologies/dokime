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

/* === External Imports === */

import { Settings2 } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";

/* === Types === */

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

/* === Constants === */

const column = dataTableColumns<PluginTableRow>();

const COLUMNS: DataTableColumns<PluginTableRow> = column.columns([
  column.accessor("name", {
    cell: ({ row }) => (
      <div className="grid min-w-0 gap-0.5">
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
    // A disabled control swallows pointer events, so the reason it is
    // disabled has to live on the wrapper.
    cell: ({ row }) => (
      <span className="flex items-center" title={row.original.onOpenSettings ? "Plugin Settings" : "No settings"}>
        <Button
          aria-label="Plugin Settings"
          disabled={!row.original.onOpenSettings}
          onClick={row.original.onOpenSettings}
          size="icon-sm"
          type="button"
          variant="ghost"
        >
          <Settings2 />
        </Button>
      </span>
    ),
    header: "Settings",
    id: "settings",
    meta: { label: "Settings", width: "w-24" },
  }),
  column.display({
    cell: ({ row }) => <span className="flex items-center justify-end gap-2">{row.original.trailing}</span>,
    header: () => <span className="sr-only">Actions</span>,
    id: "actions",
    meta: { align: "end", label: "Actions", width: "w-44" },
  }),
]);

/* === Components === */

export function PluginTable({ label, rows }: { label: string; rows: PluginTableRow[] }) {
  return (
    <div className="overflow-hidden rounded-lg border bg-card">
      <DataTable
        aria-label={label}
        className="[--table-inset:1rem] [--table-surface:var(--card)]"
        columns={COLUMNS}
        data={rows}
        emptyText="No plugins."
        getRowId={rowKey}
        onRowClick={(row) => row.onOpenSettings?.()}
        stickyHeader={false}
      />
    </div>
  );
}

/* === Private Functions === */

function rowKey(row: PluginTableRow): string {
  return row.key;
}
