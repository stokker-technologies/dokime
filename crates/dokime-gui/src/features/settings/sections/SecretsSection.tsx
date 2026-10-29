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

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil } from "lucide-react";
import { useId, useState } from "react";

import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";

import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { SecretTransfer } from "@/features/settings/sections/SecretTransfer";
import { putSecret, readSecrets, removeSecret, type SecretCatalog, type SecretEntry } from "@/lib/api/native-secrets";

const SECRETS_DESCRIPTION_BYTES = 2_048;
const SECRETS_LABEL_BYTES = 256;
const SECRETS_IDENTIFIER_BYTES = 128;
const SECRETS_SECRET_BYTES = 65_536;

type Editor = ({ kind: "new" } | { kind: "edit"; entry: SecretEntry }) & { generation: bigint };

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
  const [deleting, setDeleting] = useState<{ identifier: string; generation: bigint } | null>(null);
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
    column.accessor((entry) => entry.metadata.identifier, {
      id: "identifier",
      header: "Identifier",
      cell: ({ row }) => <span className="font-mono text-xs">{row.original.metadata.identifier}</span>,
    }),
    column.accessor((entry) => entry.metadata.label, {
      id: "label",
      header: "Label",
      cell: ({ row }) => (
        <div className="grid gap-1">
          <strong>{row.original.metadata.label}</strong>
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
          label={row.original.metadata.identifier}
          primary={{
            label: "Edit",
            icon: Pencil,
            disabled: busy || !snapshot,
            onSelect: () => snapshot && setEditor({ kind: "edit", entry: row.original, generation: snapshot.generation }),
          }}
        >
          <DropdownMenuItem
            variant="destructive"
            disabled={busy || !snapshot}
            onSelect={() => snapshot && setDeleting({ identifier: row.original.metadata.identifier, generation: snapshot.generation })}
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
        <p className="text-sm text-muted-foreground">
          All plugins on this host can look up these identifiers. Values stay in its OS keyring.
        </p>
        {(error || query.error) && <NotificationMessage>{error ?? query.error?.message}</NotificationMessage>}
        {snapshot?.cleanupPending && (
          <NotificationMessage source="Secrets" title="Keyring Cleanup Pending" tone="info">
            The change is active, but an obsolete keyring slot could not be removed.
          </NotificationMessage>
        )}
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
          getRowId={(entry) => entry.metadata.identifier}
          emptyText="No Shared Secrets"
        />
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
            key={editor.kind === "edit" ? editor.entry.metadata.identifier : editor.kind}
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
  const [identifier, setIdentifier] = useState(original?.identifier ?? "");
  const [label, setLabel] = useState(original?.label ?? "");
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
      </DialogContent>
    </Dialog>
  );
}
