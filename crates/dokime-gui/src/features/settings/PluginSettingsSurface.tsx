/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One plugin's settings, on its own page.
 *
 * A plugin does not earn a place in the shell's navigation; it earns a page
 * under Plugins. The page bar carries the breadcrumb and its own Back chevron,
 * so this page publishes no action of its own, and the
 * body is the artifact's own editable settings and secrets followed by the
 * status cards it contributes. The Plugins list therefore stays one row per
 * plugin, with nothing nested inside it.
 */

/* === External Imports === */

import { useMemo, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { InstalledPluginDetail } from "@/features/settings/installed-plugins/InstalledPluginDetail";
import {
  useInstalledPluginDetailQuery,
  useInstalledPluginSecretMutation,
  useInstalledPluginSettingsMutation,
} from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginSettingsCards } from "@/features/settings/PluginSettingsCards";
import { usePluginsQuery, useRestartAppMutation } from "@/features/settings/plugins/plugin-queries";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsGrid } from "@/features/settings/SettingsGrid";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { SettingsAdvancedSwitch } from "@/features/settings/SettingsSurface";
import { useSettingsSections } from "@/features/settings/use-settings-sections";
import { usePageChrome } from "@/shell/page-chrome";

/* === Types === */

type PluginSettingsSurfaceProps = {
  /** The contributed section's id, which is how the frame keys its cards. */
  pluginId: string;
};

/* === Components === */

export function PluginSettingsSurface({ pluginId: sectionId }: PluginSettingsSurfaceProps) {
  const settings = useSettingsSections();
  const pluginsQuery = usePluginsQuery();
  const section = settings.frame?.sections.find((entry) => entry.section_id === sectionId) ?? null;
  const pluginId = section?.owner.kind === "installed_plugin" ? section.owner.plugin_id : null;
  const installation = !section
    ? pluginsQuery.data?.find((entry) => entry.install_id === sectionId && entry.state !== "removal_pending")
    : null;

  const actions = useMemo(() => <SettingsAdvancedSwitch />, []);
  usePageChrome({
    title: `Settings › Plugins › ${section?.label ?? installation?.metadata?.identity.name ?? sectionId}`,
    overflow: actions,
  });

  const cards = settings.frame?.cards.filter((card) => card.section_id === sectionId) ?? [];

  return (
    <div className="p-[var(--shell-inset)]">
      <SettingsGrid>
        {(settings.loading && !settings.frame) || (!section && pluginsQuery.isPending) ? (
          <SettingsCard title="Settings">
            <SettingsRow label="Status">
              <p role="status" className="text-sm text-muted-foreground">
                Reading the settings frame.
              </p>
            </SettingsRow>
          </SettingsCard>
        ) : settings.error ? (
          <SettingsCard title="Settings">
            <SettingsRow label="Status">
              <NotificationMessage>{settings.error}</NotificationMessage>
              <Button onClick={settings.retry} size="sm" variant="outline">
                Try Again
              </Button>
            </SettingsRow>
          </SettingsCard>
        ) : (
          <>
            {pluginId ? <PluginDetailCard pluginId={pluginId} /> : null}
            {installation?.boot_failed && installation.last_boot?.state === "failed" ? (
              <SettingsCard title="Startup Failed">
                <SettingsRow label={installation.last_boot.failure.plugin_name}>
                  <p role="alert" className="text-sm text-destructive">
                    {installation.last_boot.failure.message}
                    {installation.last_boot.failure.code ? <code className="block">{installation.last_boot.failure.code}</code> : null}
                  </p>
                  {installation.last_boot.failure.missing_secret ? (
                    <a className="text-sm underline" href={`#plugin-settings-${installation.last_boot.failure.plugin_id}`}>
                      Configure {installation.last_boot.failure.missing_secret}
                    </a>
                  ) : null}
                </SettingsRow>
              </SettingsCard>
            ) : null}
            {installation?.metadata?.owners.map((owner) => (
              <PluginDetailCard key={owner} pluginId={owner} />
            ))}
            {installation && !installation.active ? <PluginRestart /> : null}
            {!section && pluginsQuery.isError ? <NotificationMessage>{pluginsQuery.error.message}</NotificationMessage> : null}
            {cards.length ? (
              <PluginSettingsCards cards={cards} />
            ) : pluginId || installation ? null : (
              <SettingsCard title="Settings">
                <SettingsRow label="Status">
                  <span className="text-sm text-muted-foreground">This plugin contributes no settings.</span>
                </SettingsRow>
              </SettingsCard>
            )}
          </>
        )}
      </SettingsGrid>
    </div>
  );
}

