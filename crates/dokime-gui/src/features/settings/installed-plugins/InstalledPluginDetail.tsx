/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin detail. */

/* === External Imports === */

/* === Local Imports === */

import { InstalledPluginRuntimeBadge, InstalledPluginStatusBadge } from "@/features/settings/installed-plugins/InstalledPluginBadges";
import { InstalledPluginSettingsForm } from "@/features/settings/installed-plugins/InstalledPluginSettingsForm";
import type {
  InstalledPluginDetail as InstalledPluginDetailModel,
  SettingValues,
} from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useBrand } from "@/lib/brand";

/* === Types === */

type InstalledPluginDetailProps = {
  detail: InstalledPluginDetailModel;
  /** Resolves when the write landed; each settings row states its own outcome. */
  onSaveSettings: (pluginId: string, values: SettingValues) => Promise<void>;
  onSaveSecret: (pluginId: string, secretId: string, secret: string) => void;
  savingSecret: boolean;
};

/* === Components === */

export function InstalledPluginDetail({ detail, onSaveSettings, onSaveSecret, savingSecret }: InstalledPluginDetailProps) {
  const brand = useBrand();
  const registration =
    detail.owner?.kind === "host"
      ? `${brand.label} host`
      : detail.owner?.kind === "bundle"
        ? detail.owner.package
        : detail.owner?.kind === "embedded"
          ? "Embedded Provider"
          : "Unclassified Catalog";
  return (
    <>
      <SettingsCard title={`${detail.label} v${detail.version}`} description={detail.plugin_id}>
        <SettingsRow label="Status" error={detail.error}>
          <InstalledPluginRuntimeBadge runtime={detail.runtime} />
          <InstalledPluginStatusBadge status={detail.status} />
        </SettingsRow>
        <SettingsRow label="Registration">
          <span className="min-w-0 text-sm">{registration}</span>
        </SettingsRow>
      </SettingsCard>
      <SettingsCard title="Settings" description="Settings specific to this program.">
        <InstalledPluginSettingsForm
          onSubmit={(values) => onSaveSettings(detail.plugin_id, values)}
          schema={detail.settings}
          values={detail.values}
          discovered={detail.discovered ?? {}}
          onSaveSecret={(id, secret) => onSaveSecret(detail.plugin_id, id, secret)}
          savingSecret={savingSecret}
          bindings={detail.secret_bindings}
        />
      </SettingsCard>
    </>
  );
}
