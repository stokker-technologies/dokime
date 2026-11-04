/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Selected Base64 File Export And Explicit Import Conflict Review.
 */

/* === Imports === */

// Standard Library Imports

// Local Imports
import { MoreHorizontal } from "lucide-react";
import { useState } from "react";
// Third Party Imports
import { notificationMessage } from "@/lib/notification-details";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import {
  exportSecrets,
  importSecrets,
  pickSecretArchive,
  previewSecrets,
  type SecretCatalog,
  type SecretPreview,
} from "@/lib/api/native-secrets";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Props = { session: SessionIdentity; catalog: SecretCatalog; onImported: (catalog: SecretCatalog) => void };

/* === Components === */

export function SecretTransfer({ session, catalog, onImported }: Props) {
  const [mode, setMode] = useState<"export" | "import" | null>(null);
  const [generation, setGeneration] = useState(catalog.generation);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [data, setData] = useState<Uint8Array | null>(null);
  const [preview, setPreview] = useState<SecretPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState<string | null>(null);

  function open(next: typeof mode) {
    setMode(next);
    setGeneration(catalog.generation);
    setSelected(new Set());
    setData(null);
    setPreview(null);
    setError(null);
    setNotice(null);
  }

  function select(identifier: string, checked: boolean) {
    setSelected((previous) => {
      const next = new Set(previous);
      if (checked) next.add(identifier);
      else next.delete(identifier);
      return next;
    });
  }

  async function run(work: () => Promise<void>) {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await work();
    } catch (failure) {
      if (session.isCurrent()) setError(failure);
    } finally {
      if (session.isCurrent()) setBusy(false);
    }
  }

  async function submitExport() {
    await run(async () => {
      const path = await exportSecrets(session, [...selected], generation);
      if (session.isCurrent()) setNotice(path ? `Saved ${path}` : "Export cancelled");
    });
  }

  async function submitImport() {
    if (!data) return;
    await run(async () => {
      if (!preview) {
        const next = await previewSecrets(session, data);
        if (!session.isCurrent()) return;
        setPreview(next);
        setSelected(new Set(next.entries.filter((entry) => !entry.exists).map((entry) => entry.metadata.identifier)));
        return;
      }
      const next = await importSecrets(session, data, preview, selected);
      if (!session.isCurrent()) return;
      onImported(next);
      setData(null);
      setPreview(null);
      setNotice("Selected secrets imported");
    });
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm" aria-label="Secret Transfer Actions">
            <MoreHorizontal />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem disabled={busy || catalog.entries.length === 0} onSelect={() => open("export")}>
            Export Selected Secrets
          </DropdownMenuItem>
          <DropdownMenuItem disabled={busy} onSelect={() => open("import")}>
            Import Secrets
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {notice ? (
        <NotificationMessage source="Secrets" title="Secret Transfer Updated" tone="info">
          {notice}
        </NotificationMessage>
      ) : null}
      <Dialog
        open={mode !== null}
        onOpenChange={(isOpen) => {
          if (!isOpen && !busy) open(null);
        }}
      >
        <DialogContent className="max-h-[85dvh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{mode === "export" ? "Export Secrets" : "Import Secrets"}</DialogTitle>
          </DialogHeader>
          {Boolean(error) && <NotificationMessage error={error}>{notificationMessage(error)}</NotificationMessage>}

          {mode === "export" && (
            <form
              className="grid gap-3"
              onSubmit={(event) => {
                event.preventDefault();
                void submitExport();
              }}
            >
              <p className="text-sm text-muted-foreground">
                Choose identifiers to save in a Base64 .b64 file. Base64 is not encryption: anyone with the file can read its secrets. Each
                transfer holds at most 512 KiB of secret data and metadata.
              </p>
              <fieldset className="grid gap-2" disabled={busy}>
                <legend className="mb-2 text-sm font-medium">Identifiers To Export</legend>
                {catalog.entries.map((entry) => (
                  <label key={entry.metadata.identifier} className="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={selected.has(entry.metadata.identifier)}
                      onChange={(event) => select(entry.metadata.identifier, event.target.checked)}
                    />
                    {entry.metadata.identifier}
                  </label>
                ))}
              </fieldset>
              <div className="flex gap-2">
                <Button type="submit" disabled={busy || selected.size === 0}>
                  Save Secret File
                </Button>
                <Button type="button" disabled={busy} variant="outline" onClick={() => open(null)}>
                  Cancel
                </Button>
              </div>
            </form>
          )}
          {mode === "import" && (
            <form
              className="grid gap-3"
              onSubmit={(event) => {
                event.preventDefault();
                void submitImport();
              }}
            >
              <Button
                type="button"
                disabled={busy}
                variant="outline"
                onClick={() =>
                  void run(async () => {
                    setPreview(null);
                    setData(null);
                    const bytes = await pickSecretArchive(session);
                    if (session.isCurrent()) setData(bytes);
                  })
                }
              >
                Choose Secret Archive
              </Button>
              {data && <p className="text-sm">Secret archive selected ({data.byteLength} bytes)</p>}
              {preview && (
                <>
                  <p className="text-sm">
                    Import Into {preview.hostLabel} · {preview.hostId}
                  </p>
                  <fieldset className="grid gap-2" disabled={busy}>
                    <legend className="mb-2 text-sm font-medium">Review Every Identifier</legend>
                    {preview.entries.map((entry) => (
                      <label key={entry.metadata.identifier} className="flex items-center gap-2 text-sm">
                        <input
                          type="checkbox"
                          checked={selected.has(entry.metadata.identifier)}
                          onChange={(event) => select(entry.metadata.identifier, event.target.checked)}
                        />
                        {entry.exists ? "Replace" : "Import"} {entry.metadata.identifier} ({entry.metadata.label})
                      </label>
                    ))}
                  </fieldset>
                  <p className="text-sm text-muted-foreground">
                    Unchecked entries are skipped. Existing identifiers start unchecked. Apply only the identifiers you have reviewed.
                  </p>
                </>
              )}
              <div className="flex gap-2">
                <Button type="submit" disabled={busy || !data}>
                  {preview ? "Apply Reviewed Import" : "Preview Import"}
                </Button>
                <Button type="button" disabled={busy} variant="outline" onClick={() => open(null)}>
                  Cancel
                </Button>
              </div>
            </form>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
