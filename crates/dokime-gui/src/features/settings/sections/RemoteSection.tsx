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
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { commands } from "@/lib/desktop/commands";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type ListenerStatus = Wire<Awaited<ReturnType<typeof rpc.remoteReadListener>>>;
type ListenerSettings = NonNullable<ListenerStatus["settings"]>;
type Action = "apply" | "key" | "certificate";

/* === Components === */

export function RemoteSection() {
  const session = useNativeSessionOptional();
  const canManage = session?.snapshot.connection.backend?.methods.includes("/dokime.v1.Remote/ReadListener") ?? false;
  if (!isTauriDesktop() || !session || !canManage) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Host Settings">
          <p role="status">Open Remote Access in the host's connected desktop session to configure its listener.</p>
        </SettingsRow>
      </SettingsCard>
    );
  }
  return <ConnectedRemoteSection key={session.identity.generation} session={session} />;
}

function ConnectedRemoteSection({ session }: { session: ConnectedSession }) {
  const queryKey = ["settings", "remote-listener", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => rpc.remoteReadListener(session.identity, {}), retry: false });
  if (query.isPending) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Status">
          <span role="status">Loading remote settings…</span>
        </SettingsRow>
      </SettingsCard>
    );
  }
  if (query.error || !query.data.settings) {
    return (
      <SettingsCard title="Remote Access">
        <SettingsRow label="Status">
          <p role="alert">{message(query.error)}</p>
          <Button variant="outline" onClick={() => void query.refetch()}>
            Try Again
          </Button>
        </SettingsRow>
      </SettingsCard>
    );
  }
  return <ListenerForm session={session} status={query.data} initial={query.data.settings} />;
}

