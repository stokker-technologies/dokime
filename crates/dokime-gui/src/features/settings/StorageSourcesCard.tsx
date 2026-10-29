/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Storage sources, in priority order.
 *
 * The same table as every other list: `Source | State | Priority`, the state
 * beside its source rather than across the card from it. The switch writes
 * the moment it moves and the arrow pair writes the moment it is pressed; the
 * card states the outcome once rather than once per row.
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowUp } from "lucide-react";
import { useMemo } from "react";
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Switch } from "@/components/ui/switch";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { fetchStorageSources, reorderStorageSources, setStorageSourceEnabled } from "@/lib/api/native-storage-settings";

const NO_SOURCES: StorageSource[] = [];

type StorageSource = Awaited<ReturnType<typeof fetchStorageSources>>[number];

type Change = { kind: "enabled"; identifier: string; enabled: boolean } | { kind: "order"; identifiers: string[] };

export function StorageSourcesCard() {
  const session = useNativeSessionOptional();
  return session ? <ConnectedStorageSources key={session.identity.generation} session={session} /> : null;
}

function ConnectedStorageSources({ session }: { session: ConnectedSession }) {
  const queryClient = useQueryClient();
  const queryKey = ["settings", "storage-sources", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => fetchStorageSources(session.identity) });
  const mutation = useMutation({
    mutationFn: async (change: Change) => {
      await queryClient.cancelQueries({ queryKey });
      return change.kind === "order"
        ? reorderStorageSources(session.identity, change.identifiers)
        : setStorageSourceEnabled(session.identity, change.identifier, change.enabled);
    },
    onSuccess: (sources) => {
      if (session.identity.isCurrent()) queryClient.setQueryData(queryKey, sources);
    },
  });
  const methods = session.snapshot.connection.backend?.methods ?? [];
  const canReorder = methods.includes("/dokime.v1.Settings/ReorderStorageSources");
  const canEnable = methods.includes("/dokime.v1.Settings/SetStorageSourceEnabled");
  const sources = query.data ?? NO_SOURCES;
  const error = mutation.error ?? query.error;
  // The autosave readout belongs beside the control that moved, so the row
  // remembers which source the last write was about.
  const change = mutation.variables;
  const changedIdentifier = change?.kind === "enabled" ? change.identifier : null;

  const { isPending, isSuccess, mutate } = mutation;

  // Columns: the order is the data, so nothing here sorts.
  const columns = useMemo(() => {
    const column = dataTableColumns<StorageSource>();

    function move(index: number, direction: number) {
      const identifiers = sources.map((source) => source.identifier);
      const target = index + direction;
      if (target < 0 || target >= identifiers.length || isPending) return;
      [identifiers[index], identifiers[target]] = [identifiers[target], identifiers[index]];
      mutate({ kind: "order", identifiers });
    }

    const source = column.accessor((row) => sourceLabel(row.identifier), {
      cell: ({ getValue }) => <span className="font-medium">{getValue()}</span>,
      header: "Source",
      id: "source",
      meta: { label: "Source", truncate: true },
    });
    const state = column.display({
      cell: ({ row }) => {
        const label = sourceLabel(row.original.identifier);
        const locked = row.original.identifier === "local" || !canEnable;
        return (
          <div className="flex items-center gap-2">
            <Switch
              aria-label={`${row.original.enabled ? "Disable" : "Enable"} ${label}`}
              checked={row.original.enabled}
              disabled={locked || isPending}
              onCheckedChange={(enabled) => mutate({ kind: "enabled", identifier: row.original.identifier, enabled })}
            />
            <span className="text-muted-foreground">{row.original.enabled ? "Enabled" : "Disabled"}</span>
            {changedIdentifier === row.original.identifier ? (
              <span className="text-xs text-muted-foreground" role={isPending ? "status" : undefined}>
                {isPending ? "Saving" : isSuccess ? "Saved" : null}
              </span>
            ) : null}
          </div>
        );
      },
      header: "State",
      id: "state",
      meta: { label: "State", width: "w-48" },
    });
    const priority = column.accessor((source) => sources.findIndex((item) => item.identifier === source.identifier) + 1, {
      id: "priority",
      header: "Priority",
      meta: { width: "w-24" },
    });
    const actions = column.display({
      id: "actions",
      header: "Actions",
      cell: ({ row }) => {
        const index = sources.findIndex((source) => source.identifier === row.original.identifier);
        return (
          <DataTableRowActions
            label={sourceLabel(row.original.identifier)}
            primary={{
              label: "Move Up",
              icon: ArrowUp,
              disabled: !canReorder || isPending || index === 0,
              onSelect: () => move(index, -1),
            }}
          >
            <DropdownMenuItem disabled={!canReorder || isPending || index === sources.length - 1} onSelect={() => move(index, 1)}>
              Move Down
            </DropdownMenuItem>
          </DataTableRowActions>
        );
      },
    });
    return [source, state, priority, actions] as DataTableColumns<StorageSource>;
  }, [canEnable, canReorder, changedIdentifier, isPending, isSuccess, mutate, sources]);

  return (
    <SettingsCard>
      <DataTable
        aria-label="Storage Source Priority"
        columns={columns}
        data={sources}
        description="Higher rows have priority when resolving conflicting edits. Changing priority does not move data."
        emptyText="No Storage Sources Are Configured"
        getRowId={sourceId}
        title="Storage Sources"
      />
      {query.isPending ? (
        <SettingsRow label="Status">
          <NotificationMessage tone="info">Loading storage sources…</NotificationMessage>
        </SettingsRow>
      ) : null}
      {error ? (
        <SettingsRow
          label="Status"
          status={
            query.isError ? (
              <Button onClick={() => void query.refetch()} size="sm" variant="outline">
                Try Again
              </Button>
            ) : null
          }
        >
          <NotificationMessage>{error instanceof Error ? error.message : String(error)}</NotificationMessage>
        </SettingsRow>
      ) : null}

      {/* An unavailable feature is a note, not a row with an empty value. */}
      <p className="px-4 py-3 text-xs text-muted-foreground [[data-density=touch]_&]:px-6">Remote synchronization is not available yet.</p>
    </SettingsCard>
  );
}

function sourceId(source: StorageSource): string {
  return source.identifier;
}

function sourceLabel(identifier: string): string {
  return identifier === "local" ? "Local" : identifier === "fabric" ? "Fabric" : identifier;
}
