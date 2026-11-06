/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useNavigate } from "@tanstack/react-router";
import { Loader2, RotateCw } from "lucide-react";
import { useCallback, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
import { useInstalledPluginsQuery, usePluginRestartRefresh } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginDropZone } from "@/features/settings/plugins/PluginDropZone";
import { PluginList } from "@/features/settings/plugins/PluginList";
import { bundleSettingsPage, programRows, programSource } from "@/features/settings/plugins/PluginPrograms";
import type { PluginTableRow } from "@/features/settings/plugins/PluginTable";
import {
  useInstallPluginMutation,
  usePluginReleaseMutation,
  usePluginsQuery,
  useRemovePluginMutation,
  useReloadPluginMutation,
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

export function PluginsSection({ advanced = false }: { advanced?: boolean }) {
  const brand = useBrand();
  const desktop = isTauriDesktop();
  const navigate = useNavigate();
  const settings = useSettingsSections();
  const pluginsQuery = usePluginsQuery();
  const programsQuery = useInstalledPluginsQuery();
  const installMutation = useInstallPluginMutation();
  const releaseMutation = usePluginReleaseMutation();
  const reloadMutation = useReloadPluginMutation();
  const removeMutation = useRemovePluginMutation();
  const enabledMutation = useSetPluginEnabledMutation();
  const [installOpen, setInstallOpen] = useState(false);
  const [uploadingFilename, setUploadingFilename] = useState<string | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<PluginEntry | null>(null);
  const entries = pluginsQuery.data ?? [];
  const programs = programsQuery.data ?? [];
  // Every row, host included, opens its plugin's own contributed page.
  const builtInRows: PluginTableRow[] = programRows(programs, entries, settings.frame).map(({ key, program, settingsSection }) => ({
    key,
    name: program.owner?.kind === "host" ? `${brand.label} Built-in` : program.label,
    source: programSource(program, brand.label),
    trailing:
      program.owner?.kind !== "host" ? (
        <Button
          aria-label={`Reload ${program.label}`}
          title="Reload"
          size="icon-sm"
          variant="ghost"
          disabled={reloadMutation.isPending}
          onClick={() => reloadMutation.mutate({ pluginId: program.plugin_id })}
        >
          <RotateCw />
        </Button>
      ) : undefined,
    enabled: { checked: true, disabled: true, onChange: () => {} },
    onOpenSettings: settingsSection
      ? () => void navigate({ to: "/settings/plugins/$pluginId", params: { pluginId: settingsSection } })
      : undefined,
  }));
  // Changes apply in the running backend; re-read the list until each one has.
  usePluginRestartRefresh(entries.find((entry) => entry.restart !== "idle")?.restart);

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
      }
    } catch (error) {
      toast.error(failureOf(error, "Plugin Installation Failed"));
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
    } catch {
      // Keep explicit confirmation open with the error visible.
    }
  }

  return (
    <>
      <SettingsCard title="Plugins">
        <SettingsRow
          label="Organization Release Source"
          failure={releaseMutation.error}
          description="Discover compatible plugins from the source selected under Updates."
          error={
            releaseMutation.isError
              ? releaseMutation.error
                ? failureOf(releaseMutation.error, "Plugin Operation Failed").message
                : null
              : null
          }
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
              <ErrorCard failure={failureOf(pluginsQuery.error ?? programsQuery.error, "Plugins Unavailable")} />
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
                  Check for Updates
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
                  onError: (error) => toast.error(failureOf(error, "Plugin Change Failed")),
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
      <ApplicationPreferences section="plugins" advanced={advanced} />
      <Dialog open={installOpen} onOpenChange={setInstallOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Install Plugin</DialogTitle>
            <DialogDescription>Choose a .dokime bundle to verify and install.</DialogDescription>
          </DialogHeader>
          <PluginDropZone
            failure={installMutation.error}
            disabled={installMutation.isPending}
            error={installMutation.error ? failureOf(installMutation.error, "Plugin Operation Failed").message : null}
            filename={uploadingFilename}
            onDrop={(filename) => void handleInstall("drop", filename)}
            onPick={() => void handleInstall("pick", null)}
            progress={installMutation.progress}
          />
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
              The plugin stops once execution cleanup and plugin streams finish, then its managed file is deleted.
            </DialogDescription>
          </DialogHeader>
          {removeMutation.isError && removeCandidate ? (
            <NotificationMessage error={removeMutation.error}>
              {removeMutation.error ? failureOf(removeMutation.error, "Plugin Operation Failed").message : null}
            </NotificationMessage>
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
