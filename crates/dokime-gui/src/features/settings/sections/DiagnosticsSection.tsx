/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Download } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useDiagnosticsQuery } from "@/features/cycle/server-state";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsPathField } from "@/features/settings/SettingsPathField";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { saveNativeDiagnosticsBundle } from "@/lib/api/native-delivery";
import { collectDiagnosticsBundle, diagnosticsBundleSupported } from "@/lib/api/native-diagnostics";
import { useApplicationSettings } from "@/lib/application-settings";
import { isTauriDesktop, openLocalPath } from "@/lib/desktop-open";
import { formatBytes } from "@/lib/format";
import type { DiagnosticPathKey } from "@/lib/types/view-contracts";

/* === Constants === */

/* Only the conventions the backend resolves to a real location are shown; the
   cycle and execution patterns are templates, not paths. The bundle is not
   among them: an operator sending an archive to the developers needs the
   archive, not the directory it was written in, so it has an action row of its
   own below. */
const pathConventionLabels: Record<Exclude<DiagnosticPathKey, "collect_bundle">, string> = {
  cycle_log_pattern: "Cycle Log Pattern",
  diagnostics_snapshot: "Diagnostics Snapshot",
  execution_log_pattern: "Execution Log Pattern",
  hub_log: "Hub Log",
  startup_doctor: "Startup Doctor Report",
};

/* === Private Functions === */

function startupLabel(status: string): string {
  if (status === "ok") {
    return "Healthy";
  }
  if (status === "warning") {
    return "Warning";
  }
  if (status === "failed") {
    return "Failed";
  }
  return status;
}

function startupBadgeClass(status: string): string {
  if (status === "ok") {
    return "border-success/30 bg-success/10 text-success";
  }
  if (status === "warning") {
    return "border-warning/30 bg-warning/10 text-warning";
  }
  return "border-destructive/40 bg-destructive/10 text-destructive";
}

/* === Components === */

export function DiagnosticsSection() {
  const diagnosticsQuery = useDiagnosticsQuery();
  const session = useNativeSessionOptional();
  const applicationSettings = useApplicationSettings();
  const snapshot = diagnosticsQuery.data ?? null;

  /* Only an owned local backend shares this machine's file system. */
  const openable = isTauriDesktop() && session?.snapshot?.ownership === "owned_local";
  const applicationRoot = applicationSettings.data?.applicationRoot;
  const settingsPath = applicationRoot ? `${applicationRoot}/settings.json` : null;

  if (diagnosticsQuery.isPending) {
    return (
      <SettingsCard title="Diagnostics">
        <SettingsRow label="Status">
          <span className="text-sm text-muted-foreground">Loading…</span>
        </SettingsRow>
      </SettingsCard>
    );
  }

  if (diagnosticsQuery.isError || !snapshot) {
    return (
      <SettingsCard title="Diagnostics">
        <SettingsRow
          description={
            diagnosticsQuery.error instanceof Error
              ? diagnosticsQuery.error.message
              : "Diagnostics are unavailable. Is the Test Hub server running?"
          }
          label="Status"
        />
      </SettingsCard>
    );
  }

  const { runtime, server } = snapshot;
  const counts = runtime.counts;
  const process = server.process;
  const rootPaths = runtime.path_conventions.filter(
    (convention) =>
      convention.scope === "root" &&
      convention.absolute_path !== null &&
      convention.key !== "collect_bundle",
  );

  return (
    <>
      <SettingsCard title="Server">
        <SettingsRow label="Status">
          <Badge className={startupBadgeClass(runtime.startup_doctor.status)}>
            {startupLabel(runtime.startup_doctor.status)}
          </Badge>
        </SettingsRow>
        <SettingsRow label="Version">
          <span className="font-mono text-sm">v{server.version}</span>
        </SettingsRow>
        <SettingsRow label="Process">
          <span className="text-sm text-muted-foreground">
            PID {process.pid}
            {process.cpu_percent != null ? ` · ${process.cpu_percent.toFixed(1)}% CPU` : ""}
            {process.memory_bytes != null ? ` · ${formatBytes(process.memory_bytes)}` : ""}
          </span>
        </SettingsRow>
      </SettingsCard>

      <SettingsCard title="Support">
        <DiagnosticsBundleRow />
      </SettingsCard>

      <SettingsCard title="Runtime">
        <SettingsRow label="Installed Plugins">
          <span className="text-sm text-muted-foreground">
            {counts.test_suites} suites
            {counts.installed_plugin_errors
              ? ` · ${counts.installed_plugin_errors} errors`
              : " · no errors"}
          </span>
        </SettingsRow>
      </SettingsCard>

      <SettingsCard
        description="Where this installation keeps its state, on the machine running the backend."
        title="Paths"
      >
        <SettingsRow label="Storage Root">
          <SettingsPathField openable={openable} value={runtime.storage_root} />
        </SettingsRow>
        {rootPaths.map((convention) => (
          <SettingsRow
            key={convention.key}
            label={
              pathConventionLabels[convention.key as Exclude<DiagnosticPathKey, "collect_bundle">]
            }
          >
            <SettingsPathField openable={openable} value={convention.absolute_path} />
          </SettingsRow>
        ))}
        {settingsPath ? (
          <SettingsRow label="Settings File">
            <SettingsPathField openable={openable} value={settingsPath} />
          </SettingsRow>
        ) : null}
      </SettingsCard>
    </>
  );
}

/* === Subcomponents === */

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
      toast.success("Diagnostics bundle saved", {
        action: {
          label: "Open Folder",
          onClick: () => void openLocalPath(path),
        },
      });
    } catch (failure: unknown) {
      toast.error("The diagnostics bundle could not be saved.", {
        description: failure instanceof Error ? failure.message : String(failure),
      });
    } finally {
      setBusy(false);
    }
  }

  const button = (
    <Button
      disabled={!supported || !session || busy}
      onClick={() => void download()}
      size="sm"
      variant="outline"
    >
      <Download data-icon="inline-start" />
      {busy ? "Collecting\u2026" : "Download Bundle"}
    </Button>
  );

  return (
    <SettingsRow
      description="Share this archive with the developers when reporting a problem. Credentials are redacted."
      label="Diagnostics Bundle"
    >
      {supported ? (
        button
      ) : (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-flex">{button}</span>
          </TooltipTrigger>
          <TooltipContent>Not available on this backend</TooltipContent>
        </Tooltip>
      )}
    </SettingsRow>
  );
}
