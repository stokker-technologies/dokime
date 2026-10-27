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
  const failedProfile = document?.profiles.find((profile) => profile.name === (lastSelected.current ?? document.default));
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
  function connect(name: string) {
    if (busy) return;
    onConnect?.();
    lastSelected.current = name;
    void owner.select({ kind: "profile", name });
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
        getRowId={(profile) => profile.name}
        emptyText="No saved backends."
        columns={dataTableColumns<Profile>().columns([
          dataTableColumns<Profile>().accessor("name", { header: "Name", meta: { primary: true, filter: "text" } }),
          dataTableColumns<Profile>().accessor("endpoint", {
            header: "Endpoint",
            meta: { filter: "text", width: "w-[40%]", truncate: true },
          }),
          dataTableColumns<Profile>().accessor((profile) => (document?.default === profile.name ? "Last Used" : "—"), {
            id: "lastUsed",
            header: "Last Used",
            meta: { filter: "facet", width: "w-28" },
          }),
          dataTableColumns<Profile>().display({
            id: "actions",
            header: "Actions",
            meta: { width: "w-36" },
            cell: ({ row }) => (
              <DataTableRowActions
                label={row.original.name}
                primary={{ label: "Connect", disabled: busy, onSelect: () => connect(row.original.name) }}
              >
                <DropdownMenuItem variant="destructive" disabled={busy} onSelect={() => setDeleting(row.original.name)}>
                  Delete
                </DropdownMenuItem>
              </DataTableRowActions>
            ),
          }),
        ])}
        onRowClick={(profile) => connect(profile.name)}
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
          names={document?.profiles.map((profile) => profile.name) ?? []}
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
  names,
  restoreFocus,
  endpoint: initialEndpoint,
  onClose,
  onSaved,
}: {
  names: string[];
  restoreFocus: () => void;
  endpoint: string;
  onClose: () => void;
  onSaved: (document: Document) => void;
}) {
  const fieldId = useId();
  const nameInput = useRef<HTMLInputElement>(null);
  const endpointInput = useRef<HTMLInputElement>(null);
  const [name, setName] = useState("");
  const [endpoint, setEndpoint] = useState(initialEndpoint);
  const [identity, setIdentity] = useState("");
  const [ca, setCa] = useState("");
  const [secret, setSecret] = useState<Profile["secret"]>("none");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>();
  const [submitted, setSubmitted] = useState(false);
  const validation = backendValidation(name, endpoint, ca, names);
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(undefined);
    try {
      setSubmitted(true);
      if (Object.keys(validation).length) {
        (validation.name ? nameInput : endpointInput).current?.focus();
        return;
      }
      setBusy(true);
      onSaved(
        await preferencesCommands.desktopBackendAdd({
          name: name.trim(),
          endpoint: endpoint.trim(),
          expected_backend_id: identity.trim() || null,
          ca_certificate_pem: ca.trim() || null,
          secret,
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
        <form noValidate onSubmit={submit} className="grid gap-3">
          <fieldset disabled={busy} className="grid gap-4 [&_label]:grid [&_label]:gap-2 [&_label]:text-sm">
            <label htmlFor={`${fieldId}-name`}>
              Name
              <Input
                ref={nameInput}
                id={`${fieldId}-name`}
                aria-label="Backend Name"
                aria-invalid={submitted && Boolean(validation.name)}
                aria-describedby={submitted && validation.name ? `${fieldId}-name-error` : undefined}
                required
                value={name}
                onChange={(event) => setName(event.target.value)}
              />
              {submitted && validation.name && (
                <span id={`${fieldId}-name-error`} className="text-xs text-destructive">
                  {validation.name}
                </span>
              )}
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
                placeholder="https://backend.example.com"
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
            <details className="rounded-lg border p-3">
              <summary className="cursor-pointer text-sm font-medium focus-visible:outline-2 focus-visible:outline-ring">
                Advanced Connection Options
              </summary>
              <div className="mt-3 grid gap-3">
                <label htmlFor={`${fieldId}-identity`}>
                  Expected Backend ID
                  <Input
                    id={`${fieldId}-identity`}
                    aria-label="Expected Backend ID"
                    value={identity}
                    onChange={(event) => setIdentity(event.target.value)}
                  />
                </label>
                <label>
                  CA Certificate (PEM)
                  <textarea
                    className="w-full rounded-lg border bg-background p-2 font-mono text-xs focus-visible:outline-2 focus-visible:outline-ring"
                    aria-label="CA Certificate (PEM)"
                    value={ca}
                    onChange={(event) => setCa(event.target.value)}
                  />
                </label>
                <label>
                  Secret Reference
                  <select
                    className="w-full rounded-lg border bg-background p-2 focus-visible:outline-2 focus-visible:outline-ring"
                    aria-label="Secret Reference"
                    value={secret}
                    onChange={(event) => setSecret(event.target.value as Profile["secret"])}
                  >
                    <option value="none">None</option>
                    <option value="keyring">System Keyring</option>
                  </select>
                </label>
              </div>
            </details>
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
  name: string,
  endpoint: string,
  ca: string,
  names: readonly string[],
): { name?: string; endpoint?: string } {
  const errors: { name?: string; endpoint?: string } = {};
  if (!name.trim()) errors.name = "Enter a backend name.";
  else if (names.includes(name.trim())) errors.name = "A backend already uses this name.";
  try {
    const url = new URL(endpoint.trim());
    if (!["http:", "https:"].includes(url.protocol) || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
      errors.endpoint = "Enter an HTTP or HTTPS origin without credentials, a path, or a query.";
    } else if (ca.trim() && url.protocol !== "https:") errors.endpoint = "A custom CA certificate requires HTTPS.";
  } catch {
    errors.endpoint = "Enter a valid endpoint, such as https://backend.example.com.";
  }
  return errors;
}
