/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Masked Shared Secret Management On The Connected Backend.
 */

/* === Imports === */

// Standard Library Imports

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "@tanstack/react-router";
import { useEffect, useId, useState } from "react";
// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
// Third Party Imports
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { SecretTransfer } from "@/features/settings/sections/SecretTransfer";
import SecretsTable from "./SecretsTable.svelte";
import { putSecret, readSecrets, removeSecret, type SecretSnapshot, type SecretEntry } from "@/lib/api/native-secrets";

/* === Re-Exports === */

/* === Constants === */

const SECRETS_DESCRIPTION_BYTES = 2_048;
const SECRETS_LABEL_BYTES = 256;
const SECRETS_IDENTIFIER_BYTES = 128;
const SECRETS_SECRET_BYTES = 65_536;

/* === Types === */

type Editor = ({ kind: "new"; identifier?: string } | { kind: "edit"; entry: SecretEntry }) & { generation: bigint };

/* === Components === */

export function SecretsSection() {
  const session = useNativeSessionOptional();
  return session ? (
    <ConnectedSecrets key={session.identity.generation} session={session} />
  ) : (
    <SettingsCard title="Secret Storage">
      <SettingsRow label="Backend" description="Connect to manage the host’s secrets.">
        <span role="status">Disconnected</span>
      </SettingsRow>
    </SettingsCard>
  );
}

function ConnectedSecrets({ session }: { session: ConnectedSession }) {
  const hash = useLocation({ select: (location) => location.hash });
  const navigate = useNavigate();
  const identifier = hash.startsWith("add-secret=") ? hash.slice("add-secret=".length) : undefined;
  const client = useQueryClient();
  const queryKey = ["settings", "secrets", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => readSecrets(session.identity) });
  const snapshot = query.data;
  const [editor, setEditor] = useState<Editor | null>(() =>
    identifier && snapshot ? { kind: "new", identifier, generation: snapshot.generation } : null,
  );
  const [deleting, setDeleting] = useState<{ identifier: string; generation: bigint } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  useEffect(() => {
    if (!identifier || !snapshot) return;
    setEditor({ kind: "new", identifier, generation: snapshot.generation });
    void navigate({ hash: "", replace: true });
  }, [identifier, snapshot, navigate]);

  async function change(work: () => Promise<SecretSnapshot>) {
    setBusy(true);
    setError(null);
    try {
      await client.cancelQueries({ queryKey });
      const next = await work();
      if (!session.identity.isCurrent()) return;
      client.setQueryData(queryKey, next);
      void client.invalidateQueries({ queryKey: ["settings", "installed-plugins"] });
      setEditor(null);
      setDeleting(null);
    } catch (failure) {
      if (session.identity.isCurrent()) setError(failure);
    } finally {
      if (session.identity.isCurrent()) setBusy(false);
    }
  }

  const { rendered, portals } = useSvelteSlots({
    actions: (
      <>
        <Button disabled={!snapshot || busy} onClick={() => snapshot && setEditor({ kind: "new", generation: snapshot.generation })}>
          Add Secret
        </Button>
        <Button disabled={busy || query.isFetching} variant="outline" onClick={() => void query.refetch()}>
          Refresh
        </Button>
        {snapshot && (
          <SecretTransfer
            session={session.identity}
            catalog={snapshot}
            onImported={(next) => {
              if (session.identity.isCurrent()) {
                client.setQueryData(queryKey, next);
              }
            }}
          />
        )}
      </>
    ),
  });

  return (
    <SettingsCard title="Secret Storage">
      <SettingsRow label="Owning Host">
        <span>{snapshot ? `${snapshot.hostLabel} · ${snapshot.hostId}` : "Loading host…"}</span>
      </SettingsRow>
      <div className="grid gap-3 px-4 py-3">
        {snapshot?.cleanupPending && (
          <p role="status" className="whitespace-pre-wrap text-sm text-muted-foreground">
            The change is active, but an obsolete keyring slot could not be removed.
          </p>
        )}
        <SvelteMount
          component={SecretsTable}
          props={{
            actions: rendered.actions,
            busy: busy || !snapshot,
            entries: snapshot?.entries ?? [],
            failure: error ? failureOf(error, "Operation Failed") : query.error ? failureOf(query.error, "Secrets Unavailable") : undefined,
            loading: query.isPending,
            onDelete: (entry) => snapshot && setDeleting({ identifier: entry.metadata.identifier, generation: snapshot.generation }),
            onEdit: (entry) => snapshot && setEditor({ kind: "edit", entry, generation: snapshot.generation }),
            onRetry: !error && query.error ? () => void query.refetch() : undefined,
          }}
        />
        {portals}
        {deleting && (
          <fieldset className="flex flex-wrap items-center gap-3 rounded-lg border p-3" aria-label="Confirm Secret Deletion">
            <p className="text-sm">Delete {deleting.identifier}?</p>
            <Button
              variant="destructive"
              disabled={busy}
              onClick={() => {
                const removal = deleting;
                void change(() => removeSecret(session.identity, removal.identifier, removal.generation));
              }}
            >
              Confirm Delete
            </Button>
            <Button variant="outline" disabled={busy} onClick={() => setDeleting(null)}>
              Cancel
            </Button>
          </fieldset>
        )}
        {editor && snapshot && (
          <SecretEditor
            key={editor.kind === "edit" ? editor.entry.metadata.identifier : `new-${editor.identifier ?? ""}`}
            editor={editor}
            busy={busy}
            cancel={() => setEditor(null)}
            submit={(metadata, value) => change(() => putSecret(session.identity, metadata, value, editor.generation))}
          />
        )}
      </div>
    </SettingsCard>
  );
}

