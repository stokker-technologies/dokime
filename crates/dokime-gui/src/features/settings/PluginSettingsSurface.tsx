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
 * and the
 * body is the artifact's own editable settings and declared secrets followed by the
 * status cards it contributes. The Plugins list therefore stays one row per
 * plugin, with nothing nested inside it.
 */

/* === External Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { pluginStartFailure } from "@/features/settings/plugins/plugin-queries";
import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useMemo } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";

/* === Local Imports === */

import { Button } from "@/components/ui/button";

import { InstalledPluginDetail } from "@/features/settings/installed-plugins/InstalledPluginDetail";
import {
  useInstalledPluginDetailQuery,
  useInstalledPluginSettingsMutation,
} from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginSettingsCards } from "@/features/settings/PluginSettingsCards";
import { usePluginsQuery, useReloadPluginMutation } from "@/features/settings/plugins/plugin-queries";
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
  const generation = useNativeSessionOptional()?.identity.generation ?? "";
  const settings = useSettingsSections();
  const pluginsQuery = usePluginsQuery();
  const section = settings.frame?.sections.find((entry) => entry.section_id === sectionId) ?? null;
  const pluginId = section?.owner.kind === "installed_plugin" ? section.owner.plugin_id : null;
  const installation = !section ? pluginsQuery.data?.find((entry) => entry.name === sectionId) : null;
  /* The installation whose startup or restart this page reports: the bundle's own
     page, or the installation that runs the contributed section's program. */
  const owning = installation ?? (pluginId ? (pluginsQuery.data?.find((entry) => entry.name === pluginId) ?? null) : null);

  const reload = useReloadPluginMutation();
  const reloadable = owning ? owning.value?.enabled === true : Boolean(pluginId);
  const actions = useMemo(() => <SettingsAdvancedSwitch />, []);
  usePageChrome({
    title: `Settings › Plugins › ${section?.label ?? installation?.description?.id ?? sectionId}`,
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
              <ErrorCard failure={failureOf(settings.failure, "Data Unavailable")} />
              <Button onClick={settings.retry} size="sm" variant="outline">
                Try Again
              </Button>
            </SettingsRow>
          </SettingsCard>
        ) : (
          <>
            <SettingsCard title="Reload">
              <SettingsRow label="Plugin" description="Apply saved settings and restart this plugin and all its observers.">
                <Button
                  disabled={reload.isPending || !reloadable}
                  onClick={() => {
                    if (owning) reload.mutate(owning.name);
                    else if (pluginId) reload.mutate(pluginId);
                  }}
                  variant="outline"
                >
                  Reload
                </Button>
                {reload.isError ? <NotificationMessage error={reload.error}>{reload.error.message}</NotificationMessage> : null}
              </SettingsRow>
            </SettingsCard>
            {pluginId ? <PluginDetailCard pluginId={pluginId} /> : null}
            {owning?.failure ? (
              <SettingsCard title="Plugin Unavailable">
                <SettingsRow label={owning.description?.id || owning.name}>
                  <ErrorCard failure={pluginStartFailure(owning, generation)} />
                </SettingsRow>
              </SettingsCard>
            ) : null}
            {installation ? <PluginDetailCard pluginId={installation.name} /> : null}
            {!section && pluginsQuery.isError ? <ErrorCard failure={failureOf(pluginsQuery.error, "Data Unavailable")} /> : null}
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

/** The artifact's settings and read-only secret requirements. */
function PluginDetailCard({ pluginId }: { pluginId: string }) {
  const detailQuery = useInstalledPluginDetailQuery(pluginId, pluginId);
  const settingsMutation = useInstalledPluginSettingsMutation();
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
          <ErrorCard failure={failureOf(detailQuery.error, "Data Unavailable")} />
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
      onSaveSettings={async (id, values) => {
        await settingsMutation.mutateAsync({ pluginId: id, values });
      }}
    />
  );
}
