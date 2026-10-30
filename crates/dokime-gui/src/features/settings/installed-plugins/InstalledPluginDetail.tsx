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

import { useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { AddSecretDialog } from "@/features/settings/sections/SecretsSection";

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
};

/* === Components === */

export function InstalledPluginDetail({ detail, onSaveSettings }: InstalledPluginDetailProps) {
  const brand = useBrand();
  const [adding, setAdding] = useState<string | null>(null);
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
        <div id={`plugin-settings-${detail.plugin_id}`} className="divide-y scroll-mt-4">
          <InstalledPluginSettingsForm
            onSubmit={(values) => onSaveSettings(detail.plugin_id, values)}
            schema={detail.settings}
            values={detail.values}
            discovered={detail.discovered ?? {}}
          />
        </div>
      </SettingsCard>
      {detail.settings.contributions?.some((entry) => entry.kind === "secret") ? (
        <SettingsCard
          title="Required Secrets"
          description="Identifiers are read from Secret Storage when this plugin starts. Add missing secrets, then restart Dokime or retry the plugin."
        >
          {detail.settings.contributions
            .filter((entry) => entry.kind === "secret")
            .map((secret) => {
              const available = detail.secrets.find((entry) => entry.identifier === secret.id)?.available ?? false;
              return (
                <SettingsRow key={secret.id} label={secret.title} description={secret.description}>
                  <code>{secret.id}</code>
                  <span>
                    {secret.required ? "Required" : "Optional"} · {available ? "Available" : "Missing"}
                  </span>
                  {!available ? (
                    <Button variant="outline" onClick={() => setAdding(secret.id)}>
                      Add In Secret Storage
                    </Button>
                  ) : null}
                </SettingsRow>
              );
            })}
        </SettingsCard>
      ) : null}
      {adding !== null ? <AddSecretDialog identifier={adding} onClose={() => setAdding(null)} /> : null}
    </>
  );
}
