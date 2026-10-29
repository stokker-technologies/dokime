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
import { useEffect, useId, useState } from "react";
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
          <NotificationMessage tone="info">
            Open Remote Access in the host's connected desktop session to configure its listener.
          </NotificationMessage>
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
          <NotificationMessage tone="info">Loading remote settings…</NotificationMessage>
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
  const [label, setLabel] = useState("");
  const fieldId = useId();
  const [endpoint, setEndpoint] = useState("");
  const [clientsRevision, setClientsRevision] = useState(0);
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
    await run(async () => {
      try {
        const path = await commands.desktopPairingCreate(label, endpoint);
        if (!session.identity.isCurrent()) stale();
        if (path !== null) {
          toast.success("Pairing File Saved", { description: path });
          setClientsRevision((revision) => revision + 1);
        }
      } catch (error) {
        fail("Pairing File Not Saved", listenerError(error));
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
            description="Changing certificate names disconnects remote clients. Create new pairing files afterward."
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
      <SettingsCard title="Pairing Files" description="Create a separate file for each client. Transfer it privately to that PC.">
        <form
          className="grid gap-3 p-4"
          onSubmit={(event) => {
            event.preventDefault();
            void createPairing();
          }}
        >
          <label htmlFor={`${fieldId}-label`} className="grid gap-2 text-sm">
            Label
            <Input
              id={`${fieldId}-label`}
              required
              maxLength={256}
              value={label}
              disabled={busy}
              onChange={(event) => setLabel(event.target.value)}
            />
          </label>
          <label htmlFor={`${fieldId}-endpoint`} className="grid gap-2 text-sm">
            Reachable Endpoint
            <Input
              id={`${fieldId}-endpoint`}
              type="url"
              required
              value={endpoint}
              disabled={busy}
              placeholder="https://lab.example:8765"
              onChange={(event) => setEndpoint(event.target.value)}
            />
          </label>
          <Button type="submit" disabled={busy || preferencesPending}>
            Create Pairing File
          </Button>
        </form>
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
          columns.accessor("label", { header: "Label" }),
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
