<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Ellipsis } from "lucide-svelte";
  import type { QueryClient } from "@tanstack/svelte-query";
  import type { NativeSession } from "../../app/native-session/session";
  import Button from "../../components/ui/Button.svelte";
  import { buttonVariants } from "../../components/ui/button-variants";
  import PanelHeader from "../../shell/PanelHeader.svelte";
  import { storeState } from "../../stores/store-state";
  import { useUiStore } from "../../stores/ui-store";
  import ApplicationPreferences from "./ApplicationPreferences.svelte";
  import BackendsTable from "./BackendsTable.svelte";
  import SettingsAdvancedSwitch from "./SettingsAdvancedSwitch.svelte";
  import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS } from "./settings-sections";

  let { owner, close, queryClient }: { owner: NativeSession; close: () => void; queryClient: QueryClient } = $props();
  let section = $state(DEFAULT_SETTINGS_SECTION);
  const advanced = storeState(useUiStore, (state) => state.showAdvancedSettings);
  const sections = SETTINGS_SECTIONS.filter((item) => item.id === "backends" || item.id === "appearance");
</script>

<main class="flex h-dvh flex-col bg-background p-2">
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border bg-card">
    <PanelHeader title="Settings" onBack={close}>
      {#snippet actions()}
        <DropdownMenu.Root>
          <DropdownMenu.Trigger aria-label="Page Actions" class={buttonVariants({ size: "icon", variant: "ghost" })}>
            <Ellipsis />
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content align="end" class="z-50 rounded-lg bg-popover px-2 py-1 text-popover-foreground ring-1 ring-border">
              <SettingsAdvancedSwitch />
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      {/snippet}
    </PanelHeader>
    <div class="grid min-h-0 content-start gap-4 overflow-y-auto p-[var(--shell-inset)]">
      <nav aria-label="Settings Sections" class="flex gap-2">
        {#each sections as item (item.id)}
          <Button variant="outline" aria-pressed={section === item.id} onclick={() => (section = item.id)}>{item.label}</Button>
        {/each}
      </nav>
      <div class="grid w-full min-w-0 grid-cols-12 content-start gap-4 [&>*]:col-span-12 [&>.contents>*]:col-span-12">
        {#if section === "backends"}
          <BackendsTable {owner} />
        {:else}
          <ApplicationPreferences {queryClient} advanced={advanced()} localOnly />
        {/if}
      </div>
    </div>
  </div>
</main>
