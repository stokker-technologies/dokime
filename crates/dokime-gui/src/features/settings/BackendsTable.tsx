/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Backend Registry
 */

/* === Imports === */

import { failureOf } from "@/lib/notification-details";
import type { Document, Profile } from "@generated/application-settings";
import { Plug } from "lucide-react";
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { useNativeSessionController } from "@/app/native-session/context";
import { connectionError } from "@/app/native-session/errors";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { commands } from "@/lib/desktop/commands";
import { preferencesCommands } from "@/lib/desktop/settings";
import { SettingsCard } from "./SettingsCard";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

export function BackendsTable({
  failure,
  retry,
  endpoint,
  onConnect,
}: {
  failure?: ReactNode;
  retry?: (profile: string | null) => void;
  endpoint?: string | null;
  onConnect?: () => void;
} = {}) {
  const owner = useNativeSessionController();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const lastSelected = useRef<string | null>(null);
  const importButton = useRef<HTMLButtonElement>(null);
  const deleteButton = useRef<HTMLButtonElement>(null);
  const [document, setDocument] = useState<Document | null>(null);
  const [error, setError] = useState<unknown>();
  const [deleting, setDeleting] = useState<string | null>(null);
  const [mutating, setBusy] = useState(false);
  const deletingProfile = document?.profiles.find((profile) => profile.identifier === deleting);
  const busy = mutating || state.phase === "connecting";
  const connectionFailure = failure ?? (state.phase === "failed" ? connectionError(state.error) : null);
  const failedProfile = document?.profiles.find((profile) => profile.identifier === lastSelected.current);
  useEffect(() => {
    let active = true;
    void preferencesCommands
      .desktopBackendsList()
      .then((value) => {
        if (active) setDocument(value);
      })
      .catch((failure: unknown) => {
        if (active) setError(failure);
      });
    return () => {
      active = false;
    };
  }, []);
  async function remove() {
    if (!deleting) return;
    setBusy(true);
    setError(undefined);
    try {
      setDocument(await preferencesCommands.desktopBackendRemove(deleting));
      setDeleting(null);
      const status = await commands.desktopSessionStatus();
      if (status.connection.phase === "disconnected") await owner.disconnect();
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  }
  const connect = useCallback(
    (identifier: string) => {
      if (busy) return;
      onConnect?.();
      lastSelected.current = identifier;
      void owner.select({ kind: "profile", identifier });
    },
    [busy, onConnect, owner],
  );
  const columns = useMemo(
    () =>
      dataTableColumns<Profile>().columns([
        dataTableColumns<Profile>().accessor("identifier", { header: "Identifier", meta: { primary: true, filter: "text" } }),
        dataTableColumns<Profile>().accessor("label", { header: "Label", meta: { filter: "text" } }),
        dataTableColumns<Profile>().accessor("endpoint", {
          header: "Endpoint",
          meta: { filter: "text", truncate: true },
        }),
        dataTableColumns<Profile>().display({
          id: "actions",
          header: "Actions",
          cell: ({ row }) => (
            <DataTableRowActions
              label={row.original.identifier}
              primary={{ label: "Connect", icon: Plug, disabled: busy, onSelect: () => connect(row.original.identifier) }}
            >
              <DropdownMenuItem variant="destructive" disabled={busy} onSelect={() => setDeleting(row.original.identifier)}>
                Delete
              </DropdownMenuItem>
            </DataTableRowActions>
          ),
        }),
      ]),
    [busy, connect],
  );
  return (
    <SettingsCard className="w-full text-left">
      {(connectionFailure || Boolean(error)) && (
        <div className="grid gap-3 bg-destructive/5 p-4 text-sm">
          <NotificationMessage
            source="Backends"
            title={connectionFailure ? "Backend Connection Failed" : "Backend Registry Failed"}
            error={state.phase === "failed" ? state.error : error}
          >
            {connectionFailure || failureOf(error, "Backend Operation Failed").message}
            {connectionFailure && (endpoint || failedProfile) && (
              <p className="mt-2 break-all font-mono text-xs">{endpoint || failedProfile?.endpoint}</p>
            )}
          </NotificationMessage>
          <Button
            className="justify-self-start"
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => {
              if (connectionFailure) {
                if (retry) retry(lastSelected.current);
                else if (lastSelected.current) connect(lastSelected.current);
                return;
              }
              setError(undefined);
              void preferencesCommands.desktopBackendsList().then(setDocument).catch(setError);
            }}
          >
            Retry
          </Button>
        </div>
      )}
      {!document && !error && !connectionFailure && (
        <p role="status" className="p-4 text-sm text-muted-foreground">
          Loading backends…
        </p>
      )}
      <DataTable
        aria-label="Saved Backends"
        data={document?.profiles ?? []}
        getRowId={(profile) => profile.identifier}
        emptyText="No Saved Backends"
        columns={columns}
        onRowClick={(profile) => connect(profile.identifier)}
        actions={
          <>
            <Button
              ref={importButton}
              size="sm"
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                setError(undefined);
                try {
                  const imported = await preferencesCommands.desktopBackendImportPairing(null);
                  if (imported) {
                    setDocument(imported);
                    onConnect?.();
                  }
                } catch (failure) {
                  setError(failure);
                } finally {
                  setBusy(false);
                }
              }}
            >
              Import Remote
            </Button>
          </>
        }
      />
      <Dialog
        open={deleting !== null}
        onOpenChange={(open) => {
          if (!open && !busy) setDeleting(null);
        }}
      >
        <DialogContent
          onOpenAutoFocus={(event) => {
            event.preventDefault();
            deleteButton.current?.focus();
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            importButton.current?.focus();
          }}
        >
          <DialogHeader>
            <DialogTitle>Delete Backend</DialogTitle>
            <DialogDescription>
              Delete {deletingProfile?.label || deleting}? An active connection will disconnect.
              {deletingProfile && <span className="mt-2 block break-all font-mono text-xs">{deletingProfile.identifier}</span>}
            </DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button ref={deleteButton} disabled={busy} variant="outline" onClick={() => setDeleting(null)}>
              Cancel
            </Button>
            <Button disabled={busy} variant="destructive" onClick={() => void remove()}>
              Confirm Delete
            </Button>
          </div>
          {Boolean(error) && (
            <NotificationMessage source="Backends" title="Backend Could Not Be Deleted" error={error}>
              {failureOf(error, "Backend Operation Failed").message}
            </NotificationMessage>
          )}
        </DialogContent>
      </Dialog>
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
