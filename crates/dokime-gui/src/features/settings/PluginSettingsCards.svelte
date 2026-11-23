<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { HostCardTone, type SettingsCard, SettingsCardState } from "@generated/host/model";
  import Badge from "../../components/ui/Badge.svelte";
  import type { Wire } from "../../lib/rpc/canonical";
  import SettingsPanel from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";

  let { cards }: { cards: Wire<SettingsCard>[] } = $props();

  function stateMeta(state: SettingsCardState): {
    label: string;
    variant: "secondary" | "success" | "warning" | "destructive" | "outline";
  } {
    switch (state) {
      case SettingsCardState.READY:
        return { label: "Ready", variant: "success" };
      case SettingsCardState.DISABLED:
        return { label: "Disabled", variant: "secondary" };
      case SettingsCardState.DISCONNECTED:
        return { label: "Disconnected", variant: "warning" };
      case SettingsCardState.STALE:
        return { label: "Stale", variant: "warning" };
      case SettingsCardState.ERROR:
        return { label: "Error", variant: "destructive" };
      case SettingsCardState.PERMISSION_DENIED:
        return { label: "Permission Denied", variant: "destructive" };
      default:
        return { label: "Unknown", variant: "outline" };
    }
  }

  function toneMeta(tone: HostCardTone): {
    label: string;
    variant: "secondary" | "success" | "warning" | "destructive" | "outline";
  } {
    switch (tone) {
      case HostCardTone.SUCCESS:
        return { label: "OK", variant: "success" };
      case HostCardTone.WARNING:
        return { label: "Warning", variant: "warning" };
      case HostCardTone.DANGER:
        return { label: "Error", variant: "destructive" };
      case HostCardTone.INFO:
        return { label: "Info", variant: "outline" };
      default:
        return { label: "Status", variant: "secondary" };
    }
  }
</script>

{#each cards as card (`${card.pluginId}:${card.cardId}`)}
  {@const state = stateMeta(card.state)}
  <SettingsPanel description={card.description} title={card.label}>
    {#snippet footer()}
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="flex min-w-0 flex-col gap-1">
          <Badge variant={state.variant}>{state.label}</Badge>
        </div>
      </div>
    {/snippet}
    {#each card.body?.blocks ?? [] as block}
      {@const variant = block.kind?.kind}
      {#if variant?.case === "text"}
        <div class="px-4 py-2 text-sm text-muted-foreground">{variant.value}</div>
      {:else if variant?.case === "status"}
        {@const tone = toneMeta(variant.value.tone)}
        <SettingsRow description={variant.value.detail} label={variant.value.label}>
          <Badge variant={tone.variant}>{tone.label}</Badge>
        </SettingsRow>
      {:else if variant?.case === "keyValueList"}
        {#each variant.value.items ?? [] as item (item.label)}
          <SettingsRow description={item.detail} label={item.label}>
            <span class="min-w-0 truncate text-sm text-muted-foreground" title={item.value}>{item.value}</span>
          </SettingsRow>
        {/each}
      {:else if variant?.case === "metrics"}
        <div class="grid gap-3 px-4 py-3 sm:grid-cols-2 lg:grid-cols-3">
          {#each variant.value.items ?? [] as item (item.label)}
            <div class="min-w-0 rounded-lg border bg-muted/20 px-3 py-2">
              <div class="truncate text-xs text-muted-foreground">{item.label}</div>
              <div class="mt-1 flex items-baseline gap-1">
                <span class="truncate text-lg font-semibold">{item.value}</span>
                {#if item.unit}
                  <span class="text-xs text-muted-foreground">{item.unit}</span>
                {/if}
              </div>
              {#if item.trend}
                <div class="mt-1 truncate text-xs text-muted-foreground">{item.trend}</div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/each}
  </SettingsPanel>
{/each}
