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

import { useMemo } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { InstalledPluginDetail } from "@/features/settings/installed-plugins/InstalledPluginDetail";
import {
  useInstalledPluginDetailQuery,
  useInstalledPluginSecretMutation,
  useInstalledPluginSettingsMutation,
} from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginSettingsCards } from "@/features/settings/PluginSettingsCards";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsGrid } from "@/features/settings/SettingsGrid";
import { SettingsAdvancedSwitch } from "@/features/settings/SettingsSurface";
import { SettingsRow } from "@/features/settings/SettingsRow";
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
  const section = settings.frame?.sections.find((entry) => entry.section_id === sectionId) ?? null;
  const pluginId = section?.owner.kind === "installed_plugin" ? section.owner.plugin_id : null;

  const actions = useMemo(() => <SettingsAdvancedSwitch />, []);
  usePageChrome({ title: `Settings › Plugins › ${section?.label ?? sectionId}`, overflow: actions });

  const cards = settings.frame?.cards.filter((card) => card.section_id === sectionId) ?? [];

  return (
    <div className="p-[var(--shell-inset)]">
      <SettingsGrid>
        {settings.loading && !settings.frame ? (
          <SettingsCard title="Settings">
            <SettingsRow label="Status">
              <span className="text-sm text-muted-foreground" role="status">
                Reading the settings frame.
              </span>
            </SettingsRow>
          </SettingsCard>
        ) : settings.error ? (
          <SettingsCard title="Settings">
            <SettingsRow label="Status">
              <span className="text-sm text-destructive" role="alert">
                {settings.error}
              </span>
              <Button onClick={settings.retry} size="sm" variant="outline">
                Try Again
              </Button>
            </SettingsRow>
          </SettingsCard>
        ) : (
          <>
            {pluginId ? <PluginDetailCard pluginId={pluginId} /> : null}
            {cards.length ? (
              <PluginSettingsCards cards={cards} />
            ) : pluginId ? null : (
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
          <span className="text-sm text-muted-foreground" role="status">
            Loading settings…
          </span>
        </SettingsRow>
      </SettingsCard>
    );
  }
  if (detailQuery.isError) {
    return (
      <SettingsCard title="Settings">
        <SettingsRow label="Status">
          <span className="text-sm text-destructive" role="alert">
            {detailQuery.error.message}
          </span>
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
