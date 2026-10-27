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
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { SecretTransfer } from "@/features/settings/sections/SecretTransfer";
import { putSecret, readSecrets, removeSecret, type SecretCatalog, type SecretEntry } from "@/lib/api/native-secrets";

/* === Re-Exports === */

/* === Constants === */

const SECRETS_DESCRIPTION_BYTES = 2_048;
const SECRETS_LABEL_BYTES = 256;
const SECRETS_NAME_BYTES = 128;
const SECRETS_SECRET_BYTES = 65_536;

/* === Types === */

type Editor = ({ kind: "new" } | { kind: "edit"; entry: SecretEntry }) & { generation: bigint };

/* === Components === */

export function SecretsSection() {
  const session = useNativeSessionOptional();
  return session ? (
    <ConnectedSecrets key={session.identity.generation} session={session} />
  ) : (
    <SettingsCard title="Secrets">
      <SettingsRow label="Backend" description="Connect to manage the host’s secrets.">
        <span role="status">Disconnected</span>
      </SettingsRow>
    </SettingsCard>
  );
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

  const column = dataTableColumns<SecretEntry>();
  const columns = column.columns([
    column.accessor((entry) => entry.metadata.name, {
      id: "name",
      header: "Name",
      cell: ({ row }) => (
        <div className="grid gap-1">
          <strong>{row.original.metadata.label}</strong>
          <span className="font-mono text-xs">{row.original.metadata.name}</span>
          <p className="text-xs text-muted-foreground">{row.original.metadata.description}</p>
        </div>
      ),
    }),
    column.display({
      id: "value",
      header: "Value",
      cell: () => (
        <span>
          <span aria-hidden="true">••••••••</span>
          <span className="sr-only">Masked Value</span>
        </span>
      ),
    }),
    column.display({
      id: "references",
      header: "References",
      cell: ({ row }) => <span className="text-xs">{row.original.knownUses.join(", ") || "—"}</span>,
    }),
    column.display({
      id: "actions",
      header: "Actions",
      cell: ({ row }) => (
        <DataTableRowActions
          label={row.original.metadata.name}
          primary={{
            label: "Edit",
            disabled: busy || !snapshot,
            onSelect: () => snapshot && setEditor({ kind: "edit", entry: row.original, generation: snapshot.generation }),
          }}
        >
          <DropdownMenuItem
            variant="destructive"
            disabled={busy || !snapshot}
            onSelect={() => snapshot && setDeleting({ name: row.original.metadata.name, generation: snapshot.generation })}
          >
            Delete
          </DropdownMenuItem>
        </DataTableRowActions>
      ),
    }),
  ]);

  return (
    <SettingsCard title="Secrets">
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
        <DataTable
          aria-label="Secrets"
          actions={
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
                    if (session.identity.isCurrent()) client.setQueryData(queryKey, next);
                  }}
                />
              )}
            </>
          }
          columns={columns}
          data={snapshot?.entries ?? []}
          getRowId={(entry) => entry.metadata.name}
          emptyText="No shared secrets."
        />
        {deleting && (
          <fieldset className="flex flex-wrap items-center gap-3 rounded-lg border p-3" aria-label="Confirm Secret Deletion">
            <p className="text-sm">Delete {deleting.name}?</p>
            <Button
              variant="destructive"
              disabled={busy}
              onClick={() => {
                const removal = deleting;
                void change(() => removeSecret(session.identity, removal.name, removal.generation));
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
            key={editor.kind === "edit" ? editor.entry.metadata.name : editor.kind}
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
      className="grid gap-3 rounded-lg border p-3"
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
        maxLength={SECRETS_NAME_BYTES}
        value={name}
        disabled={busy || editor.kind === "edit"}
        onChange={(event) => setName(event.target.value)}
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
      <Label htmlFor={`${id}-value`}>{editor.kind === "edit" ? "Replacement Value (Leave Empty To Keep)" : "Value"}</Label>
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
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
