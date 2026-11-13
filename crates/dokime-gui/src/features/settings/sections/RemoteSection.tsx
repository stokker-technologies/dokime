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

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
// Third Party Imports
import { Button } from "@/components/ui/button";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { FirewallNotice } from "@/features/settings/sections/FirewallNotice";
import { commitListener, type ListenerChange, type ListenerStatus, listenerRunning } from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";
import { isTauriDesktop } from "@/lib/desktop-open";
import { toast } from "@/lib/notifications";
import { readNativeListener, updateNativeListener, regenerateNativeCertificate } from "@/lib/api/native-remote";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

export function RemoteSection({ advanced = false }: { advanced?: boolean }) {
  const session = useNativeSessionOptional();
  const [preferencesPending, setPreferencesPending] = useState(false);
  const canManage = session?.snapshot.connection.backend?.methods.includes("/harmos.v1.access.Listener/Get") ?? false;
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
  const query = useQuery({ queryKey, queryFn: () => readNativeListener(session.identity), retry: false });
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
        <SettingsRow label="Remote Settings">
          <ErrorCard failure={failureOf(query.error, "Remote Settings Unavailable")} onRetry={() => void query.refetch()} />
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
  const settings = status.settings;
  if (!settings) throw new Error("Backend omitted listener settings.");
  const running = listenerRunning(status);

  const publish = (updated: ListenerStatus) => {
    if (!session.identity.isCurrent()) stale();
    client.setQueryData(queryKey, updated);
    return updated;
  };
  const fail = (title: string, description: string, error?: unknown) => {
    toast.error(failureOf(error, title), { description });
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
        send: async (request) => publish(await updateNativeListener(session.identity, status, request)),
        fail: (description, error) => fail("Remote Access Change Failed", description, error),
      });
      return next !== status;
    });

  async function createPairing() {
    await run(async () => {
      try {
        const path = await commands.desktopPairingCreate();
        if (!session.identity.isCurrent()) stale();
        if (path !== null) {
          toast.success("Remote Access File Saved", { description: path });
          void client.invalidateQueries({ queryKey });
        }
      } catch (error) {
        const reason = failureOf(error, "Settings Operation Failed").message;
        fail("Remote Access File Not Saved", reason, error);
      }
    });
  }

  return (
    <>
      {session.snapshot.ownership === "owned_local" && (
        <FirewallNotice generation={session.identity.generation} listenerAddress={status.listeningAddress} enabled={settings.enabled} />
      )}
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
            description="Replacing the self-signed certificate requires exporting and importing a new pairing file."
          >
            <Button
              variant="outline"
              disabled={busy || preferencesPending}
              onClick={() =>
                void run(async () => {
                  try {
                    publish(await regenerateNativeCertificate(session.identity));
                  } catch (error) {
                    fail("Certificate Generation Failed", failureOf(error, "Settings Operation Failed").message, error);
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
        description="Anyone with the file has full control. Keep pairing files private; each export creates a separate client credential."
      >
        <div className="grid gap-3 p-4">
          <p className="text-sm text-muted-foreground">
            Endpoints are detected automatically. Loopback addresses (127.0.0.1, ::1) work on the same PC only.
          </p>
          <ul aria-label="Export Endpoints" className="grid gap-1 text-sm">
            {status.endpoints.map((endpoint) => (
              <li key={endpoint.address}>
                <span className="font-mono">{endpoint.address}</span>
                {endpoint.adapterName && <span className="text-muted-foreground"> — {endpoint.adapterName}</span>}
              </li>
            ))}
          </ul>
          <Button disabled={busy || preferencesPending} onClick={() => void createPairing()}>
            Export Remote Access
          </Button>
        </div>
      </SettingsCard>
    </>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function stale(): never {
  throw new Error("Backend connection changed; reconnect to check the remote settings.");
}
