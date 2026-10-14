/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { Loader2, RotateCcw } from "lucide-react";
import { useCallback, useState } from "react";

/* === Local Imports === */

import { titleCaseIdentifier } from "@/components/start-cycle-form-model";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginConsentDialog } from "@/features/settings/plugins/PluginConsentDialog";
import { PluginDropZone } from "@/features/settings/plugins/PluginDropZone";
import { PluginList } from "@/features/settings/plugins/PluginList";
import { PluginPrograms } from "@/features/settings/plugins/PluginPrograms";
import {
  useConsentPluginMutation,
  usePluginsQuery,
  useRemovePluginMutation,
  useRestartAppMutation,
  useSetPluginEnabledMutation,
  useStagePluginMutation,
} from "@/features/settings/plugins/plugin-queries";
import { useSettingsSections } from "@/features/settings/use-settings-sections";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Components === */

export function PluginsSection() {
  const desktop = isTauriDesktop();
  const navigate = useNavigate();
  const settings = useSettingsSections();
  const pluginsQuery = usePluginsQuery();
  const programsQuery = useInstalledPluginsQuery();
  const stageMutation = useStagePluginMutation();
  const consentMutation = useConsentPluginMutation();
  const removeMutation = useRemovePluginMutation();
  const restartMutation = useRestartAppMutation();
  const enabledMutation = useSetPluginEnabledMutation();
  const [restartPrompt, setRestartPrompt] = useState(false);
  const [staged, setStaged] = useState<PluginEntry | null>(null);
  const [uploadingFilename, setUploadingFilename] = useState<string | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<PluginEntry | null>(null);
  const entries = pluginsQuery.data ?? [];
  const programs = programsQuery.data ?? [];
  const standalone = new Map<string, typeof programs>();
  for (const program of programs) {
    const owner = program.owner;
    if (
      owner?.kind === "bundle" &&
      owner.installation &&
      entries.some((entry) => entry.install_id === owner.installation)
    )
      continue;
    const name =
      owner?.kind === "host"
        ? "Dokime Host"
        : owner?.kind === "bundle"
          ? titleCaseIdentifier(owner.package)
          : owner?.kind === "embedded"
            ? "Embedded Providers"
            : "Unclassified Catalogs";
    const members = standalone.get(name) ?? [];
    members.push(program);
    standalone.set(name, members);
  }
  const restartPending = entries.some((entry) => entry.restart_required);

  /* A plugin's settings page is its contributed section. The frame keys those
     by installed plugin, and an artifact names the installation it came
     from, so the bundle reaches its own page through its programs. */
  const settingsSectionOf = useCallback(
    (entry: PluginEntry): string | null => {
      const artifacts = new Set(
        programs
          .filter(
            (program) =>
              program.owner?.kind === "bundle" && program.owner.installation === entry.install_id,
          )
          .map((program) => program.plugin_id),
      );
      const section = (settings.frame?.sections ?? []).find(
        (candidate) =>
          candidate.owner.kind === "installed_plugin" && artifacts.has(candidate.owner.plugin_id),
      );
      return section?.section_id ?? null;
    },
    [programs, settings.frame],
  );

  async function handleFile(file: File): Promise<void> {
    stageMutation.reset();
    consentMutation.reset();
    setUploadingFilename(file.name);
    try {
      const entry = await stageMutation.mutateAsync(file);
      setStaged(entry);
    } catch {
      // Mutation error is rendered beside the drop zone.
    } finally {
      setUploadingFilename(null);
    }
  }

  async function handleCancelConsent(): Promise<void> {
    if (!staged) {
      return;
    }
    consentMutation.reset();
    try {
      await removeMutation.mutateAsync(staged.install_id);
      setStaged(null);
    } catch {
      // Keep the dialog open so the operator can retry or read the failure.
    }
  }

  async function handleInstall(): Promise<void> {
    if (!staged) {
      return;
    }
    consentMutation.reset();
    try {
      await consentMutation.mutateAsync(staged.install_id);
      setStaged(null);
      setRestartPrompt(true);
    } catch {
      // Keep the inspected facts and failure visible for a retry.
    }
  }

  async function handleRemove(): Promise<void> {
    if (!removeCandidate) {
      return;
    }
    try {
      await removeMutation.mutateAsync(removeCandidate.install_id);
      setRemoveCandidate(null);
      if (removeCandidate.consented_at !== null) setRestartPrompt(true);
    } catch {
      // Keep explicit confirmation open with the error visible.
    }
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="grid gap-[var(--section-gap)] pb-4">
        <header>
          <h1 className="text-lg font-semibold">Plugins</h1>
          <p className="mt-1 max-w-[var(--prose-measure)] text-sm text-muted-foreground">
            Install and manage .dokime plugin bundles. New installs activate only after the next
            Dokime restart.
          </p>
        </header>

        <section className="grid gap-3" aria-labelledby="plugin-install-title">
          <div>
            <h2 className="text-sm font-semibold" id="plugin-install-title">
              Install A Plugin
            </h2>
            <p className="mt-0.5 text-xs text-muted-foreground">
              Files are staged for static inspection before Dokime asks for consent.
            </p>
          </div>
          <PluginDropZone
            disabled={stageMutation.isPending}
            error={errorMessage(stageMutation.error, "The plugin could not be inspected.")}
            filename={uploadingFilename}
            onFile={(file) => void handleFile(file)}
            progress={stageMutation.progress}
          />
        </section>

        {restartPending ? (
          <div className="flex flex-col gap-3 rounded-lg border border-warning/30 bg-warning/10 p-4 sm:flex-row sm:items-center">
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium">Restart Required</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                Restart Dokime to apply pending plugin changes.
              </p>
            </div>
            {desktop ? (
              <Button
                disabled={restartMutation.isPending}
                onClick={() => setRestartPrompt(true)}
                size="sm"
                variant="outline"
              >
                <RotateCcw className={restartMutation.isPending ? "animate-spin" : undefined} />
                {restartMutation.isPending ? "Restarting…" : "Restart Dokime"}
              </Button>
            ) : (
              <p className="text-sm text-muted-foreground">
                Restart <code className="font-mono text-foreground">dokime serve</code> to apply.
              </p>
            )}
            {restartMutation.isError ? (
              <p className="text-sm text-destructive" role="alert">
                {errorMessage(restartMutation.error, "Dokime could not be restarted.")}
              </p>
            ) : null}
          </div>
        ) : null}

        {entries.length > 0 ||
        pluginsQuery.isPending ||
        pluginsQuery.isError ||
        !programs.some((program) => program.owner?.kind === "bundle") ? (
          <section className="grid gap-3" aria-labelledby="installed-plugins-title">
            <div>
              <h2 className="text-sm font-semibold" id="installed-plugins-title">
                Installed Plugins
              </h2>
              <p className="mt-0.5 text-xs text-muted-foreground">
                Active state reflects this application process.
              </p>
            </div>

            {pluginsQuery.isPending ? (
              <div className="grid min-h-44 place-items-center rounded-lg border text-sm text-muted-foreground">
                <span className="inline-flex items-center gap-2">
                  <Loader2 className="size-4 animate-spin" />
                  Loading plugins…
                </span>
              </div>
            ) : pluginsQuery.isError ? (
              <div className="grid min-h-44 place-items-center gap-3 rounded-lg border p-6 text-center">
                <div>
                  <p className="font-medium">Plugins could not be loaded</p>
                  <p className="mt-1 text-sm text-destructive" role="alert">
                    {errorMessage(pluginsQuery.error, "Try loading the plugin registry again.")}
                  </p>
                </div>
                <Button onClick={() => void pluginsQuery.refetch()} size="sm" variant="outline">
                  Try Again
                </Button>
              </div>
            ) : (
              <PluginList
                entries={entries}
                changingEnabled={enabledMutation.isPending}
                onSetEnabled={(entry, enabled) =>
                  enabledMutation.mutate(
                    { installId: entry.install_id, enabled },
                    { onSuccess: () => setRestartPrompt(true) },
                  )
                }
                onOpenSettings={(entry) => {
                  const pluginId = settingsSectionOf(entry);
                  if (pluginId)
                    void navigate({ params: { pluginId }, to: "/settings/plugins/$pluginId" });
                }}
                settingsSectionOf={settingsSectionOf}
                onRemove={setRemoveCandidate}
                removingId={removeMutation.isPending ? (removeMutation.variables ?? null) : null}
              />
            )}
          </section>
        ) : null}
        {programsQuery.isError ? (
          <div className="grid justify-items-start gap-2">
            <p role="alert" className="text-sm text-destructive">
              {errorMessage(programsQuery.error, "Program settings could not be loaded.")}
            </p>
            <Button onClick={() => void programsQuery.refetch()} size="sm" variant="outline">
              Reload Programs
            </Button>
          </div>
        ) : null}
        {programsQuery.isPending ? <p className="text-sm">Loading programs…</p> : null}
        {Array.from(standalone, ([name, members]) => (
          <section className="grid gap-3" key={name}>
            <div>
              <h2 className="text-sm font-semibold">{name}</h2>
              <p className="text-xs text-muted-foreground">
                {members[0]?.owner?.kind === "host"
                  ? "Built-in Dokime features."
                  : members[0]?.owner?.kind === "bundle"
                    ? "Loaded from the launch configuration; managed outside the installation store."
                    : members[0]?.owner?.kind === "embedded"
                      ? "Registered by the application that started this backend."
                      : "Catalog definitions without provider ownership."}
              </p>
            </div>
            <PluginPrograms programs={members} />
          </section>
        ))}
      </div>

      {enabledMutation.isError ? (
        <p role="alert" className="text-sm text-destructive">
          {errorMessage(enabledMutation.error, "The plugin change could not be saved.")}
        </p>
      ) : null}
      <Dialog open={restartPrompt} onOpenChange={setRestartPrompt}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Restart Dokime?</DialogTitle>
            <DialogDescription>
              Plugin changes are saved. Restart Dokime to apply them. Active work will stop and
              plugin resources will close before the application restarts.
            </DialogDescription>
          </DialogHeader>
          {restartMutation.isError ? (
            <p role="alert" className="text-sm text-destructive">
              {errorMessage(restartMutation.error, "Dokime could not be restarted.")}
            </p>
          ) : null}
          <DialogFooter>
            <Button variant="outline" onClick={() => setRestartPrompt(false)}>
              Later
            </Button>
            {desktop ? (
              <Button disabled={restartMutation.isPending} onClick={() => restartMutation.mutate()}>
                Restart Now
              </Button>
            ) : (
              <p>Restart dokime serve to apply these changes.</p>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <PluginConsentDialog
        busy={consentMutation.isPending || (removeMutation.isPending && Boolean(staged))}
        entry={staged}
        error={errorMessage(
          consentMutation.error ?? (staged ? removeMutation.error : null),
          "The plugin action could not be completed.",
        )}
        onCancel={() => void handleCancelConsent()}
        onInstall={() => void handleInstall()}
      />

      <Dialog
        onOpenChange={(open) => {
          if (!open && !removeMutation.isPending) {
            setRemoveCandidate(null);
          }
        }}
        open={Boolean(removeCandidate)}
      >
        <DialogContent showCloseButton={!removeMutation.isPending}>
          <DialogHeader>
            <DialogTitle>Remove Plugin?</DialogTitle>
            <DialogDescription>
              {removeCandidate?.consented_at === null
                ? "The staged bundle will be discarded immediately. No restart is needed."
                : "The plugin will stop loading and its managed file will be deleted at the next Dokime restart."}
            </DialogDescription>
          </DialogHeader>
          {removeMutation.isError && removeCandidate ? (
            <p className="text-sm text-destructive" role="alert">
              {errorMessage(removeMutation.error, "The plugin could not be removed.")}
            </p>
          ) : null}
          <DialogFooter>
            <Button
              disabled={removeMutation.isPending}
              onClick={() => setRemoveCandidate(null)}
              type="button"
              variant="outline"
            >
              Cancel
            </Button>
            <Button
              disabled={removeMutation.isPending}
              onClick={() => void handleRemove()}
              type="button"
              variant="destructive"
            >
              {removeMutation.isPending ? "Removing…" : "Remove Plugin"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/* === Private Functions === */

function errorMessage(error: unknown, fallback: string): string | null {
  if (!error) {
    return null;
  }
  return error instanceof Error && error.message.trim() ? error.message : fallback;
}
