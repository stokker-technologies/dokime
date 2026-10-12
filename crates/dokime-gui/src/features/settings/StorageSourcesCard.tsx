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
 * One aligned grid, not a table: `Source | State | Priority` on the same row
 * height as every other settings row. The switch writes the moment it moves
 * and the arrow pair writes the moment it is pressed; the card states the
 * outcome once rather than once per row.
 */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, ChevronUp } from "lucide-react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import {
  fetchStorageSources,
  reorderStorageSources,
  setStorageSourceEnabled,
} from "@/lib/api/native-storage-settings";
import { cn } from "@/lib/utils";

/* === Constants === */

/**
 * The storage columns, shared by the header row and every source.
 *
 * The column count is the source count's business: one source has no priority
 * to argue about, so the pair is not drawn and the column it would take does
 * not exist. A placeholder cell is not an option here — `display: contents`
 * puts every cell on this one grid and auto-placement flows straight through
 * the row boundaries, so a header with three cells over rows with two (or a
 * screen-reader-only cell, which is absolutely positioned and therefore skipped
 * by placement) walks the whole list one column out of line.
 */
const TABLE_CLASS = "grid gap-x-6 text-left text-sm";

const COLUMNS: Record<"one" | "many", string> = {
  many: "minmax(0,1fr) auto auto",
  one: "minmax(0,1fr) auto",
};

const CELL_CLASS = "flex min-w-0 items-center";

const HEAD_CLASS = "pb-1 text-xs tracking-wide text-muted-foreground uppercase";

/** 40px rows, 48px at touch, cells centred on them. */
const ROW_HEIGHT = "min-h-[var(--settings-row-height)]";

/* === Types === */

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
  const sources = query.data ?? [];
  const error = mutation.error ?? query.error;
  // The autosave readout belongs beside the control that moved, so the row
  // remembers which source the last write was about.
  const change = mutation.variables;
  const changedIdentifier = change?.kind === "enabled" ? change.identifier : null;
  /** More than one source is what makes priority a question worth a column. */
  const ranked = sources.length > 1;

  function move(index: number, direction: number) {
    const identifiers = sources.map((source) => source.identifier);
    const target = index + direction;
    if (target < 0 || target >= identifiers.length || mutation.isPending) return;
    [identifiers[index], identifiers[target]] = [identifiers[target], identifiers[index]];
    mutation.mutate({ kind: "order", identifiers });
  }

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
        <SettingsRow label="Status">
          <span className="text-sm text-destructive" role="alert">
            {error instanceof Error ? error.message : String(error)}
          </span>
          {query.isError ? (
            <Button onClick={() => void query.refetch()} size="sm" variant="outline">
              Try Again
            </Button>
          ) : null}
        </SettingsRow>
      ) : null}
      {query.isSuccess && sources.length === 0 ? (
        <SettingsRow label="Sources">
          <span className="text-sm text-muted-foreground">No storage sources are configured.</span>
        </SettingsRow>
      ) : null}

      {sources.length > 0 ? (
        /* A real table laid out as one grid: `display: contents` on the row
           groups puts every cell on the table's own columns, so the three
           columns line up down the list without losing table semantics. */
        <table
          aria-label="Storage source priority"
          className={cn(TABLE_CLASS, "px-4 [[data-density=touch]_&]:px-6")}
          style={{ gridTemplateColumns: COLUMNS[ranked ? "many" : "one"] }}
        >
          <thead className="contents">
            <tr className="contents">
              <th className={cn(CELL_CLASS, HEAD_CLASS)} scope="col">
                Source
              </th>
              <th className={cn(CELL_CLASS, HEAD_CLASS)} scope="col">
                State
              </th>
              {ranked ? (
                <th className={cn(CELL_CLASS, HEAD_CLASS)} scope="col">
                  Priority
                </th>
              ) : null}
            </tr>
          </thead>
          <tbody className="contents">
            {sources.map((source, index) => {
              const label = sourceLabel(source.identifier);
              const locked = source.identifier === "local" || !canEnable;
              return (
                <tr className="contents" key={source.identifier}>
                  <th className={cn(CELL_CLASS, ROW_HEIGHT, "font-medium")} scope="row">
                    <span className="truncate">{label}</span>
                  </th>
                  <td className={cn(CELL_CLASS, ROW_HEIGHT, "gap-2")}>
                    <Switch
                      aria-label={`${source.enabled ? "Disable" : "Enable"} ${label}`}
                      checked={source.enabled}
                      disabled={locked || mutation.isPending}
                      onCheckedChange={(enabled) =>
                        mutation.mutate({ kind: "enabled", identifier: source.identifier, enabled })
                      }
                    />
                    <span className="text-muted-foreground">
                      {source.enabled ? "Enabled" : "Disabled"}
                    </span>
                    {changedIdentifier === source.identifier ? (
                      <span
                        className="text-xs text-muted-foreground"
                        role={mutation.isPending ? "status" : undefined}
                      >
                        {mutation.isPending ? "Saving" : mutation.isSuccess ? "Saved" : null}
                      </span>
                    ) : null}
                  </td>
                  {/* One source has no priority to argue about, so neither the
                      pair nor the column it would sit in is drawn. */}
                  {ranked ? (
                    <td className={cn(CELL_CLASS, ROW_HEIGHT, "gap-1")}>
                      <Button
                        aria-label={`Move ${label} Up`}
                        disabled={!canReorder || mutation.isPending || index === 0}
                        onClick={() => move(index, -1)}
                        size="icon-sm"
                        variant="outline"
                      >
                        <ChevronUp />
                      </Button>
                      <Button
                        aria-label={`Move ${label} Down`}
                        disabled={!canReorder || mutation.isPending || index === sources.length - 1}
                        onClick={() => move(index, 1)}
                        size="icon-sm"
                        variant="outline"
                      >
                        <ChevronDown />
                      </Button>
                    </td>
                  ) : null}
                </tr>
              );
            })}
          </tbody>
        </table>
      ) : null}

      {/* An unavailable capability is a note, not a row with an empty value. */}
      <p className="px-4 py-3 text-xs text-muted-foreground [[data-density=touch]_&]:px-6">
        Remote synchronization is not available yet.
      </p>
    </SettingsCard>
  );
}

/* === Private Functions === */

function sourceLabel(identifier: string): string {
  return identifier === "local" ? "Local" : identifier === "fabric" ? "Fabric" : identifier;
}
