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

import { Separator } from "@/components/ui/separator";
import {
  InstalledPluginRuntimeBadge,
  InstalledPluginStatusBadge,
} from "@/features/settings/installed-plugins/InstalledPluginBadges";
import { InstalledPluginSettingsForm } from "@/features/settings/installed-plugins/InstalledPluginSettingsForm";
import type {
  InstalledPluginDetail as InstalledPluginDetailModel,
  SettingValues,
} from "@/features/settings/installed-plugins/installed-plugin-contracts";

/* === Types === */

type InstalledPluginDetailProps = {
  detail: InstalledPluginDetailModel;
  /** Resolves when the write landed; each settings row states its own outcome. */
  onSaveSettings: (pluginId: string, values: SettingValues) => Promise<void>;
  onSaveCredential: (pluginId: string, credentialId: string, secret: string) => void;
  savingCredential: boolean;
};

/* === Components === */

export function InstalledPluginDetail({
  detail,
  onSaveSettings,
  onSaveCredential,
  savingCredential,
}: InstalledPluginDetailProps) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* One aligned row: what the program is, its version muted beside it,
          then the pills, each as wide as the word it carries. */}
      <header className="border-b p-4">
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3">
          <div className="flex min-w-0 items-baseline gap-2">
            <h3 className="truncate font-heading text-base font-semibold">{detail.label}</h3>
            <span className="shrink-0 text-xs text-muted-foreground">v{detail.version}</span>
          </div>
          <div className="flex shrink-0 flex-wrap items-center justify-end gap-1.5">
            <InstalledPluginRuntimeBadge runtime={detail.runtime} />
            <InstalledPluginStatusBadge status={detail.status} />
          </div>
        </div>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">{detail.plugin_id}</p>
        {detail.error ? (
          <p className="mt-3 rounded-md border border-destructive/30 bg-destructive/10 p-2 text-xs text-destructive">
            {detail.error}
          </p>
        ) : null}
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto p-4">
        <div className="grid gap-6">
          <section className="grid gap-2">
            <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Details
            </h4>
            <dl className="grid gap-1.5 text-sm">
              <DetailRow
                label="Registration"
                value={
                  detail.owner?.kind === "host"
                    ? "Dokime host"
                    : detail.owner?.kind === "bundle"
                      ? detail.owner.package
                      : detail.owner?.kind === "embedded"
                        ? "Embedded Provider"
                        : "Unclassified Catalog"
                }
              />
            </dl>
          </section>

          <Separator />

          <section className="grid gap-3">
            <div>
              <h4 className="text-sm font-semibold">Settings</h4>
              <p className="text-xs text-muted-foreground">Settings specific to this program.</p>
            </div>
            <InstalledPluginSettingsForm
              onSubmit={(values) => onSaveSettings(detail.plugin_id, values)}
              schema={detail.settings}
              values={detail.values}
              discovered={detail.discovered ?? {}}
              onSaveSecret={(id, secret) => onSaveCredential(detail.plugin_id, id, secret)}
              savingSecret={savingCredential}
              bindings={detail.credential_bindings}
            />
          </section>
        </div>
      </div>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-4">
      <dt className="shrink-0 text-muted-foreground">{label}</dt>
      <dd className="min-w-0 truncate text-right font-medium">{value}</dd>
    </div>
  );
}
