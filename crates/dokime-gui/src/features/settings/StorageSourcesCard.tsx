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

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useMemo } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import type { ConnectedSession } from "@/app/native-session/session";
import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import {
  fetchStorageSources,
  reorderStorageSources,
  setStorageSourceEnabled,
} from "@/lib/api/native-storage-settings";

/* === Constants === */

const NO_SOURCES: StorageSource[] = [];

/* === Types === */

type StorageSource = Awaited<ReturnType<typeof fetchStorageSources>>[number];

type Change =
  | { kind: "enabled"; identifier: string; enabled: boolean }
  | { kind: "order"; identifiers: string[] };

/* === Components === */

export function StorageSourcesCard() {
  const session = useNativeSessionOptional();
  return session ? (
    <ConnectedStorageSources key={session.identity.generation} session={session} />
  ) : null;
}

/* === Private Components === */

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
      enableSorting: false,
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
              onCheckedChange={(enabled) =>
                mutate({ kind: "enabled", identifier: row.original.identifier, enabled })
              }
            />
            <span className="text-muted-foreground">
              {row.original.enabled ? "Enabled" : "Disabled"}
            </span>
            {changedIdentifier === row.original.identifier ? (
              <span
                className="text-xs text-muted-foreground"
                role={isPending ? "status" : undefined}
              >
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
    // One source has no priority to argue about, so neither the pair nor its
    // column is drawn.
    const priority = column.display({
      cell: ({ row }) => {
        const label = sourceLabel(row.original.identifier);
        return (
          <div className="flex items-center gap-1">
            <Button
              aria-label={`Move ${label} Up`}
              disabled={!canReorder || isPending || row.index === 0}
              onClick={() => move(row.index, -1)}
              size="icon-sm"
              variant="outline"
            >
              <ChevronUp />
            </Button>
            <Button
              aria-label={`Move ${label} Down`}
              disabled={!canReorder || isPending || row.index === sources.length - 1}
              onClick={() => move(row.index, 1)}
              size="icon-sm"
              variant="outline"
            >
              <ChevronDown />
            </Button>
          </div>
        );
      },
      header: "Priority",
      id: "priority",
      meta: { label: "Priority", width: "w-24" },
    });

    return (
      sources.length > 1 ? [source, state, priority] : [source, state]
    ) as DataTableColumns<StorageSource>;
  }, [canEnable, canReorder, changedIdentifier, isPending, isSuccess, mutate, sources]);

  return (
    <SettingsCard
      description="Higher rows have priority when resolving conflicting edits. Changing priority does not move data."
      title="Storage Sources"
    >
      {query.isPending ? (
        <SettingsRow label="Status">
          <span className="text-sm text-muted-foreground" role="status">
            Loading storage sources…
          </span>
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
          <span className="text-sm text-destructive" role="alert">
            {error instanceof Error ? error.message : String(error)}
          </span>
        </SettingsRow>
      ) : null}
      {query.isSuccess && sources.length === 0 ? (
        <SettingsRow label="Sources">
          <span className="text-sm text-muted-foreground">No storage sources are configured.</span>
        </SettingsRow>
      ) : null}

      {sources.length > 0 ? (
        <DataTable
          aria-label="Storage source priority"
          className="[--table-inset:1rem] [--table-surface:var(--card)] [[data-density=touch]_&]:[--table-inset:1.5rem]"
          columns={columns}
          data={sources}
          emptyText="No storage sources are configured."
          getRowId={sourceId}
          stickyHeader={false}
        />
      ) : null}

      {/* An unavailable feature is a note, not a row with an empty value. */}
      <p className="px-4 py-3 text-xs text-muted-foreground [[data-density=touch]_&]:px-6">
        Remote synchronization is not available yet.
      </p>
    </SettingsCard>
  );
}

/* === Private Functions === */

function sourceId(source: StorageSource): string {
  return source.identifier;
}

function sourceLabel(identifier: string): string {
  return identifier === "local" ? "Local" : identifier === "fabric" ? "Fabric" : identifier;
}
