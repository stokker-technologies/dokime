/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Persistent Remote Listener Settings And One-Time Key Dialog
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useId, useState } from "react";
import { toast } from "sonner";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { AccessKeyDialog } from "@/features/settings/sections/AccessKeyDialog";
import { BackendSelectionCard } from "@/features/settings/sections/BackendSelectionCard";
import { RemoteSessionsCard } from "@/features/settings/sections/RemoteSessionsCard";
import {
  certificateIssue,
  certificateNames,
  commitListener,
  keyIssue,
  type ListenerChange,
  type ListenerStatus,
  listenerError,
  listenerRunning,
} from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";
import { isTauriDesktop } from "@/lib/desktop-open";
import * as rpc from "@/lib/rpc/generated/client";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type CommitInputProps = {
  id: string;
  value: string;
  disabled: boolean;
  placeholder?: string;
  /** Resolves true when the host accepted the value; a refused value reverts to the host's. */
  commit: (value: string) => Promise<boolean>;
};

/* === Components === */

export function RemoteSection() {
  const session = useNativeSessionOptional();
  const canManage = session?.snapshot.connection.backend?.methods.includes("/dokime.v1.Remote/ReadListener") ?? false;
  if (!isTauriDesktop() || !session || !canManage) {
    return (
      <>
        {isTauriDesktop() ? <BackendSelectionCard /> : null}
        <SettingsCard title="Remote Access">
          <SettingsRow label="Host Settings">
            <p role="status">Open Remote Access in the host's connected desktop session to configure its listener.</p>
          </SettingsRow>
        </SettingsCard>
      </>
    );
  }
  return (
    <>
      <BackendSelectionCard />
      <ConnectedRemoteSection key={session.identity.generation} session={session} />
    </>
  );
}

function ConnectedRemoteSection({ session }: { session: ConnectedSession }) {
  const queryKey = ["settings", "remote-listener", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => rpc.remoteReadListener(session.identity, {}), retry: false });
  if (query.isPending) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Remote Settings">
          <span role="status">Loading remote settings…</span>
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
  return <ListenerControls session={session} status={query.data} />;
}

