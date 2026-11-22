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
 * status cards it contributes. The Plugins list keeps entry metadata in one row
 * and shows a startup failure immediately below its entry.
 */

/* === External Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { pluginStartFailure } from "@/features/settings/plugins/plugin-queries";
import { ErrorCard } from "@dokime/plots/components/ErrorCard";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { useMemo } from "react";

/* === Local Imports === */

import { Button } from "@dokime/plots/components/ui/button";

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
import { usePageChrome } from "@dokime/document/shell/page-chrome";

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
  const section = settings.frame?.sections.find((entry) => entry.sectionId === sectionId) ?? null;
  const pluginId = section?.owner?.kind.case === "installedPluginId" ? section.owner.kind.value : null;
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

  const waiting = Boolean(owning?.value?.enabled && !owning.failure && !owning.description);
  const label = owning?.description?.id || owning?.name || sectionId;
  const cards = settings.frame?.cards.filter((card) => card.sectionId === sectionId) ?? [];

  return (
    <div className="p-[var(--shell-inset)]">
      <SettingsGrid>
        {owning?.failure ? (
          <ErrorCard
            failure={pluginStartFailure(owning, generation)}
            onRetry={reloadable ? () => reload.mutate(owning.name) : undefined}
            retryFirst
            retryLabel="Restart Plugin"
            retryDisabled={reload.isPending}
          />
        ) : null}
        {(settings.loading && !settings.frame) || (!section && pluginsQuery.isPending) ? (
          <SettingsCard title="Settings">
            <SettingsRow label="Status">
              <p role="status" className="text-sm text-muted-foreground">
                Reading the settings frame.
              </p>
            </SettingsRow>
          </SettingsCard>
        ) : waiting ? (
          <SettingsCard title="Settings">
            <p role="status" className="px-4 py-3 text-sm text-muted-foreground">
              Waiting for Plugin {label}
            </p>
          </SettingsCard>
        ) : settings.error ? (
          <SettingsCard title="Settings">
            <ErrorCard failure={failureOf(settings.failure, "Settings Unavailable")} onRetry={settings.retry} />
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
                {reload.isError ? <ErrorCard notificationContext="action" failure={failureOf(reload.error, "Operation Failed")} /> : null}
              </SettingsRow>
            </SettingsCard>
            {pluginId ? <PluginDetailCard key={`${generation}:${pluginId}`} pluginId={pluginId} /> : null}
            {installation ? <PluginDetailCard key={`${generation}:${installation.name}`} pluginId={installation.name} /> : null}
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
    const failure = failureOf(detailQuery.error, "Settings Unavailable");
    return (
      <SettingsCard title="Settings">
        <ErrorCard failure={failure} onRetry={() => void detailQuery.refetch()} />
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
