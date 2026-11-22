<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Search } from "lucide-svelte";
  import Button from "../../components/ui/Button.svelte";
  import Input from "../../components/ui/Input.svelte";
  import CycleStatusPill from "../cycles-home/CycleStatusPill.svelte";
  import { caseCountLabel, filterOpenCycles, type OpenCycle } from "./active-cycle-model";
  import { NO_ACTIVE_CYCLE } from "./template-dialog-model";

  let {
    activeId,
    cycles,
    loading,
    onChoose,
    onNewCycle,
    onSearch,
    search,
  }: {
    activeId: string | null;
    cycles: readonly OpenCycle[];
    loading: boolean;
    onChoose: (cycleId: string) => void;
    onNewCycle: () => void;
    onSearch: (text: string) => void;
    search: string;
  } = $props();
  const shown = $derived(filterOpenCycles(cycles, search));
  const row = "col-span-full grid grid-cols-subgrid items-center px-3";
</script>

<div class="grid min-h-0 gap-3">
  <div class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--bar-gap)]">
    <span class="relative">
      <Search aria-hidden="true" class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        aria-label="Search Open Cycles"
        class="h-[var(--bar-control-height)] pl-8"
        oninput={(event) => onSearch(event.currentTarget.value)}
        placeholder="Search by Name or Identifier"
        value={search}
      />
    </span>
    <Button class="h-[var(--bar-control-height)]" onclick={onNewCycle}>New Cycle</Button>
  </div>
  {#if shown.length}
    <div class="grid max-h-[60vh] grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] gap-x-4 overflow-y-auto text-sm" data-testid="open-cycles">
      <div aria-hidden="true" class={[row, "pb-1 text-xs text-muted-foreground"]}>
        <span>Cycle</span>
        <span>State</span>
        <span>Created</span>
        <span>Identifier</span>
        <span class="text-right">Members</span>
      </div>
      {#each shown as cycle (cycle.cycleId)}
        <Button
          variant="plain"
          size="inline"
          aria-pressed={cycle.cycleId === activeId}
          class={[row, "min-h-[var(--control-height)] rounded-lg py-2 text-left hover:bg-muted", cycle.cycleId === activeId && "bg-muted"]}
          onclick={() => onChoose(cycle.cycleId)}
        >
          <span class="truncate font-medium">{cycle.label}</span>
          <CycleStatusPill label={cycle.statusLabel} tone={cycle.statusTone} />
          <span class="font-mono text-xs text-muted-foreground tabular-nums">{cycle.createdLabel}</span>
          <span class="font-mono text-xs text-muted-foreground tabular-nums">{cycle.shortId}</span>
          <span class="text-right font-mono text-xs text-muted-foreground tabular-nums">{caseCountLabel(cycle.caseCount)}</span>
        </Button>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-muted-foreground" role="status">
      {loading ? "Loading Open Cycles" : cycles.length ? "No Open Cycle Matches This Search" : NO_ACTIVE_CYCLE}
    </p>
  {/if}
</div>
