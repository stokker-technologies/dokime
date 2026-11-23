<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Progress from "../../components/ui/Progress.svelte";
  import { progressCount } from "./control-model";
  import ExecutionSectionCard from "./ExecutionSectionCard.svelte";

  type Props = {
    label: string;
    state: string;
    detail?: string | null;
    progress?: { completed: number; phase: string; total: number } | null;
    actions?: Snippet;
    children?: Snippet;
  };

  let { actions, children, detail, label, progress, state }: Props = $props();
</script>

<ExecutionSectionCard title="Control">
  <div
    class="grid min-h-[var(--bar-control-height)] min-w-0 grid-cols-[minmax(0,auto)_minmax(0,1fr)_auto_auto] items-center gap-x-4 gap-y-2 px-1"
  >
    <p aria-live="polite" class="min-w-0 text-sm leading-5 [overflow-wrap:anywhere]" data-control-state>
      {state}
      {#if detail}
        <span class="ml-2 font-mono text-xs text-muted-foreground">{detail}</span>
      {/if}
    </p>
    {#if progress}
      <Progress
        aria-label="Execution Progress"
        class="h-1.5 min-w-16 rounded-full [&>[data-slot=progress-indicator]]:rounded-full"
        value={progress.total > 0 ? (progress.completed / progress.total) * 100 : 0}
      />
      <span class="text-xs text-muted-foreground tabular-nums">{progressCount(progress)}</span>
    {:else}
      <span></span>
      <span></span>
    {/if}
    <fieldset aria-label={label} class="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]">
      {@render actions?.()}
    </fieldset>
    {#if children}
      <div class="col-span-4 min-w-0">{@render children()}</div>
    {/if}
  </div>
</ExecutionSectionCard>
