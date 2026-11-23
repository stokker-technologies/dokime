<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "../../components/ui/Button.svelte";
  import ActiveCycleSummary from "./ActiveCycleSummary.svelte";
  import type { OpenCycle } from "./active-cycle-model";
  import { NO_ACTIVE_CYCLE, NO_CHOSEN_CYCLE } from "./template-dialog-model";

  let {
    cycle,
    hasOpenCycles,
    onChange,
    onNewCycle,
  }: {
    cycle: OpenCycle | null;
    hasOpenCycles: boolean;
    onChange: () => void;
    onNewCycle: () => void;
  } = $props();
</script>

<div
  class="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-lg border bg-muted/50 px-3 py-2 text-sm"
  data-testid="template-active-cycle"
  role="status"
>
  <span class="text-muted-foreground">Active Cycle</span>
  {#if cycle}
    <ActiveCycleSummary {cycle} />
  {:else}
    <span class="text-muted-foreground">{hasOpenCycles ? NO_CHOSEN_CYCLE : NO_ACTIVE_CYCLE}</span>
  {/if}
  <Button onclick={cycle || hasOpenCycles ? onChange : onNewCycle} size="sm" variant="outline">
    {cycle ? "Change" : hasOpenCycles ? "Choose" : "New Cycle"}
  </Button>
</div>
