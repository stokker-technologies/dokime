<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import { Dialog, Tooltip } from "bits-ui";
  import type { ConnectedSession } from "../../../app/native-session/session";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import { buttonVariants } from "../../../components/ui/button-variants";
  import DialogContent from "../../../components/ui/DialogContent.svelte";
  import {
    admittedPluginRecords,
    type InstallPluginProgress,
    type ManagedPlugin,
    type PluginSource,
  } from "../../../lib/api/native-plugins";
  import { failureOf } from "../../../lib/notification-details";
  import { toast } from "../../../lib/notifications";
  import { isTauriDesktop } from "../../../lib/surface-environment";
  import PluginDropZone from "../plugins/PluginDropZone.svelte";
  import PluginTable, { type PluginRow } from "../plugins/PluginTable.svelte";
  import { bundleSettingsPage, programRows, programSource } from "../plugins/plugin-programs";
  import {
    enablePluginOptions,
    installPluginOptions,
    pluginReleaseOptions,
    pluginsListQueryOptions,
    reloadPluginOptions,
    removePluginOptions,
  } from "../plugins/plugin-queries";
  import { settingsFrameQueryOptions } from "../settings-queries";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";

  let { session, label, openSettings }: { session: ConnectedSession; label: string; openSettings: (id: string) => void } = $props();
  const desktop = isTauriDesktop();
  const releaseReasonId = $props.id();
  const client = $derived(session.queryClient);
  const settings = createQuery(
    () => settingsFrameQueryOptions(session.identity),
    () => client,
  );
  const plugins = createQuery(
    () => pluginsListQueryOptions(session),
    () => client,
  );
  let installOpen = $state(false);
  let filename = $state<string | null>(null);
  let progress = $state.raw<InstallPluginProgress | null>(null);
  let candidate = $state.raw<ManagedPlugin | null>(null);
  const install = createMutation(
    () => installPluginOptions(session, (next) => (progress = next)),
    () => client,
  );
  const release = createMutation(
    () => pluginReleaseOptions(session),
    () => client,
  );
  const reload = createMutation(
    () => reloadPluginOptions(session),
    () => client,
  );
  const remove = createMutation(
    () => removePluginOptions(session),
    () => client,
  );
  const setEnabled = createMutation(
    () => enablePluginOptions(session),
    () => client,
  );

  const entries = $derived(plugins.data ?? []);
  const installed = $derived(admittedPluginRecords(entries));
  const frame = $derived(settings.data ?? null);
  const unavailable = $derived(
    session.snapshot.connection.backend?.target
      ? null
      : "The host does not publish its build target. Install a verified bundle file until release selection is available.",
  );
  const builtInRows = $derived(
    programRows(installed, entries, frame).map(
      ({ key, program, settingsSection }): PluginRow => ({
        key,
        name: program.owner?.kind === "host" ? `${label} Built-in` : program.label,
        source: programSource(program, label),
        reload: program.owner?.kind !== "host" ? { disabled: reload.isPending, select: () => reload.mutate(program.plugin_id) } : undefined,
        enabled: { checked: true, disabled: true, onChange: () => {} },
        onOpenSettings: settingsSection ? () => openSettings(settingsSection) : undefined,
      }),
    ),
  );

  async function handleInstall(source: PluginSource, name: string | null) {
    install.reset();
    filename = name;
    try {
      if (await install.mutateAsync(source)) {
        installOpen = false;
      }
    } catch (error) {
      toast.error(failureOf(error, "Plugin Installation Failed"));
    } finally {
      filename = null;
    }
  }

  async function handleRemove() {
    if (candidate) {
      try {
        await remove.mutateAsync(candidate.name);
        candidate = null;
      } catch {
        // Keep The Confirmation Open With The Error Visible.
      }
    }
  }

  function toggle(entry: ManagedPlugin, enabled: boolean) {
    setEnabled.mutate({ installId: entry.name, enabled }, { onError: (error) => toast.error(failureOf(error, "Plugin Change Failed")) });
  }
</script>

