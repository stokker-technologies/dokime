<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tooltip } from "bits-ui";
  import { onMount } from "svelte";
  import DesktopUpdateDialog from "../../../components/DesktopUpdateDialog.svelte";
  import { copyTraceabilityText } from "../../../components/traceability/clipboard";
  import Button from "../../../components/ui/Button.svelte";
  import { buttonVariants } from "../../../components/ui/button-variants";
  import type { BuildIdentity } from "../../../lib/contracts/connection";
  import { commands } from "../../../lib/desktop/commands";
  import { isTauriDesktop } from "../../../lib/surface-environment";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import { buildLabel } from "./build-version";

  let { brandLabel, backend }: { brandLabel: string; backend: BuildIdentity | null } = $props();
  let desktop = $state.raw<BuildIdentity | null>(null);
  let updatesOpen = $state(false);
  const native = isTauriDesktop();
  const build = $derived(desktop ?? { productVersion: __APP_VERSION__, sourceRevision: null, channel: null });
  const backendDiffers = $derived(
    Boolean(
      backend &&
        (backend.productVersion !== build.productVersion ||
          (backend.sourceRevision ?? "unknown") !== (build.sourceRevision ?? "unknown") ||
          (backend.channel ?? "") !== (build.channel ?? "")),
    ),
  );

  onMount(() => {
    if (!native) {
      return;
    }
    let active = true;
    void commands.desktopBuild().then(
      (value) => {
        if (active) {
          desktop = value;
        }
      },
      () => undefined,
    );
    return () => {
      active = false;
    };
  });
</script>

{#snippet buildVersion(
  identity: BuildIdentity,
)}
  {@const label = buildLabel(identity)}
  {@const commit = label.commit}
  <span class="font-mono text-sm">
    {label.version}
    (
    {#if commit}
      <Tooltip.Root>
        <Tooltip.Trigger
          class={buttonVariants({ variant: "link", size: "inline" })}
          aria-label={`Copy Commit ${commit} to Clipboard`}
          onclick={() => void copyTraceabilityText(commit)}
          >{label.revision}</Tooltip.Trigger
        >
        <Tooltip.Portal>
          <Tooltip.Content
            class="z-50 break-all rounded-lg bg-popover px-3 py-2 text-xs text-popover-foreground ring-1 ring-border"
            side="top"
          >
            Copy: {commit}
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    {:else}
      {label.revision}
    {/if}
    ,
    <span class="text-muted-foreground">{label.channel}</span>)
  </span>
{/snippet}

<Tooltip.Provider>
  <SettingsCard title="About">
    <SettingsRow label="Application">{brandLabel}</SettingsRow>
    <SettingsRow
      label={desktop ? "Desktop Version" : "Version"}
      description={desktop ? "The build of this desktop application." : undefined}
    >
      {@render buildVersion(build)}
    </SettingsRow>
    {#if backend && backendDiffers}
      <SettingsRow description="The build of the connected backend, which may run elsewhere." label="Backend Version">
        {@render buildVersion(backend)}
      </SettingsRow>
    {/if}
    <SettingsRow description="Dokime © Stokker Technologies, licensed under the Apache License, Version 2.0." label="License">
      Apache License 2.0
    </SettingsRow>
    <div class="flex justify-end px-4 py-3">
      <Button data-testid="desktop-updates-open" variant="outline" disabled={!native} onclick={() => (updatesOpen = true)}>
        Check for Updates
      </Button>
    </div>
  </SettingsCard>
</Tooltip.Provider>
<DesktopUpdateDialog {build} open={updatesOpen} onOpenChange={(open) => (updatesOpen = open)} />
