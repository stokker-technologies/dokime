/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Selected Encrypted Export And Explicit Import Conflict Review.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useId, useState } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
  const id = useId();
  const [mode, setMode] = useState<"export" | "import" | null>(null);
  const [generation, setGeneration] = useState(catalog.generation);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [ciphertext, setCiphertext] = useState<Uint8Array | null>(null);
  const [preview, setPreview] = useState<SecretPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  function open(next: typeof mode) {
    setMode(next);
    setGeneration(catalog.generation);
    setSelected(new Set());
    setPassword("");
    setConfirmation("");
    setCiphertext(null);
    setPreview(null);
    setError(null);
    setNotice(null);
  }

  function select(name: string, checked: boolean) {
    setSelected((previous) => {
      const next = new Set(previous);
      if (checked) next.add(name);
      else next.delete(name);
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
      if (session.isCurrent()) setError(failure instanceof Error ? failure.message : "Secret transfer failed");
    } finally {
      if (session.isCurrent()) setBusy(false);
    }
  }

  async function submitExport() {
    if (password !== confirmation) {
      setError("Passwords do not match");
      return;
    }
    const entered = password;
    setPassword("");
    setConfirmation("");
    await run(async () => {
      const path = await exportSecrets(session, [...selected], entered, generation);
      if (session.isCurrent()) setNotice(path ? `Saved ${path}` : "Export cancelled");
    });
  }

  async function submitImport() {
    if (!ciphertext) return;
    const entered = password;
    setPassword("");
    await run(async () => {
      if (!preview) {
        const next = await previewSecrets(session, ciphertext, entered);
        if (!session.isCurrent()) return;
        setPreview(next);
        setSelected(new Set(next.entries.filter((entry) => !entry.exists).map((entry) => entry.metadata.name)));
        return;
      }
      const next = await importSecrets(session, ciphertext, entered, preview, selected);
      if (!session.isCurrent()) return;
      onImported(next);
      setCiphertext(null);
      setPreview(null);
      setNotice("Selected secrets imported");
    });
  }

  return (
    <section className="grid gap-3 border-t pt-3" aria-label="Encrypted Secret Transfer">
      <div className="flex flex-wrap gap-2">
        <Button disabled={busy || catalog.entries.length === 0} variant="outline" onClick={() => open("export")}>
          Export Selected Secrets
        </Button>
        <Button disabled={busy} variant="outline" onClick={() => open("import")}>
          Import Encrypted Secrets
        </Button>
      </div>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      {notice && (
        <p role="status" className="text-sm">
          {notice}
        </p>
      )}
      {mode === "export" && (
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            void submitExport();
          }}
        >
          <p className="text-sm text-muted-foreground">
            Choose names and a strong password. Only the encrypted .age file is saved. Each transfer holds at most 512 KiB of secret data
            and metadata.
          </p>
          <fieldset className="grid gap-2" disabled={busy}>
            <legend className="mb-2 text-sm font-medium">Names To Export</legend>
            {catalog.entries.map((entry) => (
              <label key={entry.metadata.name} className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={selected.has(entry.metadata.name)}
                  onChange={(event) => select(entry.metadata.name, event.target.checked)}
                />
                {entry.metadata.name}
              </label>
            ))}
          </fieldset>
          <Label htmlFor={`${id}-export-password`}>Export Password</Label>
          <Input
            id={`${id}-export-password`}
            type="password"
            autoComplete="new-password"
            required
            maxLength={1024}
            disabled={busy}
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
          <Label htmlFor={`${id}-confirm`}>Confirm Password</Label>
          <Input
            id={`${id}-confirm`}
            type="password"
            autoComplete="new-password"
            required
            maxLength={1024}
            disabled={busy}
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
          />
          <div className="flex gap-2">
            <Button type="submit" disabled={busy || selected.size === 0}>
              Encrypt And Save
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
                setCiphertext(null);
                setPassword("");
                const bytes = await pickSecretArchive(session);
                if (session.isCurrent()) setCiphertext(bytes);
              })
            }
          >
            Choose Encrypted Archive
          </Button>
          {ciphertext && <p className="text-sm">Encrypted archive selected ({ciphertext.byteLength} bytes)</p>}
          {preview && (
            <>
              <p className="text-sm">
                Import Into {preview.hostLabel} · {preview.hostId}
              </p>
              <fieldset className="grid gap-2" disabled={busy}>
                <legend className="mb-2 text-sm font-medium">Review Every Name</legend>
                {preview.entries.map((entry) => (
                  <label key={entry.metadata.name} className="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={selected.has(entry.metadata.name)}
                      onChange={(event) => select(entry.metadata.name, event.target.checked)}
                    />
                    {entry.exists ? "Replace" : "Import"} {entry.metadata.name} ({entry.metadata.label})
                  </label>
                ))}
              </fieldset>
              <p className="text-sm text-muted-foreground">
                Unchecked entries are skipped. Existing names start unchecked. Re-enter the password to apply this reviewed selection.
              </p>
            </>
          )}
          <Label htmlFor={`${id}-import-password`}>{preview ? "Password (Enter Again To Import)" : "Archive Password"}</Label>
          <Input
            id={`${id}-import-password`}
            type="password"
            autoComplete="off"
            required
            maxLength={1024}
            disabled={busy || !ciphertext}
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
          <div className="flex gap-2">
            <Button type="submit" disabled={busy || !ciphertext}>
              {preview ? "Apply Reviewed Import" : "Authenticate And Preview"}
            </Button>
            <Button type="button" disabled={busy} variant="outline" onClick={() => open(null)}>
              Cancel
            </Button>
          </div>
        </form>
      )}
    </section>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
