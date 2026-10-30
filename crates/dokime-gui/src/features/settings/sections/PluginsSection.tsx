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
import { NotificationMessage } from "@/components/NotificationMessage";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginDropZone } from "@/features/settings/plugins/PluginDropZone";
import { PluginList } from "@/features/settings/plugins/PluginList";
import { bundleSettingsPage, programRows, programSource } from "@/features/settings/plugins/PluginPrograms";
import type { PluginTableRow } from "@/features/settings/plugins/PluginTable";
import {
  useInstallPluginMutation,
  usePluginReleaseMutation,
  usePluginsQuery,
  useRemovePluginMutation,
  useRestartAppMutation,
  useSetPluginEnabledMutation,
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
  const installMutation = useInstallPluginMutation();
  const releaseMutation = usePluginReleaseMutation();
  const removeMutation = useRemovePluginMutation();
  const restartMutation = useRestartAppMutation();
  const enabledMutation = useSetPluginEnabledMutation();
  const [installOpen, setInstallOpen] = useState(false);
  const [restartPrompt, setRestartPrompt] = useState(false);
  const [uploadingFilename, setUploadingFilename] = useState<string | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<PluginEntry | null>(null);
  const entries = pluginsQuery.data ?? [];
  const programs = programsQuery.data ?? [];
  // Every row, host included, opens its plugin's own contributed page.
  const builtInRows: PluginTableRow[] = programRows(programs, entries, settings.frame).map(({ key, program, settingsSection }) => ({
    key,
    name: program.owner?.kind === "host" ? `${brand.label} Built-in` : program.label,
    source: programSource(program, brand.label),
    enabled: { checked: true, disabled: true, onChange: () => {} },
    onOpenSettings: settingsSection
      ? () => void navigate({ to: "/settings/plugins/$pluginId", params: { pluginId: settingsSection } })
      : undefined,
  }));
  const restartPending = entries.some((entry) => entry.restart_required);

  /* A plugin's settings page is its contributed section. The frame keys those
     by installed plugin, and an artifact names the installation it came
     from, so the bundle reaches its own page through its programs. */
  const settingsSectionOf = useCallback(
    (entry: PluginEntry): string | null => bundleSettingsPage(entry, programs, settings.frame),
    [programs, settings.frame],
  );

  async function handleInstall(source: PluginSource, filename: string | null): Promise<void> {
    installMutation.reset();
    setUploadingFilename(filename);
    try {
      const entry = await installMutation.mutateAsync(source);
      if (entry) {
        setInstallOpen(false);
        setRestartPrompt(true);
      }
    } catch (error) {
      toast.error("Plugin Installation Failed", {
        description: errorMessage(error, "The plugin could not be installed.") ?? undefined,
      });
    } finally {
      setUploadingFilename(null);
    }
  }

  async function handleRemove(): Promise<void> {
    if (!removeCandidate) {
      return;
    }
    try {
      await removeMutation.mutateAsync(removeCandidate.install_id);
      setRemoveCandidate(null);
      setRestartPrompt(true);
    } catch {
      // Keep explicit confirmation open with the error visible.
    }
  }

  return (
    <>
      <SettingsCard title="Plugins">
        <SettingsRow
          label="Organization Release Source"
          description="Discover compatible plugins from the source selected under Updates."
          error={releaseMutation.isError ? errorMessage(releaseMutation.error, "The release feed could not be checked.") : null}
        />
        {releaseMutation.data?.map((release) => (
          <SettingsRow
            key={release.candidate}
            label={`${release.package} ${release.version}`}
            description={`${release.target} · Dokime ${release.requiresDokime}`}
          >
            <Button
              size="sm"
              disabled={installMutation.isPending}
              onClick={() => {
                void handleInstall({ release: release.candidate }, release.package);
              }}
            >
              Install Release
            </Button>
          </SettingsRow>
        ))}
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
        <div className="px-4 py-3">
          {pluginsQuery.isPending || programsQuery.isPending ? (
            <div className="grid min-h-44 place-items-center text-sm text-muted-foreground">
              <span className="inline-flex items-center gap-2">
                <Loader2 className="size-4 animate-spin" />
                Loading Plugins…
              </span>
            </div>
          ) : pluginsQuery.isError || programsQuery.isError ? (
            <div className="grid min-h-44 place-items-center gap-3 p-6 text-center">
              <div>
                <p className="font-medium">Plugins could not be loaded</p>
                <NotificationMessage>
                  {errorMessage(pluginsQuery.error ?? programsQuery.error, "Try loading the plugin registry again.")}
                </NotificationMessage>
              </div>
              <Button
                onClick={() => {
                  void pluginsQuery.refetch();
                  void programsQuery.refetch();
                }}
                size="sm"
                variant="outline"
              >
                Try Again
              </Button>
            </div>
          ) : null}

          <PluginList
            actions={
              <>
                <Button disabled={!desktop || installMutation.isPending} onClick={() => setInstallOpen(true)}>
                  Install Plugin
                </Button>
                <Button
                  variant="outline"
                  disabled={!desktop || releaseMutation.isPending || installMutation.isPending}
                  onClick={() => releaseMutation.mutate()}
                >
                  Check For Updates
                </Button>
              </>
            }
            entries={entries}
            builtInRows={builtInRows}
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
        </div>
      </SettingsCard>
      <Dialog open={installOpen} onOpenChange={setInstallOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Install Plugin</DialogTitle>
            <DialogDescription>Choose a .dokime bundle to verify and install.</DialogDescription>
          </DialogHeader>
          <PluginDropZone
            disabled={installMutation.isPending}
            error={errorMessage(installMutation.error, "The plugin could not be installed.")}
            filename={uploadingFilename}
            onDrop={(filename) => void handleInstall("drop", filename)}
            onPick={() => void handleInstall("pick", null)}
            progress={installMutation.progress}
          />
        </DialogContent>
      </Dialog>
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
            <NotificationMessage>{errorMessage(restartMutation.error, `${brand.label} could not be restarted.`)}</NotificationMessage>
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
              {`The plugin will stop loading and its managed file will be deleted at the next ${brand.label} restart.`}
            </DialogDescription>
          </DialogHeader>
          {removeMutation.isError && removeCandidate ? (
            <NotificationMessage>{errorMessage(removeMutation.error, "The plugin could not be removed.")}</NotificationMessage>
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
  if (typeof error === "string" && error.trim()) return error;
  return error instanceof Error && error.message.trim() ? error.message : fallback;
}
