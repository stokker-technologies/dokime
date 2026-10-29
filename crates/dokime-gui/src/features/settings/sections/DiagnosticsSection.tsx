/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Download, TriangleAlert } from "lucide-react";
import { type FormEvent, useId, useState } from "react";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useNativeSessionController, useNativeSessionOptional } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { STORAGE_RESET_CONFIRMATION, storageResetAvailable, storageResetConfirmed } from "@/features/settings/storage-reset-model";
import { saveNativeDiagnosticsBundle } from "@/lib/api/native-delivery";
import { collectDiagnosticsBundle, diagnosticsBundleSupported } from "@/lib/api/native-diagnostics";
import { bootOrganization, organizationPack } from "@/lib/brand";
import { organizationPackError } from "@/lib/desktop/organization";
import { isTauriDesktop, openLocalPath } from "@/lib/desktop-open";
import { useSettingsFrameQuery } from "../settings-queries";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

export function DiagnosticsSection() {
  const session = useNativeSessionOptional();
  const settings = useSettingsFrameQuery(undefined, session !== null);
  const directory = settings.data?.bootstrap_paths?.storage;
  const resettable = storageResetAvailable({
    development: import.meta.env.DEV,
    desktop: isTauriDesktop(),
    ownership: session?.snapshot.ownership,
    directory,
  });
  return (
    <>
      <SettingsCard title="Support">
        <DiagnosticsBundleRow />
      </SettingsCard>
      <SettingsCard title="Runtime">
        <SettingsRow label="Organization Pack" error={organizationPackError(bootOrganization.error, organizationPack !== null)}>
          <span className="text-sm text-muted-foreground">
            {organizationPack ? `${organizationPack.label} pack applied` : "Compiled brand"}
          </span>
        </SettingsRow>
      </SettingsCard>
      {resettable && directory ? (
        <SettingsCard description="Shown only in development builds that own their local backend." title="Development">
          <StorageResetRow directory={directory} />
        </SettingsCard>
      ) : null}
    </>
  );
}

/**
 * The support archive, as an action rather than a path.
 *
 * What the operator does with this row is send the archive to the developers,
 * so the row hands them the archive: one button that pulls the zip and saves
 * it, and a toast that offers the folder it landed in. Where the backend
 * happened to build it is not their problem.
 *
 * The shell now has the one command this needs. `desktop_save_diagnostics_bundle`
 * takes the `diagnostics_bundle` kind inline — the archive is assembled here
 * from `Backend.CollectDiagnostics` frames and has no artifact, execution-file
 * or cycle id for the verified pull to address — and answers with the path the
 * operator chose. That answer is what the toast opens: the backend's filename
 * says nothing about which folder the picker landed in.
 */
export function DiagnosticsBundleRow() {
  const session = useNativeSessionOptional();
  const [busy, setBusy] = useState(false);
  const supported = diagnosticsBundleSupported();

  async function download() {
    if (!session || busy) return;
    setBusy(true);
    try {
      const bundle = await collectDiagnosticsBundle(session.identity);
      const path = await saveNativeDiagnosticsBundle(session.identity, {
        bytes: bundle.bytes,
        filename: bundle.filename,
      });
      toast.success("Diagnostics Bundle Saved", {
        action: {
          label: "Open Folder",
          onClick: () => void openLocalPath(path),
        },
      });
    } catch (failure: unknown) {
      toast.error("The Diagnostics Bundle Could Not Be Saved", {
        description: failure instanceof Error ? failure.message : String(failure),
      });
    } finally {
      setBusy(false);
    }
  }

  const button = (
    <Button disabled={!supported || !session || busy} onClick={() => void download()} size="sm" variant="outline">
      <Download data-icon="inline-start" />
      {busy ? "Collecting\u2026" : "Download Bundle"}
    </Button>
  );

  return (
    <SettingsRow
      description="Share this archive with the developers when reporting a problem. Secrets are redacted."
      label="Diagnostics Bundle"
    >
      {supported ? (
        button
      ) : (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-flex">{button}</span>
          </TooltipTrigger>
          <TooltipContent>Not Available On This Backend</TooltipContent>
        </Tooltip>
      )}
    </SettingsRow>
  );
}

/**
 * The only destructive action in the application, behind a typed confirmation.
 *
 * The dialog names the directory and arms only on the exact confirmation word.
 * Confirming hands the session to the connection screen, which shows the
 * shell's Stopping, Deleting and Starting steps and reconnects with fresh
 * queries, exactly as after any restart; a backend that refuses the empty root
 * leaves its diagnostics there like any failed start.
 */
function StorageResetRow({ directory }: { directory: string }) {
  const owner = useNativeSessionController();
  const inputId = useId();
  const [open, setOpen] = useState(false);
  const [typed, setTyped] = useState("");
  const armed = storageResetConfirmed(typed);

  function change(next: boolean) {
    setOpen(next);
    setTyped("");
  }

  function confirm(event: FormEvent) {
    event.preventDefault();
    if (!armed) return;
    change(false);
    void owner.resetLocalStorage(directory);
  }

  return (
    <SettingsRow
      description="Deletes every cycle, execution, record and file in the current data directory, then restarts the backend on an empty root."
      label="Reset Local Storage"
    >
      <Button onClick={() => change(true)} size="sm" variant="destructive">
        <TriangleAlert data-icon="inline-start" />
        Reset Local Storage
      </Button>
      <Dialog onOpenChange={change} open={open}>
        <DialogContent className="sm:max-w-lg">
          <form className="grid gap-4" onSubmit={confirm}>
            <DialogHeader>
              <DialogTitle>Reset Local Storage</DialogTitle>
              <DialogDescription>
                Every cycle, execution, record and file in this directory is permanently deleted. The backend then restarts on the empty
                directory. This cannot be undone.
              </DialogDescription>
            </DialogHeader>
            <code className="block rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 font-mono text-xs break-all text-destructive">
              {directory}
            </code>
            <div className="grid gap-2">
              <label className="text-sm" htmlFor={inputId}>
                Type <span className="font-mono font-semibold">{STORAGE_RESET_CONFIRMATION}</span> To Confirm
              </label>
              <Input autoComplete="off" id={inputId} onChange={(event) => setTyped(event.target.value)} spellCheck={false} value={typed} />
            </div>
            <DialogFooter className="gap-2">
              <Button onClick={() => change(false)} type="button" variant="outline">
                Cancel
              </Button>
              <Button disabled={!armed} type="submit" variant="destructive">
                Delete And Restart
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </SettingsRow>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