/* === Private Components === */

function PluginRestart() {
  const brand = useBrand();
  const restart = useRestartAppMutation();
  const [open, setOpen] = useState(false);
  return (
    <SettingsCard title="Restart Required" description="Saved settings and secrets apply on the next start.">
      <SettingsRow label="Retry Startup">
        {isTauriDesktop() ? (
          <Button onClick={() => setOpen(true)} variant="outline">
            Restart {brand.label}
          </Button>
        ) : (
          <p>Restart dokime server to retry.</p>
        )}
      </SettingsRow>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Restart {brand.label}?</DialogTitle>
            <DialogDescription>Active work will stop and plugin resources will close before the application restarts.</DialogDescription>
          </DialogHeader>
          {restart.isError ? <NotificationMessage>{restart.error.message}</NotificationMessage> : null}
          <DialogFooter>
            <Button onClick={() => setOpen(false)} variant="outline">
              Later
            </Button>
            <Button disabled={restart.isPending} onClick={() => restart.mutate()}>
              Restart Now
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </SettingsCard>
  );
}

/** The artifact's own editable settings and secret bindings. */
function PluginDetailCard({ pluginId }: { pluginId: string }) {
  const detailQuery = useInstalledPluginDetailQuery(pluginId, pluginId);
  const settingsMutation = useInstalledPluginSettingsMutation();
  const secretMutation = useInstalledPluginSecretMutation();

  async function saveSecret(operation: () => Promise<unknown>) {
    try {
      await operation();
      toast.success("Secret Stored");
    } catch (error) {
      toast.error("Secret Not Stored", { description: error instanceof Error ? error.message : "Settings could not be saved." });
    } finally {
      secretMutation.reset();
    }
  }

  if (detailQuery.isPending) {
    return (
      <SettingsCard title="Settings">
        <SettingsRow label="Status">
          <p role="status" className="text-sm text-muted-foreground">
            Loading settings…
          </p>
        </SettingsRow>
      </SettingsCard>
    );
  }
  if (detailQuery.isError) {
    return (
      <SettingsCard title="Settings">
        <SettingsRow label="Status">
          <NotificationMessage>{detailQuery.error.message}</NotificationMessage>
        </SettingsRow>
      </SettingsCard>
    );
  }
  /* The frame and the artifact store answer separately, so a plugin replaced
     between the two reads would otherwise offer the previous installation's
     fields under the new one's name. */
  if (!detailQuery.data || detailQuery.data.plugin_id !== pluginId) {
    return (
      <SettingsCard title="Settings">
        <SettingsRow label="Status">
          <span className="text-sm text-muted-foreground">This program changed. Reload Plugins to view its current settings.</span>
        </SettingsRow>
      </SettingsCard>
    );
  }

  return (
    <InstalledPluginDetail
      detail={detailQuery.data}
      onSaveSecret={(id, secretId, secret) => void saveSecret(() => secretMutation.mutateAsync({ pluginId: id, secretId, secret }))}
      /* Settings write themselves and each row reports its own outcome; a
         secret, which is an explicit write, reports through a toast. */
      onSaveSettings={async (id, values) => {
        await settingsMutation.mutateAsync({ pluginId: id, values });
      }}
      savingSecret={secretMutation.isPending}
    />
  );
}
