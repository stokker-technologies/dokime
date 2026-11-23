<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { publishPageChrome } from "../../shell/page-chrome.svelte";
  import { hubStore } from "../../stores/hub-store";
  import { storeState } from "../../stores/store-state";
  import InstalledPluginDetail from "./installed-plugins/InstalledPluginDetail.svelte";
  import {
    installedPluginDetailsOptions,
    installedPluginSettingsOptions,
    pluginQueryKeys,
  } from "./installed-plugins/installed-plugin-queries";
  import { pluginsListQueryOptions, pluginStartFailure, reloadPluginOptions } from "./plugins/plugin-queries";
  import PluginSettingsCards from "./PluginSettingsCards.svelte";
  import { settingsFrameQueryOptions } from "./settings-queries";
  import SettingsAdvancedSwitch from "./SettingsAdvancedSwitch.svelte";
  import SettingsCard from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";

  let { pluginId: sectionId, session, label }: { pluginId: string; session: ConnectedSession; label: string } = $props();
  const client = $derived(session.queryClient);
  const generation = $derived(session.identity.generation);
  const frame = createQuery(
    () => settingsFrameQueryOptions(session.identity),
    () => client,
  );
  const plugins = createQuery(
    () => pluginsListQueryOptions(session),
    () => client,
  );
  const reload = createMutation(
    () => reloadPluginOptions(session),
    () => client,
  );
  const save = createMutation(
    () => installedPluginSettingsOptions(session),
    () => client,
  );

  const section = $derived(frame.data?.sections.find((candidate) => candidate.sectionId === sectionId));
  const pluginId = $derived(section?.owner?.kind.case === "installedPluginId" ? section.owner.kind.value : null);
  const installation = $derived(!section ? plugins.data?.find((entry) => entry.name === sectionId) : undefined);
  const owning = $derived(installation ?? plugins.data?.find((entry) => entry.name === pluginId));
  const reloadable = $derived(owning ? owning.value?.enabled === true : Boolean(pluginId));
  const loading = $derived((frame.isPending && !frame.data) || (!section && plugins.isPending));
  const waiting = $derived(Boolean(owning?.value?.enabled && !owning.failure && !owning.description));
  const cards = $derived(frame.data?.cards.filter((card) => card.sectionId === sectionId) ?? []);
  const detailsId = $derived(!loading && !waiting && !frame.error ? (pluginId ?? installation?.name ?? "") : "");
  const details = createQuery(
    () => installedPluginDetailsOptions(session, detailsId, detailsId),
    () => client,
  );

  const operations = storeState(hubStore, (state) => state.operations.map((operation) => `${operation.id}:${operation.status}`).join());
  let observedOperations: string | undefined;
  $effect(() => {
    const current = operations();
    if (detailsId && observedOperations !== undefined && current !== observedOperations) {
      void client.invalidateQueries({ queryKey: pluginQueryKeys.detail(detailsId, generation) });
    }
    observedOperations = current;
  });

  publishPageChrome(() => ({ title: `Settings › Plugins › ${section?.label ?? installation?.description?.id ?? sectionId}`, overflow }));

  function reloadOwner() {
    const id = owning?.name ?? pluginId;
    if (id) {
      reload.mutate(id);
    }
  }
</script>

{#snippet overflow()}
  <SettingsAdvancedSwitch />
{/snippet}

<div class="p-[var(--shell-inset)]">
  <div class="grid w-full min-w-0 grid-cols-12 content-start gap-4 [&>*]:col-span-12">
    {#if owning?.failure}
      <ErrorCard
        failure={pluginStartFailure(owning, generation)}
        onRetry={reloadable ? reloadOwner : undefined}
        retryFirst
        retryLabel="Restart Plugin"
        retryDisabled={reload.isPending}
      />
    {/if}
    {#if loading}
      <SettingsCard title="Settings">
        <SettingsRow label="Status"><p role="status" class="text-sm text-muted-foreground">Reading the settings frame.</p></SettingsRow>
      </SettingsCard>
    {:else if waiting}
      <SettingsCard title="Settings">
        <p role="status" class="px-4 py-3 text-sm text-muted-foreground">
          Waiting for Plugin {owning?.description?.id || owning?.name || sectionId}
        </p>
      </SettingsCard>
    {:else if frame.error}
      <SettingsCard title="Settings">
        <ErrorCard failure={failureOf(frame.error, "Settings Unavailable")} onRetry={() => void frame.refetch()} />
      </SettingsCard>
    {:else}
      <SettingsCard title="Reload">
        <SettingsRow label="Plugin" description="Apply saved settings and restart this plugin and all its observers.">
          <Button disabled={reload.isPending || !reloadable} onclick={reloadOwner} variant="outline">Reload</Button>
          {#if reload.error}
            <ErrorCard notificationContext="action" failure={failureOf(reload.error, "Operation Failed")} />
          {/if}
        </SettingsRow>
      </SettingsCard>
      {#if detailsId}
        {#if details.isPending}
          <SettingsCard title="Settings">
            <SettingsRow label="Status"><p role="status" class="text-sm text-muted-foreground">Loading settings…</p></SettingsRow>
          </SettingsCard>
        {:else if details.error}
          <SettingsCard title="Settings">
            <ErrorCard failure={failureOf(details.error, "Settings Unavailable")} onRetry={() => void details.refetch()} />
          </SettingsCard>
        {:else if details.data?.plugin_id === detailsId}
          {#key `${generation}:${detailsId}`}
            <InstalledPluginDetail
              detail={details.data}
              {session}
              brandLabel={label}
              onSaveSettings={async (pluginId, values) => {
                await save.mutateAsync({ pluginId, values });
              }}
            />
          {/key}
        {:else}
          <SettingsCard title="Settings">
            <SettingsRow label="Status"
              ><span class="text-sm text-muted-foreground"
                >This program changed. Reload Plugins to view its current settings.</span
              ></SettingsRow
            >
          </SettingsCard>
        {/if}
      {/if}
      {#if !section && plugins.error}
        <ErrorCard failure={failureOf(plugins.error, "Data Unavailable")} />
      {/if}
      {#if cards.length}
        <PluginSettingsCards {cards} />
      {:else if !detailsId}
        <SettingsCard title="Settings">
          <SettingsRow label="Status"><span class="text-sm text-muted-foreground">This plugin contributes no settings.</span></SettingsRow>
        </SettingsCard>
      {/if}
    {/if}
  </div>
</div>
