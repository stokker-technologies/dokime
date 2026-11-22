<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { GripVertical, RotateCcw } from "lucide-svelte";
  import { onDestroy } from "svelte";
  import Metric from "../../Metric.svelte";
  import Button from "../../ui/Button.svelte";
  import {
    metricGroupWidthRem,
    moveMetricGroup,
    orderMetricGroups,
    type OverviewMetricGroup,
    readMetricFlowOrder,
    removeMetricFlowOrder,
    writeMetricFlowOrder,
  } from "./metric-model";

  let { groups, storageKey }: { groups: OverviewMetricGroup[]; storageKey: string } = $props();
  let order = $state.raw<{ key: string; groupIds: string[] } | null>(null);
  const savedOrder = $derived(order?.key === storageKey ? order.groupIds : readMetricFlowOrder(storageKey));
  const orderedGroups = $derived(orderMetricGroups(groups, savedOrder));
  let draggedGroupId = $state<string | null>(null);
  let targetGroupId = $state<string | null>(null);
  let drag: AbortController | undefined;

  onDestroy(() => drag?.abort());

  // Native File Drops Reserve HTML Drag Events.
  function startDrag(event: PointerEvent & { currentTarget: HTMLButtonElement }, sourceGroupId: string) {
    if (event.button !== 0) {
      return;
    }
    event.preventDefault();
    const grip = event.currentTarget;
    grip.setPointerCapture(event.pointerId);
    drag?.abort();
    drag = new AbortController();
    const { signal } = drag;
    const key = storageKey;
    const groupIds = orderedGroups.map((group) => group.groupId);
    draggedGroupId = sourceGroupId;
    targetGroupId = null;

    const move = (moved: PointerEvent) => {
      targetGroupId = dropTarget(moved, sourceGroupId)?.groupId ?? null;
    };
    const finish = (ended: PointerEvent) => {
      drag?.abort();
      drag = undefined;
      draggedGroupId = null;
      targetGroupId = null;
      const target = ended.type === "pointerup" ? dropTarget(ended, sourceGroupId) : null;
      if (target && key === storageKey) {
        const next = moveMetricGroup(groupIds, sourceGroupId, target.groupId, target.insertAfter);
        writeMetricFlowOrder(key, next);
        order = { key, groupIds: next };
      }
    };
    grip.addEventListener("pointermove", move, { signal });
    grip.addEventListener("pointerup", finish, { signal });
    grip.addEventListener("lostpointercapture", finish, { signal });
  }

  function resetOrder() {
    removeMetricFlowOrder(storageKey);
    order = { key: storageKey, groupIds: [] };
  }

  function dropTarget(event: PointerEvent, sourceGroupId: string): { groupId: string; insertAfter: boolean } | null {
    const element = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>("[data-metric-group-id]");
    const groupId = element?.dataset.metricGroupId;
    if (!element || !groupId || groupId === sourceGroupId) {
      return null;
    }
    const bounds = element.getBoundingClientRect();
    return { groupId, insertAfter: event.clientX > bounds.left + bounds.width / 2 };
  }
</script>

<div class="grid gap-3" data-testid="metric-flow">
  {#if savedOrder.length}
    <div class="flex justify-end">
      <Button
        variant="outline"
        size="icon"
        aria-label="Reset Metric Group Order"
        class="text-muted-foreground"
        title="Reset Metric Group Order"
        onclick={resetOrder}
      >
        <RotateCcw size={14} />
      </Button>
    </div>
  {/if}
  <div class="flex flex-wrap items-start gap-3">
    {#each orderedGroups as { groupId, title, metrics } (groupId)}
      <section
        aria-label={title}
        class={[
          "grid min-w-[min(100%,var(--metric-group-width))] max-w-full flex-[1_1_var(--metric-group-width)] gap-2 rounded-lg border p-1",
          draggedGroupId === groupId || targetGroupId === groupId ? "border-primary/40" : "border-transparent",
          draggedGroupId === groupId && "bg-primary/5 opacity-70",
        ]}
        data-metric-group-id={groupId}
        data-testid={`metric-group-${groupId}`}
        style:--metric-group-width={`${metricGroupWidthRem(metrics.length)}rem`}
      >
        <div class="flex min-w-0 items-center gap-1">
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label={`Drag ${title}`}
            class="size-6 cursor-grab touch-none text-muted-foreground active:cursor-grabbing"
            title={`Drag ${title}`}
            onpointerdown={(event) => startDrag(event, groupId)}
          >
            <GripVertical size={14} />
          </Button>
          <p class="min-w-0 truncate text-xs font-bold uppercase text-muted-foreground">{title}</p>
        </div>
        <div class="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(min(100%,9.5rem),1fr))]">
          {#each metrics as metric}
            <Metric {...metric} />
          {/each}
        </div>
      </section>
    {/each}
  </div>
</div>
