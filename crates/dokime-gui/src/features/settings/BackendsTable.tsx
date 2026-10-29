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

import type { Document, Profile } from "@generated/application-settings";
import { Plug } from "lucide-react";
import { type FormEvent, type ReactNode, useEffect, useId, useRef, useState, useSyncExternalStore } from "react";
import { useNativeSessionController, useNativeSessionOptional } from "@/app/native-session/context";
import { connectionError } from "@/app/native-session/errors";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { commands } from "@/lib/desktop/commands";
import { preferencesCommands } from "@/lib/desktop/settings";
import { preferencesError } from "./application-preferences";
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
  const session = useNativeSessionOptional();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const lastSelected = useRef<string | null>(null);
  const addButton = useRef<HTMLButtonElement>(null);
  const deleteButton = useRef<HTMLButtonElement>(null);
  const [document, setDocument] = useState<Document | null>(null);
  const [error, setError] = useState<unknown>();
  const [adding, setAdding] = useState(false);
  const [deleting, setDeleting] = useState<string | null>(null);
  const [mutating, setBusy] = useState(false);
  const busy = mutating || state.phase === "connecting";
  const connectionFailure = failure ?? (state.phase === "failed" ? connectionError(state.error) : null);
  const failedProfile = document?.profiles.find((profile) => profile.identifier === (lastSelected.current ?? document.default));
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
  function connect(identifier: string) {
    if (busy) return;
    onConnect?.();
    lastSelected.current = identifier;
    void owner.select({ kind: "profile", identifier });
  }
  return (
    <SettingsCard className="w-full text-left">
      {(connectionFailure || Boolean(error)) && (
        <div className="grid gap-3 bg-destructive/5 p-4 text-sm">
          <div role="alert" className="text-destructive">
            {connectionFailure || preferencesError(error)}
            {connectionFailure && (endpoint || failedProfile) && (
              <p className="mt-2 break-all font-mono text-xs">{endpoint || failedProfile?.endpoint}</p>
            )}
          </div>
          <Button
            className="justify-self-start"
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => {
              if (connectionFailure) {
                if (retry) retry(lastSelected.current ?? document?.default ?? null);
                else if (lastSelected.current) connect(lastSelected.current);
                else if (document?.default) connect(document.default);
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
      {!document && !error && (
        <p role="status" className="p-4 text-sm text-muted-foreground">
          Loading backends…
        </p>
      )}
      <DataTable
        aria-label="Saved Backends"
        data={document?.profiles ?? []}
        getRowId={(profile) => profile.identifier}
        emptyText="No saved backends."
        columns={dataTableColumns<Profile>().columns([
          dataTableColumns<Profile>().accessor("identifier", { header: "Identifier", meta: { primary: true, filter: "text" } }),
          dataTableColumns<Profile>().accessor("label", { header: "Label", meta: { filter: "text" } }),
          dataTableColumns<Profile>().accessor("endpoint", {
            header: "Endpoint",
            meta: { filter: "text", width: "w-[40%]", truncate: true },
          }),
          dataTableColumns<Profile>().accessor((profile) => (document?.default === profile.identifier ? "Last Used" : "—"), {
            id: "lastUsed",
            header: "Last Used",
            meta: { filter: "facet", width: "w-28" },
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
        ])}
        onRowClick={(profile) => connect(profile.identifier)}
        actions={
          <Button ref={addButton} size="sm" variant="outline" disabled={busy} onClick={() => setAdding(true)}>
            Add Backend
          </Button>
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
            addButton.current?.focus();
          }}
        >
          <DialogHeader>
            <DialogTitle>Delete Backend</DialogTitle>
            <DialogDescription>Delete {deleting}? An active connection will disconnect.</DialogDescription>
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
            <p role="alert" className="text-sm text-destructive">
              {preferencesError(error)}
            </p>
          )}
        </DialogContent>
      </Dialog>
      {adding && (
        <AddBackend
          identifiers={document?.profiles.map((profile) => profile.identifier) ?? []}
          restoreFocus={() => addButton.current?.focus()}
          endpoint={session?.snapshot.connection.endpoint ?? ""}
          onClose={() => setAdding(false)}
          onSaved={(value) => {
            setDocument(value);
            setAdding(false);
          }}
        />
      )}
    </SettingsCard>
  );
}

function AddBackend({
  identifiers,
  restoreFocus,
  endpoint: initialEndpoint,
  onClose,
  onSaved,
}: {
  identifiers: string[];
  restoreFocus: () => void;
  endpoint: string;
  onClose: () => void;
  onSaved: (document: Document) => void;
}) {
  const fieldId = useId();
  const identifierInput = useRef<HTMLInputElement>(null);
  const endpointInput = useRef<HTMLInputElement>(null);
  const [identifier, setIdentifier] = useState("");
  const [endpoint, setEndpoint] = useState(initialEndpoint);
  const [label, setLabel] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>();
  const [submitted, setSubmitted] = useState(false);
  const validation = backendValidation(identifier, endpoint, identifiers);
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(undefined);
    try {
      setSubmitted(true);
      if (Object.keys(validation).length) {
        (validation.identifier ? identifierInput : endpointInput).current?.focus();
        return;
      }
      setBusy(true);
      onSaved(
        await preferencesCommands.desktopBackendAdd({
          identifier: identifier.trim(),
          label: label.trim(),
          endpoint: endpoint.trim(),
          expected_backend_id: null,
          ca_certificate_pem: null,
          secret: "none",
        }),
      );
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) onClose();
      }}
    >
      <DialogContent
        className="sm:max-w-lg max-h-[90dvh] overflow-y-auto"
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          restoreFocus();
        }}
      >
        <DialogHeader>
          <DialogTitle>Add Backend</DialogTitle>
          <DialogDescription>Saved profiles are shared with the CLI. Access keys stay in the system keyring.</DialogDescription>
        </DialogHeader>
        <Button
          disabled={busy}
          onClick={async () => {
            setBusy(true);
            setError(undefined);
            try {
              const document = await preferencesCommands.desktopBackendImportPairing(identifier.trim() || null);
              if (document) onSaved(document);
            } catch (failure) {
              setError(failure);
            } finally {
              setBusy(false);
            }
          }}
        >
          Import Pairing File
        </Button>
        <p className="text-sm text-muted-foreground">Manual entry is for unsecured HTTP lab listeners.</p>
        <form onSubmit={submit} className="grid gap-3">
          <fieldset disabled={busy} className="grid gap-4 [&_label]:grid [&_label]:gap-2 [&_label]:text-sm">
            <label htmlFor={`${fieldId}-identifier`}>
              Identifier
              <Input
                className="font-mono"
                maxLength={128}
                pattern="(?!\.{1,2}$)[A-Za-z0-9._\-]+"
                ref={identifierInput}
                id={`${fieldId}-identifier`}
                aria-label="Backend Identifier"
                aria-invalid={submitted && Boolean(validation.identifier)}
                aria-describedby={submitted && validation.identifier ? `${fieldId}-identifier-error` : undefined}
                required
                value={identifier}
                onChange={(event) => setIdentifier(event.target.value)}
              />
              {submitted && validation.identifier && (
                <span id={`${fieldId}-identifier-error`} className="text-xs text-destructive">
                  {validation.identifier}
                </span>
              )}
            </label>
            <label htmlFor={`${fieldId}-label`}>
              Label
              <Input id={`${fieldId}-label`} required maxLength={256} value={label} onChange={(event) => setLabel(event.target.value)} />
            </label>
            <label htmlFor={`${fieldId}-endpoint`}>
              Endpoint
              <Input
                ref={endpointInput}
                id={`${fieldId}-endpoint`}
                aria-label="Backend Endpoint"
                required
                aria-invalid={submitted && Boolean(validation.endpoint)}
                aria-describedby={submitted && validation.endpoint ? `${fieldId}-endpoint-error` : undefined}
                placeholder="http://lab.example:8765"
                type="url"
                value={endpoint}
                onChange={(event) => setEndpoint(event.target.value)}
              />
              {submitted && validation.endpoint && (
                <span id={`${fieldId}-endpoint-error`} className="text-xs text-destructive">
                  {validation.endpoint}
                </span>
              )}
            </label>
            <div className="mt-2 flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit">{busy ? "Saving…" : "Add Backend"}</Button>
            </div>
          </fieldset>
          {Boolean(error) && <p role="alert">{preferencesError(error)}</p>}
        </form>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

export function backendValidation(
  identifier: string,
  endpoint: string,
  identifiers: readonly string[],
): { identifier?: string; endpoint?: string } {
  const errors: { identifier?: string; endpoint?: string } = {};
  if (!/^(?!\.{1,2}$)[A-Za-z0-9._-]{1,128}$/.test(identifier.trim())) errors.identifier = "Enter a backend identifier.";
  else if (identifiers.includes(identifier.trim())) errors.identifier = "A backend already uses this identifier.";
  try {
    const url = new URL(endpoint.trim());
    if (url.protocol !== "http:" || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
      errors.endpoint = "Enter an unsecured HTTP origin without credentials, a path, or a query.";
    }
  } catch {
    errors.endpoint = "Enter a valid endpoint, such as http://lab.example:8765.";
  }
  return errors;
}
