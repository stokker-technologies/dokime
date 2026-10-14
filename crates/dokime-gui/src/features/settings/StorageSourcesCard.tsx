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

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import {
  fetchStorageSources,
  reorderStorageSources,
  setStorageSourceEnabled,
} from "@/lib/api/native-storage-settings";

/* === Constants === */

/** 40px rows, 48px at touch, cells centred on them. */
const ROW_HEIGHT = "h-[var(--settings-row-height)]";

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
        <div className="px-2 [[data-density=touch]_&]:px-4">
          <Table aria-label="Storage source priority">
            <TableHeader>
              <TableRow>
                <TableHead className="w-full">Source</TableHead>
                <TableHead>State</TableHead>
                {/* One source has no priority to argue about, so neither the
                    pair nor its column is drawn. */}
                {ranked ? <TableHead>Priority</TableHead> : null}
              </TableRow>
            </TableHeader>
            <TableBody>
              {sources.map((source, index) => {
                const label = sourceLabel(source.identifier);
                const locked = source.identifier === "local" || !canEnable;
                return (
                  <TableRow className={ROW_HEIGHT} key={source.identifier}>
                    <TableCell className="font-medium">{label}</TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Switch
                          aria-label={`${source.enabled ? "Disable" : "Enable"} ${label}`}
                          checked={source.enabled}
                          disabled={locked || mutation.isPending}
                          onCheckedChange={(enabled) =>
                            mutation.mutate({
                              kind: "enabled",
                              identifier: source.identifier,
                              enabled,
                            })
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
                      </div>
                    </TableCell>
                    {ranked ? (
                      <TableCell>
                        <div className="flex items-center gap-1">
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
                            disabled={
                              !canReorder || mutation.isPending || index === sources.length - 1
                            }
                            onClick={() => move(index, 1)}
                            size="icon-sm"
                            variant="outline"
                          >
                            <ChevronDown />
                          </Button>
                        </div>
                      </TableCell>
                    ) : null}
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
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