function ListenerControls({ session, status }: { session: ConnectedSession; status: ListenerStatus }) {
  const client = useQueryClient();
  const id = useId();
  const queryKey = ["settings", "remote-listener", session.identity.generation];
  const [busy, setBusy] = useState(false);
  const [pendingEnabled, setPendingEnabled] = useState<boolean | null>(null);
  const [rotating, setRotating] = useState(false);
  const [accessKey, setAccessKey] = useState<string | null>(null);
  const [keySaved, setKeySaved] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
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

  const toggleEnabled = async (enabled: boolean) => {
    setPendingEnabled(enabled);
    const accepted = await commit({ enabled });
    setPendingEnabled(null);
    if (accepted) toast.success(enabled ? "Remote Access Enabled" : "Remote Access Disabled");
  };

  const regenerate = (names: string[]) =>
    run(async () => {
      try {
        publish(await rpc.remoteRegenerateCertificate(session.identity, { certificateNames: names }));
        toast.success("Certificate Generated", { description: "Remote clients disconnect and must trust the new public certificate." });
        return true;
      } catch (error) {
        fail("Certificate Generation Failed", listenerError(error));
        return false;
      }
    });

  const generateKey = () =>
    run(async () => {
      try {
        const generated = await rpc.remoteGenerateKey(session.identity, {});
        if (!session.identity.isCurrent()) stale();
        if (!generated.listener) throw new Error("Backend omitted listener state.");
        // Only Component State Holds The One-Time Key; Query Caches Keep Public Status.
        setAccessKey(generated.accessKey);
        setKeySaved(false);
        setSaveError(null);
        publish(generated.listener);
        setRotating(false);
      } catch (error) {
        fail("Access Key Generation Failed", listenerError(error));
      }
    });

  const save = async (kind: "remote_key" | "public_certificate", value: string) => {
    setSaving(true);
    setSaveError(null);
    try {
      const filename = kind === "remote_key" ? "dokime-remote.key" : "dokime-server.pem";
      const path = await commands.desktopSaveDocument(session.identity.generation, { kind, filename, bytesBase64: btoa(value) });
      if (path !== null) {
        toast.success(kind === "remote_key" ? "Key Saved" : "Certificate Saved", { description: path });
        if (kind === "remote_key") setKeySaved(true);
      }
    } catch (error) {
      const description = listenerError(error);
      setSaveError(description);
      toast.error("File Not Saved", { description });
    } finally {
      setSaving(false);
    }
  };

  const certificatePem = status.certificatePem;
  const saveCertificate = certificatePem ? () => void save("public_certificate", certificatePem) : undefined;

  return (
    <>
      <SettingsCard
        title="Listener"
        description="Let another PC control this host. Listener changes disconnect remote clients; admitted executions continue."
      >
        <SettingsRow
          label="Enable Remote Access"
          htmlFor={`${id}-enabled`}
          description={running ? `Listening on ${status.listeningAddress}.` : "Remote clients can connect only while this is on."}
          error={status.error}
        >
          <Switch
            id={`${id}-enabled`}
            checked={pendingEnabled ?? running}
            disabled={busy}
            onCheckedChange={(enabled) => void toggleEnabled(enabled)}
          />
        </SettingsRow>
        <SettingsRow
          label="Listen Address"
          htmlFor={`${id}-address`}
          description="An IP address and port, such as 0.0.0.0:8765 or [::]:8765. It applies when you press Enter or leave the field."
        >
          <CommitInput id={`${id}-address`} value={settings.address} disabled={busy} commit={(address) => commit({ address })} />
        </SettingsRow>
        <SettingsRow
          label="TLS Encryption"
          htmlFor={`${id}-tls`}
          description="Clients verify the public certificate. Turning it on without one generates a certificate for this host."
          error={
            settings.tlsEnabled ? null : "Traffic, including the access key, is sent without encryption. Use this only in an isolated lab."
          }
        >
          <Switch
            id={`${id}-tls`}
            checked={settings.tlsEnabled}
            disabled={busy}
            onCheckedChange={(tlsEnabled) => void commit({ tlsEnabled })}
          />
        </SettingsRow>
        <SettingsRow
          label="Require An Access Key"
          htmlFor={`${id}-auth`}
          description="One key gives full control of this host."
          error={
            settings.authenticationEnabled ? null : "Anyone who can reach this listener has full control. Use this only in an isolated lab."
          }
        >
          <Switch
            id={`${id}-auth`}
            checked={settings.authenticationEnabled}
            disabled={busy}
            onCheckedChange={(authenticationEnabled) => void commit({ authenticationEnabled })}
          />
        </SettingsRow>
      </SettingsCard>
      <SettingsCard
        title="Secrets"
        description="Clients present the access key and trust the public certificate. The TLS private key stays on this host."
      >
        <SettingsRow
          label="Certificate DNS Names And IP Addresses"
          htmlFor={`${id}-names`}
          description="The names clients connect to, separated by commas. Leave it empty to use this host's name and addresses. A change regenerates the certificate."
        >
          <CommitInput
            id={`${id}-names`}
            value={settings.certificateNames.join(", ")}
            placeholder="This host's name and addresses"
            disabled={busy}
            commit={(text) => regenerate(certificateNames(text))}
          />
        </SettingsRow>
        <SettingsRow
          label="Access Key"
          description={
            status.keyConfigured
              ? "The host keeps only a verifier and cannot show the key again. Rotate it if it is lost."
              : "Clients present this key to connect."
          }
          error={keyIssue(status)}
        >
          <Button variant="outline" disabled={busy} onClick={() => (status.keyConfigured ? setRotating(true) : void generateKey())}>
            {status.keyConfigured ? "Rotate Access Key…" : "Generate Access Key"}
          </Button>
        </SettingsRow>
        <SettingsRow label="Public Certificate" description="Transfer this public file to the remote PC." error={certificateIssue(status)}>
          {saveCertificate ? (
            <>
              <Button variant="outline" disabled={saving} onClick={saveCertificate}>
                Save Certificate…
              </Button>
              <Button variant="outline" disabled={busy} onClick={() => void regenerate(settings.certificateNames)}>
                Regenerate Certificate
              </Button>
            </>
          ) : (
            <Button variant="outline" disabled={busy} onClick={() => void regenerate(settings.certificateNames)}>
              Generate Certificate
            </Button>
          )}
        </SettingsRow>
      </SettingsCard>
      {session.snapshot.connection.backend?.methods.includes("/dokime.v1.Remote/ListSessions") ? (
        <RemoteSessionsCard session={session} />
      ) : null}
      <Dialog
        open={rotating}
        onOpenChange={(open) => {
          if (!open && !busy) setRotating(false);
        }}
      >
        <DialogContent showCloseButton={!busy}>
          <DialogHeader>
            <DialogTitle>Rotate Access Key</DialogTitle>
            <DialogDescription>
              The old key stops working and remote streams disconnect. Save the replacement when it appears. Admitted executions continue.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" disabled={busy} onClick={() => setRotating(false)}>
              Cancel
            </Button>
            <Button disabled={busy} onClick={() => void generateKey()}>
              {busy ? "Rotating…" : "Rotate"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {accessKey !== null ? (
        <AccessKeyDialog
          value={accessKey}
          busy={saving}
          saved={keySaved}
          saveError={saveError}
          close={() => setAccessKey(null)}
          save={() => void save("remote_key", `${accessKey}\n`)}
          saveCertificate={saveCertificate}
        />
      ) : null}
    </>
  );
}

/** A text field that applies on Enter or blur and otherwise shows the host's value. */
function CommitInput({ id, value, disabled, placeholder, commit }: CommitInputProps) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  return (
    <Input
      id={id}
      value={draft}
      disabled={disabled}
      placeholder={placeholder}
      onChange={(event) => setDraft(event.target.value)}
      onKeyDown={(event) => {
        if (event.key === "Enter") event.currentTarget.blur();
      }}
      onBlur={() => {
        const next = draft.trim();
        if (next === value) return setDraft(value);
        void commit(next).then((accepted) => {
          if (!accepted) setDraft(value);
        });
      }}
    />
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