<SettingsCard title="Plugins">
  <SettingsRow
    label="Organization Release Source"
    description="Discover compatible plugins from the source selected under Updates."
    failure={release.error}
    error={release.error ? failureOf(release.error, "Plugin Operation Failed").message : undefined}
  />
  {#each release.data ?? [] as result (result.candidate)}
    <SettingsRow label={`${result.package} ${result.version}`} description={`${result.target} · ${label} ${result.requiresDokime}`}>
      <Button size="sm" disabled={install.isPending} onclick={() => void handleInstall({ release: result.candidate }, result.package)}
        >Install Release</Button
      >
    </SettingsRow>
  {/each}
  <div class="px-4 py-3">
    <PluginTable
      {entries}
      programs={builtInRows}
      generation={session.identity.generation}
      loading={plugins.isPending}
      reloading={reload.isPending}
      removingId={remove.isPending ? (remove.variables ?? null) : null}
      changingEnabled={setEnabled.isPending}
      onReload={(id) => reload.mutate(id)}
      onRemove={(entry) => (candidate = entry)}
      onSetEnabled={toggle}
      {openSettings}
      settingsSectionOf={(entry) => bundleSettingsPage(entry, installed, frame)}
      failure={plugins.error ? failure : undefined}
    >
      {#snippet actions()}
        <Button disabled={!desktop || install.isPending} onclick={() => (installOpen = true)}>Install Plugin</Button>
        {#if unavailable}
          <span id={releaseReasonId} class="sr-only">{unavailable}</span>
          <Tooltip.Provider>
            <Tooltip.Root>
              <Tooltip.Trigger>
                {#snippet child({
                  props,
                })}
                  <button
                    {...props}
                    type="button"
                    aria-disabled="true"
                    aria-describedby={releaseReasonId}
                    class={buttonVariants({ variant: "outline", class: "opacity-50" })}
                  >
                    Check for Updates
                  </button>
                {/snippet}
              </Tooltip.Trigger>
              <Tooltip.Portal
                ><Tooltip.Content class="z-50 max-w-sm rounded-lg bg-popover px-3 py-2 text-xs text-popover-foreground ring-1 ring-border"
                  >{unavailable}</Tooltip.Content
                ></Tooltip.Portal
              >
            </Tooltip.Root>
          </Tooltip.Provider>
        {:else}
          <Button variant="outline" disabled={!desktop || release.isPending || install.isPending} onclick={() => release.mutate()}
            >Check for Updates</Button
          >
        {/if}
      {/snippet}
    </PluginTable>
  </div>
</SettingsCard>

{#snippet failure()}
  <ErrorCard
    failure={failureOf(plugins.error, "Plugins Unavailable")}
    onRetry={() => {
      void plugins.refetch();
    }}
  />
{/snippet}

<Dialog.Root bind:open={installOpen}>
  <DialogContent>
    <header class="flex flex-col gap-2">
      <Dialog.Title class="font-medium">Install Plugin</Dialog.Title>
      <Dialog.Description class="text-muted-foreground">Choose a .dokime bundle to verify and install.</Dialog.Description>
    </header>
    <PluginDropZone
      {label}
      failure={install.error}
      disabled={install.isPending}
      {filename}
      {progress}
      onDrop={(name) => void handleInstall("drop", name)}
      onPick={() => void handleInstall("pick", null)}
    />
  </DialogContent>
</Dialog.Root>
<Dialog.Root
  open={Boolean(candidate)}
  onOpenChange={(open) => {
    if (!open && !remove.isPending) {
      candidate = null;
    }
  }}
>
  <DialogContent showCloseButton={!remove.isPending}>
    <header class="flex flex-col gap-2">
      <Dialog.Title class="font-medium">Remove Plugin?</Dialog.Title>
      <Dialog.Description class="text-muted-foreground"
        >The plugin stops once execution cleanup and plugin streams finish, then its managed file is deleted.</Dialog.Description
      >
    </header>
    {#if remove.error && candidate}
      <ErrorCard notificationContext="action" failure={failureOf(remove.error, "Operation Failed")} />
    {/if}
    <footer class="flex justify-end gap-2">
      <Button disabled={remove.isPending} onclick={() => (candidate = null)} variant="outline">Cancel</Button>
      <Button disabled={remove.isPending} onclick={handleRemove} variant="destructive"
        >{remove.isPending ? "Removing…" : "Remove Plugin"}</Button
      >
    </footer>
  </DialogContent>
</Dialog.Root>
