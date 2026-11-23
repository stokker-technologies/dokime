<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "../../../app/native-session/session";
  import Badge from "../../../components/ui/Badge.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import AddSecretDialog from "../sections/AddSecretDialog.svelte";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import {
    type InstalledPluginDetail,
    type InstalledPluginRuntimeKind,
    RUNTIME_LABELS,
    type SettingValues,
  } from "./installed-plugin-contracts";
  import InstalledPluginSettingsForm from "./InstalledPluginSettingsForm.svelte";

  let {
    detail,
    session,
    brandLabel,
    onSaveSettings,
  }: {
    detail: InstalledPluginDetail;
    session: ConnectedSession;
    brandLabel: string;
    onSaveSettings: (pluginId: string, values: SettingValues) => Promise<void>;
  } = $props();
  const badge = "h-5 rounded px-2 text-xs font-medium";
  const runtimeClass: Record<InstalledPluginRuntimeKind, string> = { code: "border-info/30 bg-info/10 text-info" };
  const statuses: Record<string, { style: string; label: string }> = {
    registered: { style: "border-success/30 bg-success/10 text-success", label: "Registered" },
    disabled: { style: "text-muted-foreground", label: "Disabled" },
    failed: { style: "text-destructive", label: "Failed" },
    not_running: { style: "text-muted-foreground", label: "Not Running" },
    restarting: { style: "border-info/30 bg-info/10 text-info", label: "Restarting…" },
    restart_pending: { style: "text-muted-foreground", label: "Restart Pending" },
  };
  let adding = $state<string | null>(null);
  const registration = $derived(
    detail.owner?.kind === "host"
      ? `${brandLabel} Host`
      : detail.owner?.kind === "bundle"
        ? detail.owner.package
        : detail.owner?.kind === "embedded"
          ? "Embedded Provider"
          : "Unclassified Catalog",
  );
  const runtime = $derived(detail.runtime as InstalledPluginRuntimeKind);
  const status = $derived(statuses[detail.status] ?? { style: "text-muted-foreground", label: detail.status });
  const secrets = $derived(detail.settings.contributions?.filter((entry) => entry.kind === "secret") ?? []);
</script>

<SettingsCard title={`${detail.label} v${detail.version}`} description={detail.plugin_id}>
  <SettingsRow label="Status" error={detail.error ?? undefined}>
    <Badge class={`${badge} ${runtimeClass[runtime]}`} variant="secondary">{RUNTIME_LABELS[runtime]}</Badge>
    <Badge class={`${badge} ${status.style}`} variant="outline">{status.label}</Badge>
  </SettingsRow>
  <SettingsRow label="Registration"><span class="min-w-0 text-sm">{registration}</span></SettingsRow>
</SettingsCard>
<SettingsCard title="Settings" description="Settings specific to this program.">
  <div id={`plugin-settings-${detail.plugin_id}`} class="divide-y scroll-mt-4">
    <InstalledPluginSettingsForm
      onSubmit={(values) => onSaveSettings(detail.plugin_id, values)}
      schema={detail.settings}
      values={detail.values}
    />
  </div>
</SettingsCard>
{#if secrets.length}
  <SettingsCard title="Required Secrets" description="The plugin reads each identifier from Secret Storage. Add any that are missing.">
    {#each secrets as secret (secret.id)}
      {@const available = detail.secrets.find((entry) => entry.identifier === secret.id)?.available ?? false}
      <SettingsRow label={secret.title} description={secret.description ?? undefined}>
        <code class="text-xs">{secret.id}</code>
        <span>{secret.required ? "Required" : "Optional"} · {available ? "Available" : "Missing"}</span>
        {#if !available}
          <Button variant="outline" onclick={() => (adding = secret.id)}>Add in Secret Storage</Button>
        {/if}
      </SettingsRow>
    {/each}
  </SettingsCard>
{/if}
{#if adding !== null}
  <AddSecretDialog identifier={adding} {session} queryClient={session.queryClient} onClose={() => (adding = null)} />
{/if}
