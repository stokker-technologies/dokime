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
import { toast } from "sonner";

/* === Local Imports === */

import { titleCaseIdentifier } from "@/components/start-cycle-form-model";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginConsentDialog } from "@/features/settings/plugins/PluginConsentDialog";
import { PluginDropZone } from "@/features/settings/plugins/PluginDropZone";
import { PluginList } from "@/features/settings/plugins/PluginList";
import { PluginPrograms } from "@/features/settings/plugins/PluginPrograms";
import {
  useConsentPluginMutation,
  usePluginReleaseMutation,
  usePluginsQuery,
  useRemovePluginMutation,
  useRestartAppMutation,
  useSetPluginEnabledMutation,
  useStagePluginMutation,
} from "@/features/settings/plugins/plugin-queries";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useSettingsSections } from "@/features/settings/use-settings-sections";
import type { PluginSource } from "@/lib/api/native-plugins";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Components === */

export function PluginsSection() {
  const brand = useBrand();
  const desktop = isTauriDesktop();
  const navigate = useNavigate();
  const settings = useSettingsSections();
  const pluginsQuery = usePluginsQuery();
  const programsQuery = useInstalledPluginsQuery();
  const stageMutation = useStagePluginMutation();
  const releaseMutation = usePluginReleaseMutation();
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
    if (owner?.kind === "bundle" && owner.installation && entries.some((entry) => entry.install_id === owner.installation)) continue;
    const name =
      owner?.kind === "host"
        ? `${brand.label} Host`
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
          .filter((program) => program.owner?.kind === "bundle" && program.owner.installation === entry.install_id)
          .map((program) => program.plugin_id),
      );
      const section = (settings.frame?.sections ?? []).find(
        (candidate) => candidate.owner.kind === "installed_plugin" && artifacts.has(candidate.owner.plugin_id),
      );
      return section?.section_id ?? null;
    },
    [programs, settings.frame],
  );

  async function handleStage(source: PluginSource, filename: string | null): Promise<void> {
    stageMutation.reset();
    consentMutation.reset();
    setUploadingFilename(filename);
    try {
      const entry = await stageMutation.mutateAsync(source);
      if (entry) setStaged(entry);
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
      await consentMutation.mutateAsync(staged);
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
    <>
      <SettingsCard
        title="Install A Plugin"
        description={`Install and manage .dokime plugin bundles. Files are staged for static inspection before ${brand.label} asks for consent, and new installs activate after the next restart.`}
      >
        <div className="px-4 py-3">
          <PluginDropZone
            disabled={stageMutation.isPending}
            error={errorMessage(stageMutation.error, "The plugin could not be inspected.")}
            filename={uploadingFilename}
            onDrop={(filename) => void handleStage("drop", filename)}
            onPick={() => void handleStage("pick", null)}
            progress={stageMutation.progress}
          />
        </div>
        <SettingsRow
          label="Organization Release Source"
          description={
            releaseMutation.data
              ? `${releaseMutation.data.package} ${releaseMutation.data.version} · ${releaseMutation.data.target} · Dokime ${releaseMutation.data.requiresDokime}`
              : "Discover a compatible plugin from the source selected under Updates."
          }
          error={releaseMutation.isError ? errorMessage(releaseMutation.error, "The release feed could not be checked.") : null}
        >
          <Button
            variant="outline"
            size="sm"
            disabled={!desktop || releaseMutation.isPending || stageMutation.isPending}
            onClick={() => releaseMutation.mutate()}
          >
            Check Plugin Release
          </Button>
          {releaseMutation.data ? (
            <Button
              size="sm"
              disabled={stageMutation.isPending}
              onClick={() => {
                const release = releaseMutation.data;
                if (release) void handleStage({ release: release.candidate }, release.package);
              }}
            >
              Inspect Release
            </Button>
          ) : null}
        </SettingsRow>
        {restartPending ? (
          <SettingsRow
            label="Restart Required"
            description={`Restart ${brand.label} to apply pending plugin changes.`}
            error={restartMutation.isError ? errorMessage(restartMutation.error, `${brand.label} could not be restarted.`) : null}
          >
            {desktop ? (
              <Button disabled={restartMutation.isPending} onClick={() => setRestartPrompt(true)} size="sm" variant="outline">
                <RotateCcw className={restartMutation.isPending ? "animate-spin" : undefined} />
                {restartMutation.isPending ? "Restarting…" : `Restart ${brand.label}`}
              </Button>
            ) : (
              <p className="text-sm text-muted-foreground">
                Restart <code className="font-mono text-foreground">dokime server</code> to apply.
              </p>
            )}
          </SettingsRow>
        ) : null}
      </SettingsCard>

      {entries.length > 0 ||
      pluginsQuery.isPending ||
      pluginsQuery.isError ||
      !programs.some((program) => program.owner?.kind === "bundle") ? (
        <SettingsCard span="full" title="Installed Plugins" description="Active state reflects this application process.">
          <div className="px-4 py-3">
            {pluginsQuery.isPending ? (
              <div className="grid min-h-44 place-items-center text-sm text-muted-foreground">
                <span className="inline-flex items-center gap-2">
                  <Loader2 className="size-4 animate-spin" />
                  Loading plugins…
                </span>
              </div>
            ) : pluginsQuery.isError ? (
              <div className="grid min-h-44 place-items-center gap-3 p-6 text-center">
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
                    {
                      onSuccess: () => setRestartPrompt(true),
                      onError: (error) =>
                        toast.error("Plugin Change Failed", {
                          description: errorMessage(error, "The plugin change could not be saved.") ?? undefined,
                        }),
                    },
                  )
                }
                onOpenSettings={(entry) => {
                  const pluginId = settingsSectionOf(entry);
                  if (pluginId) void navigate({ params: { pluginId }, to: "/settings/plugins/$pluginId" });
                }}
                settingsSectionOf={settingsSectionOf}
                onRemove={setRemoveCandidate}
                removingId={removeMutation.isPending ? (removeMutation.variables ?? null) : null}
              />
            )}
          </div>
        </SettingsCard>
      ) : null}
      {programsQuery.isError || programsQuery.isPending ? (
        <SettingsCard title="Programs">
          <SettingsRow
            label="Programs"
            error={programsQuery.isError ? errorMessage(programsQuery.error, "Program settings could not be loaded.") : null}
          >
            {programsQuery.isPending ? (
              <span className="text-sm text-muted-foreground" role="status">
                Loading programs…
              </span>
            ) : (
              <Button onClick={() => void programsQuery.refetch()} size="sm" variant="outline">
                Reload Programs
              </Button>
            )}
          </SettingsRow>
        </SettingsCard>
      ) : null}
      {Array.from(standalone, ([name, members]) => (
        <SettingsCard
          key={name}
          span="full"
          title={name}
          description={
            members[0]?.owner?.kind === "host"
              ? `Built-in ${brand.label} features.`
              : members[0]?.owner?.kind === "bundle"
                ? "Loaded from the launch configuration; managed outside the installation store."
                : members[0]?.owner?.kind === "embedded"
                  ? "Registered by the application that started this backend."
                  : "Catalog definitions without provider ownership."
          }
        >
          <div className="px-4 py-3">
            <PluginPrograms programs={members} />
          </div>
        </SettingsCard>
      ))}
      <Dialog open={restartPrompt} onOpenChange={setRestartPrompt}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Restart {brand.label}?</DialogTitle>
            <DialogDescription>
              Plugin changes are saved. Restart {brand.label} to apply them. Active work will stop and plugin resources will close before
              the application restarts.
            </DialogDescription>
          </DialogHeader>
          {restartMutation.isError ? (
            <p role="alert" className="text-sm text-destructive">
              {errorMessage(restartMutation.error, `${brand.label} could not be restarted.`)}
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
              <p>Restart dokime server to apply these changes.</p>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <PluginConsentDialog
        busy={consentMutation.isPending || (removeMutation.isPending && Boolean(staged))}
        entry={staged}
        error={errorMessage(consentMutation.error ?? (staged ? removeMutation.error : null), "The plugin action could not be completed.")}
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
                : `The plugin will stop loading and its managed file will be deleted at the next ${brand.label} restart.`}
            </DialogDescription>
          </DialogHeader>
          {removeMutation.isError && removeCandidate ? (
            <p className="text-sm text-destructive" role="alert">
              {errorMessage(removeMutation.error, "The plugin could not be removed.")}
            </p>
          ) : null}
          <DialogFooter>
            <Button disabled={removeMutation.isPending} onClick={() => setRemoveCandidate(null)} type="button" variant="outline">
              Cancel
            </Button>
            <Button disabled={removeMutation.isPending} onClick={() => void handleRemove()} type="button" variant="destructive">
              {removeMutation.isPending ? "Removing…" : "Remove Plugin"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

/* === Private Functions === */

function errorMessage(error: unknown, fallback: string): string | null {
  if (!error) {
    return null;
  }
  return error instanceof Error && error.message.trim() ? error.message : fallback;
}
