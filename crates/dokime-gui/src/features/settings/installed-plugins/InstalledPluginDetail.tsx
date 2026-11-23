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
import { SvelteMount } from "@/components/SvelteMount";
import InstalledPluginSettingsForm from "@/features/settings/installed-plugins/InstalledPluginSettingsForm.svelte";
import type * as contracts from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useBrand } from "@/lib/brand-context";

/* === Types === */

type InstalledPluginDetailProps = {
  detail: contracts.InstalledPluginDetail;
  onSaveSettings: (pluginId: string, values: contracts.SettingValues) => Promise<void>;
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
          <SvelteMount
            component={InstalledPluginSettingsForm}
            props={{ onSubmit: (values) => onSaveSettings(detail.plugin_id, values), schema: detail.settings, values: detail.values }}
          />
        </div>
      </SettingsCard>
      {detail.settings.contributions?.some((entry) => entry.kind === "secret") ? (
        <SettingsCard
          title="Required Secrets"
          description="The plugin reads each identifier from Secret Storage; add any that are missing."
        >
          {detail.settings.contributions
            .filter((entry) => entry.kind === "secret")
            .map((secret) => {
              const available = detail.secrets.find((entry) => entry.identifier === secret.id)?.available ?? false;
              return (
                <SettingsRow key={secret.id} label={secret.title} description={secret.description}>
                  <code className="text-xs">{secret.id}</code>
                  <span>
                    {secret.required ? "Required" : "Optional"} · {available ? "Available" : "Missing"}
                  </span>
                  {!available ? (
                    <Button variant="outline" onClick={() => setAdding(secret.id)}>
                      Add in Secret Storage
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