function ListenerForm({ session, status, initial }: { session: ConnectedSession; status: ListenerStatus; initial: ListenerSettings }) {
  const client = useQueryClient();
  const id = useId();
  const [settings, setSettings] = useState(initial);
  const [names, setNames] = useState(initial.certificateNames.join(", "));
  const [confirm, setConfirm] = useState<Action | null>(null);
  const [accessKey, setAccessKey] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<unknown>(null);
  const [keySaved, setKeySaved] = useState(false);
  const certificateNames = names.split(/[\s,]+/).filter(Boolean);
  const mutation = useMutation({
    mutationFn: async (action: Action) => {
      setNotice(null);
      setSaveError(null);
      let updated: ListenerStatus;
      if (action === "key") {
        const generated = await rpc.remoteGenerateKey(session.identity, {});
        if (!session.identity.isCurrent()) throw new Error("Backend connection changed; reconnect and generate a new key.");
        if (!generated.listener) throw new Error("Backend omitted listener state.");
        // Only Component State Holds The One-Time Key; Query Caches Keep Public Status.
        setAccessKey(generated.accessKey);
        setKeySaved(false);
        updated = generated.listener;
      } else if (action === "certificate") {
        updated = await rpc.remoteRegenerateCertificate(session.identity, { certificateNames });
      } else {
        updated = await rpc.remoteUpdateListener(session.identity, { settings: { ...settings, certificateNames } });
      }
      if (!session.identity.isCurrent()) throw new Error("Backend connection changed; reconnect to check the saved settings.");
      client.setQueryData(["settings", "remote-listener", session.identity.generation], updated);
      if (action === "apply" && updated.settings) setSettings(updated.settings);
      setNotice(action === "key" ? "New Key Generated" : action === "certificate" ? "Certificate Regenerated" : "Settings Applied");
      setConfirm(null);
    },
    onError: () => void client.invalidateQueries({ queryKey: ["settings", "remote-listener", session.identity.generation] }),
  });
  const save = async (kind: "remote_key" | "public_certificate", value: string) => {
    setSaving(true);
    setSaveError(null);
    try {
      const filename = kind === "remote_key" ? "dokime-remote.key" : "dokime-server.pem";
      const path = await commands.desktopSaveDocument(session.identity.generation, { kind, filename, bytesBase64: btoa(value) });
      if (path !== null) {
        setNotice(kind === "remote_key" ? "Key Saved" : "Certificate Saved");
        if (kind === "remote_key") setKeySaved(true);
      }
    } catch (error) {
      setSaveError(error);
    } finally {
      setSaving(false);
    }
  };
  const busy = mutation.isPending || saving;
  const ready =
    settings.address.trim().length > 0 &&
    (!settings.enabled ||
      ((!settings.authenticationEnabled || status.keyConfigured) && (!settings.tlsEnabled || certificateNames.length > 0)));
  const error = mutation.error ?? saveError;

  return (
    <>
      <SettingsCard
        title="Remote Access"
        description="Let another PC control this host. Listener changes disconnect remote clients; admitted executions continue."
      >
        <SettingsRow label="Status">
          <span role="status">{status.listeningAddress ? `Listening On ${status.listeningAddress}` : "Remote Listener Stopped"}</span>
        </SettingsRow>
        {status.error ? (
          <SettingsRow label="Listener Error">
            <p role="alert" className="text-destructive">
              {status.error}
            </p>
          </SettingsRow>
        ) : null}
        <fieldset disabled={busy}>
          <SettingsRow label="Enable Remote Access" htmlFor={`${id}-enabled`}>
            <Switch id={`${id}-enabled`} checked={settings.enabled} onCheckedChange={(enabled) => setSettings({ ...settings, enabled })} />
          </SettingsRow>
          <SettingsRow
            label="Listen Address"
            htmlFor={`${id}-address`}
            description="Use an IP address and port, such as 0.0.0.0:8765 or [::]:8765."
          >
            <Input
              id={`${id}-address`}
              value={settings.address}
              onChange={(event) => setSettings({ ...settings, address: event.target.value })}
            />
          </SettingsRow>
          <SettingsRow
            label="TLS Encryption"
            htmlFor={`${id}-tls`}
            description="Clients verify the public certificate. Plaintext is an explicit isolated-lab option."
          >
            <Switch
              id={`${id}-tls`}
              checked={settings.tlsEnabled}
              onCheckedChange={(tlsEnabled) => setSettings({ ...settings, tlsEnabled })}
            />
          </SettingsRow>
          <SettingsRow
            label="Require An Access Key"
            htmlFor={`${id}-auth`}
            description="One key gives full control. Authentication can be disabled independently for an isolated lab."
          >
            <Switch
              id={`${id}-auth`}
              checked={settings.authenticationEnabled}
              onCheckedChange={(authenticationEnabled) => setSettings({ ...settings, authenticationEnabled })}
            />
          </SettingsRow>
          <SettingsRow
            label="Certificate DNS Names And IP Addresses"
            htmlFor={`${id}-names`}
            description="Enter the exact host names or IP addresses clients will connect to, separated by commas. Changing these requires explicit certificate regeneration."
          >
            <Input id={`${id}-names`} value={names} onChange={(event) => setNames(event.target.value)} />
          </SettingsRow>
          {!settings.tlsEnabled || !settings.authenticationEnabled ? (
            <SettingsRow label="Isolated Lab Only">
              <p>
                {!settings.tlsEnabled ? "Traffic, including any access key, will be sent without encryption. " : ""}
                {!settings.authenticationEnabled ? "Anyone who can reach this listener will have full control." : ""}
              </p>
            </SettingsRow>
          ) : null}
          <SettingsRow
            label="Access Key"
            description={
              status.keyConfigured
                ? "The saved verifier cannot recover your key. Generate a new one if it is lost."
                : "Generate and save a key before enabling authenticated access."
            }
          >
            <Button variant="outline" onClick={() => (status.keyConfigured ? setConfirm("key") : mutation.mutate("key"))}>
              {status.keyConfigured ? "Rotate Access Key…" : "Generate Access Key"}
            </Button>
          </SettingsRow>
          <SettingsRow
            label="Public Certificate"
            description="Transfer this public file to the remote PC. The TLS private key stays on the host."
          >
            <div className="flex flex-wrap gap-2">
              <Button
                variant="outline"
                disabled={!status.certificatePem}
                onClick={() => {
                  if (status.certificatePem) void save("public_certificate", status.certificatePem);
                }}
              >
                Save Certificate…
              </Button>
              <Button variant="outline" disabled={certificateNames.length === 0} onClick={() => setConfirm("certificate")}>
                Regenerate Certificate…
              </Button>
            </div>
          </SettingsRow>
          <SettingsRow
            label="Apply Settings"
            description="Only this host's remote listener restarts. Its previous settings are restored if the change fails."
          >
            <Button disabled={!ready} onClick={() => setConfirm("apply")}>
              Apply…
            </Button>
          </SettingsRow>
        </fieldset>
        {busy ? (
          <SettingsRow label="Working">
            <span role="status">{saving ? "Waiting For File Save…" : "Applying Remote Access Change…"}</span>
          </SettingsRow>
        ) : null}
        {error ? (
          <SettingsRow label="Change Failed">
            <p role="alert" className="text-destructive">
              {message(error)}
            </p>
          </SettingsRow>
        ) : null}
        {notice ? (
          <SettingsRow label="Result">
            <span role="status">{notice}</span>
          </SettingsRow>
        ) : null}
      </SettingsCard>
      <Dialog
        open={confirm !== null}
        onOpenChange={(open) => {
          if (!open && !busy) setConfirm(null);
        }}
      >
        <DialogContent showCloseButton={!busy}>
          <DialogHeader>
            <DialogTitle>
              {confirm === "key"
                ? "Rotate Access Key"
                : confirm === "certificate"
                  ? "Regenerate Certificate"
                  : "Apply Remote Access Settings"}
            </DialogTitle>
            <DialogDescription>
              {confirm === "key"
                ? "The old key stops working and remote streams disconnect. Save the replacement when it appears. Admitted executions continue."
                : confirm === "certificate"
                  ? "Remote clients disconnect and must trust the new public certificate. This changes the advertised certificate identities without changing the access key."
                  : "Remote clients disconnect while these settings take effect. Admitted executions and the native desktop session continue."}
            </DialogDescription>
          </DialogHeader>
          {confirm === "apply" && (!settings.tlsEnabled || !settings.authenticationEnabled) ? (
            <p role="alert">
              {!settings.tlsEnabled ? "Encryption Is Disabled. " : ""}
              {!settings.authenticationEnabled ? "Authentication Is Disabled. " : ""}Use these settings only in an isolated lab.
            </p>
          ) : null}
          <DialogFooter>
            <Button variant="outline" disabled={busy} onClick={() => setConfirm(null)}>
              Cancel
            </Button>
            <Button disabled={busy} onClick={() => confirm && mutation.mutate(confirm)}>
              {busy ? "Applying…" : "Confirm"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {accessKey !== null ? (
        <AccessKeyDialog
          value={accessKey}
          busy={saving}
          saved={keySaved}
          saveError={saveError === null ? null : message(saveError)}
          close={() => setAccessKey(null)}
          save={() => {
            void save("remote_key", `${accessKey}\n`);
          }}
          saveCertificate={
            status.certificatePem
              ? () => {
                  void save("public_certificate", status.certificatePem ?? "");
                }
              : undefined
          }
        />
      ) : null}
    </>
  );
}

function AccessKeyDialog({
  value,
  busy,
  saved,
  saveError,
  close,
  save,
  saveCertificate,
}: {
  value: string;
  busy: boolean;
  saved: boolean;
  saveError: string | null;
  close: () => void;
  save: () => void;
  saveCertificate: (() => void) | undefined;
}) {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) close();
      }}
    >
      <DialogContent className="sm:max-w-lg" showCloseButton={!busy}>
        <DialogHeader>
          <DialogTitle>Save Your Access Key</DialogTitle>
          <DialogDescription>
            This is the only time the key is shown. It gives full control of this host. Copy it or save it as a key file before closing.
          </DialogDescription>
        </DialogHeader>
        <Input aria-label="Generated Access Key" autoComplete="off" readOnly value={value} className="font-mono" />
        {error || saveError ? <p role="alert">{error ?? saveError}</p> : null}
        {saved || copied ? <p role="status">{saved ? "Key File Saved" : "Key Copied"}</p> : null}
        <DialogFooter>
          <Button
            variant="outline"
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(value);
                setCopied(true);
              } catch {
                setError("Copy failed; save the key file or select and copy the key.");
              }
            }}
          >
            {copied ? "Copied" : "Copy Key"}
          </Button>
          <Button variant="outline" disabled={busy} onClick={save}>
            {busy ? "Saving…" : "Save Key File…"}
          </Button>
          <Button variant="outline" disabled={busy || !saveCertificate} onClick={saveCertificate}>
            Save Public Certificate…
          </Button>
          <Button disabled={busy} onClick={close}>
            I Have Saved The Key
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function message(error: unknown): string {
  return error instanceof Error ? error.message : "Remote access settings are unavailable. Reconnect to the host and try again.";
}
