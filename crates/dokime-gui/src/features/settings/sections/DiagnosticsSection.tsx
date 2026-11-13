/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { failureOf } from "@/lib/notification-details";
import { Download, TriangleAlert } from "lucide-react";
import { useState } from "react";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { storageResetAvailable } from "@/features/settings/storage-reset-model";
import { saveNativeDiagnosticsBundle } from "@/lib/api/native-document-delivery";
import { collectDiagnosticsBundle, diagnosticsBundleSupported } from "@/lib/api/native-diagnostics";
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
 * so the row hands them the archive: one button that pulls the archive and saves
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
      toast.error(failureOf(failure, "The Diagnostics Bundle Could Not Be Saved"));
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

/** Recovery Waits for the Host's Preservation Contract. */
function StorageResetRow({ directory }: { directory: string }) {
  return (
    <SettingsRow
      description="Recovery is unavailable until the host can preserve plugins, settings, identity and secrets. Preserve this folder and select a fresh storage root."
      label="Reset Local Storage"
    >
      <code className="text-xs break-all">{directory}</code>
      <Button disabled size="sm" variant="destructive">
        <TriangleAlert data-icon="inline-start" />
        Reset Local Storage
      </Button>
    </SettingsRow>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
