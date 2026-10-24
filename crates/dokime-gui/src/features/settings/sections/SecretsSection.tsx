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

// Third Party Imports
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { SecretTransfer } from "@/features/settings/sections/SecretTransfer";
import { putSecret, readSecrets, removeSecret, type SecretCatalog, type SecretEntry } from "@/lib/api/native-secrets";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Editor = ({ kind: "new" } | { kind: "edit"; entry: SecretEntry }) & { generation: bigint };

/* === Components === */

export function SecretsSection() {
  const session = useNativeSessionOptional();
  return session ? <ConnectedSecrets key={session.identity.generation} session={session} /> : null;
}

function ConnectedSecrets({ session }: { session: ConnectedSession }) {
  const client = useQueryClient();
  const queryKey = ["settings", "secrets", session.identity.generation];
  const query = useQuery({ queryKey, queryFn: () => readSecrets(session.identity) });
  const [editor, setEditor] = useState<Editor | null>(null);
  const [deleting, setDeleting] = useState<{ name: string; generation: bigint } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const snapshot = query.data;

  async function change(work: () => Promise<SecretCatalog>) {
    setBusy(true);
    setError(null);
    try {
      await client.cancelQueries({ queryKey });
      const next = await work();
      if (!session.identity.isCurrent()) return;
      client.setQueryData(queryKey, next);
      setEditor(null);
      setDeleting(null);
    } catch (failure) {
      if (session.identity.isCurrent()) setError(failure instanceof Error ? failure.message : "Secret storage is unavailable");
    } finally {
      if (session.identity.isCurrent()) setBusy(false);
    }
  }

  return (
    <SettingsCard title="Secret Storage">
      <SettingsRow label="Owning Host">
        <span>{snapshot ? `${snapshot.hostLabel} · ${snapshot.hostId}` : "Loading host…"}</span>
      </SettingsRow>
      <div className="grid gap-3 px-4 py-3">
        <p className="text-sm text-muted-foreground">All plugins on this host can look up these names. Values stay in its OS keyring.</p>
        {(error || query.error) && (
          <p role="alert" className="text-sm text-destructive">
            {error ?? query.error?.message}
          </p>
        )}
        {snapshot?.cleanupPending && <p role="status">The change is active, but an obsolete keyring slot could not be removed.</p>}
        <div className="flex flex-wrap gap-2">
          <Button disabled={!snapshot || busy} onClick={() => snapshot && setEditor({ kind: "new", generation: snapshot.generation })}>
            Add Secret
          </Button>
          <Button disabled={busy || query.isFetching} variant="outline" onClick={() => void query.refetch()}>
            Refresh
          </Button>
        </div>
        {snapshot?.entries.length === 0 && <p className="text-sm text-muted-foreground">No shared secrets are stored on this host.</p>}
        {snapshot?.entries.map((entry) => (
          <div key={entry.metadata.name} className="grid gap-2 rounded-md border p-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <strong>{entry.metadata.label}</strong>
                <p className="font-mono text-sm">{entry.metadata.name}</p>
              </div>
              <span>
                <span aria-hidden="true">••••••••</span>
                <span className="sr-only">Stored value is masked</span>
              </span>
              <div className="flex gap-2">
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() => setEditor({ kind: "edit", entry, generation: snapshot.generation })}
                >
                  Edit {entry.metadata.name}
                </Button>
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() => setDeleting({ name: entry.metadata.name, generation: snapshot.generation })}
                >
                  Delete {entry.metadata.name}
                </Button>
              </div>
            </div>
            {entry.metadata.description && <p className="text-sm">{entry.metadata.description}</p>}
            <p className="text-sm text-muted-foreground">
              Known References: {entry.knownUses.length ? entry.knownUses.join(", ") : "None"}. Plugins may also look up this name
              dynamically.
            </p>
            {deleting && deleting.name === entry.metadata.name && (
              <fieldset className="flex flex-wrap items-center gap-2" aria-label={`Confirm deletion of ${deleting.name}`}>
                <p className="text-sm">Future lookups will fail. Already fetched values remain with their callers.</p>
                <Button
                  disabled={busy}
                  variant="destructive"
                  onClick={() => {
                    const removal = deleting;
                    if (removal) void change(() => removeSecret(session.identity, entry.metadata.name, removal.generation));
                  }}
                >
                  Confirm Delete
                </Button>
                <Button disabled={busy} variant="outline" onClick={() => setDeleting(null)}>
                  Cancel
                </Button>
              </fieldset>
            )}
          </div>
        ))}
        {editor && snapshot && (
          <SecretEditor
            key={editor.kind === "edit" ? editor.entry.metadata.name : editor.kind}
            editor={editor}
            busy={busy}
            cancel={() => setEditor(null)}
            submit={(metadata, value) => change(() => putSecret(session.identity, metadata, value, editor.generation))}
          />
        )}
        {snapshot && (
          <SecretTransfer
            session={session.identity}
            catalog={snapshot}
            onImported={(next) => {
              if (session.identity.isCurrent()) client.setQueryData(queryKey, next);
            }}
          />
        )}
      </div>
    </SettingsCard>
  );
}

function SecretEditor({
  editor,
  busy,
  cancel,
  submit,
}: {
  editor: Editor;
  busy: boolean;
  cancel: () => void;
  submit: (metadata: SecretEntry["metadata"], value: string | undefined) => Promise<void>;
}) {
  const id = useId();
  const original = editor.kind === "edit" ? editor.entry.metadata : undefined;
  const [name, setName] = useState(original?.name ?? "");
  const [label, setLabel] = useState(original?.label ?? "");
  const [description, setDescription] = useState(original?.description ?? "");
  const [value, setValue] = useState("");

  return (
    <form
      className="grid gap-3 rounded-md border p-3"
      onSubmit={(event) => {
        event.preventDefault();
        const entered = value || undefined;
        setValue("");
        void submit({ name, label, description: description || undefined }, entered);
      }}
    >
      <Label htmlFor={`${id}-name`}>Name</Label>
      <Input
        id={`${id}-name`}
        required
        maxLength={128}
        value={name}
        disabled={busy || editor.kind === "edit"}
        onChange={(event) => setName(event.target.value)}
      />
      <Label htmlFor={`${id}-label`}>Label</Label>
      <Input id={`${id}-label`} required maxLength={256} value={label} disabled={busy} onChange={(event) => setLabel(event.target.value)} />
      <Label htmlFor={`${id}-description`}>Description</Label>
      <Input
        id={`${id}-description`}
        maxLength={2048}
        value={description}
        disabled={busy}
        onChange={(event) => setDescription(event.target.value)}
      />
      <Label htmlFor={`${id}-value`}>{editor.kind === "edit" ? "Replacement Value (Leave Empty To Keep)" : "Value"}</Label>
      <Input
        id={`${id}-value`}
        type="password"
        autoComplete="new-password"
        required={editor.kind === "new"}
        maxLength={65536}
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
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
