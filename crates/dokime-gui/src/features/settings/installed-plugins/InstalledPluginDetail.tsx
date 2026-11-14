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

import { useEffect, useId, useState } from "react";
import { Input } from "@/components/ui/input";
import { useAutosave } from "@/features/settings/settings-autosave";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";

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
  onSaveBinding: (name: string, identifier: string, secret: string) => Promise<void>;
  onSaveSettings: (pluginId: string, values: SettingValues) => Promise<void>;
};

/* === Components === */

export function InstalledPluginDetail({ detail, onSaveSettings, onSaveBinding }: InstalledPluginDetailProps) {
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
          />
        </div>
      </SettingsCard>
      {detail.settings.contributions?.some((entry) => entry.kind === "secret") ? (
        <SettingsCard
          title="Required Secrets"
          description="Bind each declared name to an identifier in Secret Storage, then restart the plugin."
        >
          {detail.settings.contributions
            .filter((entry) => entry.kind === "secret")
            .map((secret) => {
              const binding = detail.secrets.find((entry) => entry.identifier === secret.id);
              const available = binding?.available ?? false;
              return (
                <SettingsRow key={secret.id} label={secret.title} description={secret.description}>
                  <SecretBinding value={binding?.secret ?? ""} onSave={(value) => onSaveBinding(detail.plugin_id, secret.id, value)} />
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

function SecretBinding({ value, onSave }: { value: string; onSave: (value: string) => Promise<void> }) {
  const id = useId();
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const autosave = useAutosave(onSave);
  return (
    <div className="flex items-center gap-2">
      <Input
        id={id}
        aria-label="Secret Storage Identifier"
        value={draft}
        onChange={(event) => {
          const next = event.target.value;
          setDraft(next);
          autosave.change(next);
        }}
      />
      <SettingsFieldStatus state={autosave.state} onRetry={autosave.retry} />
    </div>
  );
}