export function AddSecretDialog({ identifier, onClose }: { identifier: string; onClose: () => void }) {
  const session = useNativeSessionOptional();
  const client = useQueryClient();
  const queryKey = ["settings", "secrets", session?.identity.generation];
  const query = useQuery({
    queryKey,
    queryFn: () => {
      if (!session) throw new Error("Connect to manage Secret Storage.");
      return readSecrets(session.identity);
    },
    enabled: Boolean(session),
  });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const snapshot = query.data;
  if (!session || !snapshot) {
    return (
      <Dialog
        open
        onOpenChange={(open) => {
          if (!open) onClose();
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Secret</DialogTitle>
            <DialogDescription>
              {query.error?.message ?? (session ? "Reading Secret Storage…" : "Connect to manage Secret Storage.")}
            </DialogDescription>
          </DialogHeader>
        </DialogContent>
      </Dialog>
    );
  }
  return (
    <SecretEditor
      editor={{ kind: "new", identifier, generation: snapshot.generation }}
      busy={busy}
      cancel={onClose}
      error={error}
      submit={async (metadata, value) => {
        setBusy(true);
        setError(null);
        try {
          await client.cancelQueries({ queryKey });
          const next = await putSecret(session.identity, metadata, value, snapshot.generation);
          if (!session.identity.isCurrent()) return;
          client.setQueryData(queryKey, next);
          void client.invalidateQueries({ queryKey: ["settings", "installed-plugins"] });
          onClose();
        } catch (failure) {
          if (session.identity.isCurrent()) setError(failure);
        } finally {
          if (session.identity.isCurrent()) setBusy(false);
        }
      }}
    />
  );
}

function SecretEditor({
  editor,
  busy,
  cancel,
  submit,
  error,
}: {
  error?: unknown;
  editor: Editor;
  busy: boolean;
  cancel: () => void;
  submit: (metadata: SecretEntry["metadata"], value: string | undefined) => Promise<void>;
}) {
  const id = useId();
  const original = editor.kind === "edit" ? editor.entry.metadata : undefined;
  const [identifier, setIdentifier] = useState(original?.identifier ?? (editor.kind === "new" ? editor.identifier : "") ?? "");
  const [label, setLabel] = useState(original?.label ?? (editor.kind === "new" ? editor.identifier : "") ?? "");
  const [description, setDescription] = useState(original?.description ?? "");
  const [value, setValue] = useState("");

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) cancel();
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{editor.kind === "new" ? "Add Secret" : "Edit Secret"}</DialogTitle>
          <DialogDescription>Values stay in the host’s OS keyring.</DialogDescription>
        </DialogHeader>
        {error ? <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} /> : null}
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            const entered = value || undefined;
            setValue("");
            void submit({ identifier, label, description: description || undefined }, entered);
          }}
        >
          <Label htmlFor={`${id}-identifier`}>Identifier</Label>
          <Input
            id={`${id}-identifier`}
            required
            className="font-mono"
            pattern="(?!\.{1,2}$)[A-Za-z0-9._\-]+"
            maxLength={SECRETS_IDENTIFIER_BYTES}
            value={identifier}
            disabled={busy || editor.kind === "edit"}
            onChange={(event) => setIdentifier(event.target.value)}
          />
          <Label htmlFor={`${id}-label`}>Label</Label>
          <Input
            id={`${id}-label`}
            required
            maxLength={SECRETS_LABEL_BYTES}
            value={label}
            disabled={busy}
            onChange={(event) => setLabel(event.target.value)}
          />
          <Label htmlFor={`${id}-description`}>Description</Label>
          <Input
            id={`${id}-description`}
            maxLength={SECRETS_DESCRIPTION_BYTES}
            value={description}
            disabled={busy}
            onChange={(event) => setDescription(event.target.value)}
          />
          <Label htmlFor={`${id}-value`}>{editor.kind === "edit" ? "Replacement Value (Leave Empty to Keep)" : "Value"}</Label>
          <Input
            id={`${id}-value`}
            type="password"
            autoComplete="new-password"
            required={editor.kind === "new"}
            maxLength={SECRETS_SECRET_BYTES}
            value={value}
            disabled={busy}
            onChange={(event) => setValue(event.target.value)}
          />
          <div className="flex gap-2">
            <Button type="submit" disabled={busy}>
              Save Secret
            </Button>
            <Button type="button" disabled={busy} variant="outline" onClick={cancel}>
              Cancel
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
