/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Remote Listener Settings And Pairing Files
 */

/* === Imports === */

// Standard Library Imports

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Ban } from "lucide-react";
import { useEffect, useState } from "react";
// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
// Third Party Imports
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { RemoteSessionsCard } from "@/features/settings/sections/RemoteSessionsCard";
import {
  commitListener,
  type ListenerChange,
  type ListenerStatus,
  listenerError,
  listenerRunning,
} from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";
import { isTauriDesktop } from "@/lib/desktop-open";
import { toast } from "@/lib/notifications";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

export function RemoteSection({ advanced = false }: { advanced?: boolean }) {
  const session = useNativeSessionOptional();
  const [preferencesPending, setPreferencesPending] = useState(false);
  const canManage = session?.snapshot.connection.backend?.methods.includes("/dokime.v1.Remote/ReadListener") ?? false;
  if (!isTauriDesktop() || !session || !canManage) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Host Settings">
          <p className="text-sm text-muted-foreground">
            Open Remote Access in the host's connected desktop session to configure its listener.
          </p>
        </SettingsRow>
      </SettingsCard>
    );
  }
  return (
    <>
      <ApplicationPreferences section="remote" advanced={advanced} onPendingChange={setPreferencesPending} />
      <ConnectedRemoteSection
        key={session.identity.generation}
        session={session}
        preferencesPending={preferencesPending}
        advanced={advanced}
      />
    </>
  );
}

function ConnectedRemoteSection({
  session,
  preferencesPending,
  advanced,
}: {
  session: ConnectedSession;
  preferencesPending: boolean;
  advanced: boolean;
}) {
  const queryKey = ["settings", "remote-listener", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => rpc.remoteReadListener(session.identity, {}), retry: false });
  if (query.isPending) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Remote Settings">
          <p role="status" className="text-sm text-muted-foreground">
            Loading remote settings…
          </p>
        </SettingsRow>
      </SettingsCard>
    );
  }
  if (query.error || !query.data.settings) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Remote Settings" error={message(query.error)}>
          <Button variant="outline" onClick={() => void query.refetch()}>
            Try Again
          </Button>
        </SettingsRow>
      </SettingsCard>
    );
  }
  return <ListenerControls session={session} status={query.data} preferencesPending={preferencesPending} advanced={advanced} />;
}

function ListenerControls({
  session,
  status,
  preferencesPending,
  advanced,
}: {
  session: ConnectedSession;
  status: ListenerStatus;
  preferencesPending: boolean;
  advanced: boolean;
}) {
  const client = useQueryClient();
  const queryKey = ["settings", "remote-listener", session.identity.generation];
  const [busy, setBusy] = useState(false);
  const [clientsRevision, setClientsRevision] = useState(0);
  const [pairingError, setPairingError] = useState<string | null>(null);
  const settings = status.settings;
  if (!settings) throw new Error("Backend omitted listener settings.");
  const running = listenerRunning(status);

  const publish = (updated: ListenerStatus) => {
    if (!session.identity.isCurrent()) stale();
    client.setQueryData(queryKey, updated);
    return updated;
  };
  const fail = (title: string, description: string) => {
    toast.error(title, { description });
    void client.invalidateQueries({ queryKey });
  };
  const run = async <T,>(work: () => Promise<T>): Promise<T> => {
    setBusy(true);
    try {
      return await work();
    } finally {
      setBusy(false);
    }
  };

  const commit = (change: ListenerChange) =>
    run(async () => {
      const next = await commitListener({
        status,
        change,
        send: async (request) => publish(await rpc.remoteUpdateListener(session.identity, { settings: request })),
        fail: (description) => fail("Remote Access Change Failed", description),
      });
      return next !== status;
    });

  async function createPairing() {
    setPairingError(null);
    await run(async () => {
      try {
        const path = await commands.desktopPairingCreate();
        if (!session.identity.isCurrent()) stale();
        if (path !== null) {
          toast.success("Remote Access File Saved", { description: path });
          void client.invalidateQueries({ queryKey });
          setClientsRevision((revision) => revision + 1);
        }
      } catch (error) {
        const reason = listenerError(error);
        setPairingError(reason);
        fail("Remote Access File Not Saved", reason);
        setClientsRevision((revision) => revision + 1);
      }
    });
  }

  return (
    <>
      <SettingsCard
        title="Effective Listener State"
        footer={
          <Button variant="outline" disabled={busy || preferencesPending} onClick={() => void commit({})}>
            Restart Listener
          </Button>
        }
      >
        <SettingsRow label="Status" description="Saved preferences take effect when the listener restarts." error={status.error}>
          <span role="status">{running ? status.listeningAddress : "Stopped"}</span>
        </SettingsRow>
      </SettingsCard>
      {advanced && (
        <SettingsCard title="Certificate">
          <SettingsRow
            label="Certificate Identity"
            description="Certificate names refresh automatically. Reissuing preserves the CA trusted by existing remote files."
          >
            <Button
              variant="outline"
              disabled={busy || preferencesPending}
              onClick={() =>
                void run(async () => {
                  try {
                    publish(await rpc.remoteRegenerateCertificate(session.identity, { certificateNames: settings.certificateNames }));
                  } catch (error) {
                    fail("Certificate Generation Failed", listenerError(error));
                  }
                })
              }
            >
              Regenerate Certificate
            </Button>
          </SettingsRow>
        </SettingsCard>
      )}
      <SettingsCard
        title="Remote Access Files"
        description="Export a separate secret file for each client. Anyone with the file has full control; transfer it privately."
      >
        <div className="grid gap-3 p-4">
          <p className="text-sm text-muted-foreground">
            Endpoints are detected automatically. Loopback addresses (127.0.0.1, ::1) work on the same PC only. Rename client labels later
            in Paired Clients.
          </p>
          {pairingError && (
            <p role="alert" className="text-sm text-destructive">
              {pairingError}
            </p>
          )}
          <Button disabled={busy || preferencesPending} onClick={() => void createPairing()}>
            Export Remote Access
          </Button>
        </div>
      </SettingsCard>
      <PairedClients key={clientsRevision} session={session} />
      {session.snapshot.connection.backend?.methods.includes("/dokime.v1.Remote/ListSessions") ? (
        <RemoteSessionsCard session={session} />
      ) : null}
    </>
  );
}

