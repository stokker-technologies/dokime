<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import TagChip from "../../components/suite-picker/TagChip.svelte";
  import type { RuntimeInstalledPluginRecord } from "../settings/installed-plugins/installed-plugin-contracts";

  let { brandLabel, plugin, source }: { brandLabel: string; plugin?: RuntimeInstalledPluginRecord; source: string } = $props();
  const owner = $derived(plugin?.owner);
  const label = $derived(plugin?.label && plugin.label !== plugin.plugin_id ? `${plugin.label} · ${plugin.plugin_id}` : source);
</script>

<div class="flex min-w-0 flex-wrap items-center gap-1">
  <TagChip class="font-mono" {label} title={`Registered by Plugin ${label}`} />
  {#if owner?.kind === "host"}
    <TagChip label="Host" title={`Registered by the ${brandLabel} Host`} />
  {:else if owner?.kind === "embedded"}
    <TagChip label="Built-In" title={`Built into This Application (${owner.generation})`} />
  {:else if owner?.kind === "bundle"}
    <TagChip
      label={owner.installation ? "Installed" : "Configured"}
      title={`${owner.installation ? "Installed" : "Configured"} Bundle ${owner.package}`}
    />
  {/if}
  {#if plugin?.version}
    <TagChip class="font-mono" label={/^v\d/i.test(plugin.version) ? plugin.version : `v${plugin.version}`} title="Plugin Version" />
  {/if}
  {#if plugin && !plugin.enabled}
    <TagChip label="Disabled" title="Plugin Is Disabled" />
  {/if}
</div>
