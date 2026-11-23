<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession, NativeSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import type { NavigationMode } from "../../shell/navigation-model";
  import PageChrome from "../../shell/PageChrome.svelte";
  import { storeState } from "../../stores/store-state";
  import { uiStore } from "../../stores/ui-store";
  import ApplicationPreferences from "./ApplicationPreferences.svelte";
  import BackendsTable from "./BackendsTable.svelte";
  import AboutSection from "./sections/AboutSection.svelte";
  import DiagnosticsSettings from "./sections/DiagnosticsSettings.svelte";
  import EngineSection from "./sections/EngineSection.svelte";
  import PluginsSection from "./sections/PluginsSection.svelte";
  import RemoteSection from "./sections/RemoteSection.svelte";
  import SecretsSection from "./sections/SecretsSection.svelte";
  import SettingsAdvancedSwitch from "./SettingsAdvancedSwitch.svelte";
  import SettingsCard from "./SettingsCard.svelte";
  import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS, type SettingsSectionId } from "./settings-sections";

  let {
    session,
    owner,
    mode,
    selected,
    label,
    hash,
    clearHash,
    choose,
    openPluginSettings,
  }: {
    session: ConnectedSession;
    owner: NativeSession;
    mode: NavigationMode;
    selected?: SettingsSectionId;
    label: string;
    hash: string;
    clearHash: () => void;
    choose: (section: SettingsSectionId) => void;
    openPluginSettings: (id: string) => void;
  } = $props();
  const section = $derived(selected ?? DEFAULT_SETTINGS_SECTION);
  const advanced = storeState(uiStore, (state) => state.showAdvancedSettings);
  const showNavigation = $derived(mode !== "phone" || !selected);
  const showSection = $derived(mode !== "phone" || Boolean(selected));
</script>

<PageChrome title={showSection ? `Settings › ${SETTINGS_SECTIONS.find((item) => item.id === section)?.label ?? section}` : "Settings"} {overflow} />

{#snippet overflow()}
  <SettingsAdvancedSwitch />
{/snippet}

<div class="flex min-h-full min-w-0 flex-col tablet:flex-row">
  {#if showNavigation}
    <nav
      aria-label="Settings Sections"
      class="grid shrink-0 content-start gap-2 border-b p-4 tablet:w-48 tablet:border-r tablet:border-b-0"
    >
      {#each SETTINGS_SECTIONS as item (item.id)}
        <Button variant={section === item.id && showSection ? "secondary" : "ghost"} class="justify-start" onclick={() => choose(item.id)}>
          {item.label}
        </Button>
      {/each}
    </nav>
  {/if}
  {#if showSection}
    <div class="min-w-0 flex-1 p-4">
      <div class="grid w-full min-w-0 grid-cols-12 content-start gap-4 [&>*]:col-span-12">
        {#if section === "backends"}
          <BackendsTable {owner} />
        {:else if section === "engine"}
          <EngineSection queryClient={session.queryClient} />
        {:else if section === "diagnostics"}
          <DiagnosticsSettings {session} />
        {:else if section === "about"}
          <AboutSection brandLabel={label} backend={session.snapshot.connection.backend} />
        {:else if section === "appearance"}
          <ApplicationPreferences queryClient={session.queryClient} {session} advanced={advanced()} />
        {:else if section === "remote"}
          <RemoteSection queryClient={session.queryClient} {session} advanced={advanced()} {label} />
        {:else if section === "secrets"}
          {#key session.identity.generation}
            <SecretsSection {session} {hash} {clearHash} />
          {/key}
        {:else if section === "plugins"}
          <PluginsSection {session} {label} openSettings={openPluginSettings} />
        {:else}
          <SettingsCard title="Settings">
            <ErrorCard
              failure={failureOf("Choose a section from Settings Sections.", "Settings Unavailable", "missing-settings-section")}
            />
          </SettingsCard>
        {/if}
      </div>
    </div>
  {/if}
</div>