function PairedClients({ session }: { session: ConnectedSession }) {
  type Page = Wire<Awaited<ReturnType<typeof rpc.remoteListPairedClients>>>;
  type Client = Page["clients"][number];
  const [page, setPage] = useState<Page | null>(null);
  const [request, setRequest] = useState<{ cursor?: string }>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [renaming, setRenaming] = useState<Client | null>(null);
  const [name, setName] = useState("");
  useEffect(() => {
    let active = true;
    setBusy(true);
    setPage(null);
    setError(null);
    void rpc
      .remoteListPairedClients(session.identity, { cursor: request.cursor, limit: 100 })
      .then((next) => {
        if (active && session.identity.isCurrent()) setPage(next);
      })
      .catch((failure: unknown) => {
        if (active) setError(listenerError(failure));
      })
      .finally(() => {
        if (active) setBusy(false);
      });
    return () => {
      active = false;
    };
  }, [session, request]);
  async function revoke(identifier: string) {
    setBusy(true);
    setError(null);
    try {
      await rpc.remoteRevokePairedClient(session.identity, { identifier });
      if (!session.identity.isCurrent()) return;
      setRequest((value) => ({ ...value }));
    } catch (failure) {
      setError(listenerError(failure));
    } finally {
      setBusy(false);
    }
  }
  const columns = dataTableColumns<Client>();
  return (
    <SettingsCard>
      <DataTable
        aria-label="Paired Clients"
        title="Paired Clients"
        data={page?.clients ?? []}
        getRowId={(client) => client.identifier}
        emptyText={busy ? "Loading Paired Clients…" : "No Paired Clients"}
        columns={columns.columns([
          columns.accessor("label", {
            header: "Label",
            cell: ({ row }) => (
              <Button
                variant="ghost"
                disabled={busy}
                onClick={() => {
                  setRenaming(row.original);
                  setName(row.original.label);
                }}
              >
                {row.original.label}
              </Button>
            ),
          }),
          columns.accessor((client) => new Date(Number(client.createdAt)).toLocaleString(), { id: "created", header: "Created" }),
          columns.accessor((client) => (client.lastSeenAt == null ? "—" : new Date(Number(client.lastSeenAt)).toLocaleString()), {
            id: "lastSeen",
            header: "Last Seen",
          }),
          columns.display({
            id: "actions",
            header: "Actions",
            cell: ({ row }) => (
              <DataTableRowActions
                label={row.original.label}
                primary={{ label: "Revoke", icon: Ban, disabled: busy, onSelect: () => void revoke(row.original.identifier) }}
              />
            ),
          }),
        ])}
        actions={
          <Button disabled={busy} variant="outline" onClick={() => setRequest((value) => ({ ...value }))}>
            Refresh
          </Button>
        }
      />
      {renaming && (
        <form
          className="flex gap-2 p-4"
          onSubmit={(event) => {
            event.preventDefault();
            setBusy(true);
            setError(null);
            void rpc
              .remoteRenamePairedClient(session.identity, { identifier: renaming.identifier, label: name })
              .then(() => {
                setRenaming(null);
                setRequest((value) => ({ ...value }));
              })
              .catch((failure: unknown) => setError(listenerError(failure)))
              .finally(() => setBusy(false));
          }}
        >
          <Input
            aria-label="Client Label"
            required
            maxLength={256}
            value={name}
            disabled={busy}
            onChange={(event) => setName(event.target.value)}
          />
          <Button type="submit" disabled={busy}>
            Rename
          </Button>
          <Button type="button" variant="outline" disabled={busy} onClick={() => setRenaming(null)}>
            Cancel
          </Button>
        </form>
      )}
      {error && <NotificationMessage>{error}</NotificationMessage>}
      <div className="flex gap-2 p-4">
        <Button disabled={busy || !request.cursor} variant="outline" onClick={() => setRequest({})}>
          First Page
        </Button>
        <Button disabled={busy || !page?.nextCursor} variant="outline" onClick={() => setRequest({ cursor: page?.nextCursor })}>
          Next Page
        </Button>
      </div>
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function message(error: unknown): string {
  return error instanceof Error ? error.message : "Remote access settings are unavailable. Reconnect to the host and try again.";
}

function stale(): never {
  throw new Error("Backend connection changed; reconnect to check the remote settings.");
}
