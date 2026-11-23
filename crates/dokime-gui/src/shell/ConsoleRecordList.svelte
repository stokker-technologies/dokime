<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ArrowDownToLine } from "lucide-svelte";
  import Button from "../components/ui/Button.svelte";
  import { fixedRowWindow } from "../lib/row-window";
  import type { HubRecord } from "../lib/types";
  import { cn } from "../lib/utils";
  import { CONSOLE_LIST_PADDING_PX, CONSOLE_ROW_HEIGHT_PX, consoleFollowsTail, consoleTailPinned } from "./console-follow";
  import ConsoleRecordRow from "./ConsoleRecordRow.svelte";

  type Props = {
    class?: string;
    empty: string;
    follow: boolean;
    onFollowChange?: (follow: boolean) => void;
    rows: HubRecord[];
  };

  let { class: className, empty, follow, onFollowChange, rows }: Props = $props();
  let viewport = $state<HTMLDivElement>();
  let height = $state(0);
  let scrollTop = $state(0);
  // Scroll Events Update The Operator's Position Before The Drawer Re-Renders.
  let pinned = true;
  const overscan = 12;
  const tail = $derived(rows.at(-1)?.record_id);
  const slice = $derived(fixedRowWindow(rows.length, CONSOLE_ROW_HEIGHT_PX, scrollTop, height, overscan, CONSOLE_LIST_PADDING_PX));
  const visible = $derived(rows.slice(slice.first, slice.last));
  const contentHeight = $derived(slice.before + visible.length * CONSOLE_ROW_HEIGHT_PX + slice.after);

  $effect(() => {
    if (viewport && height > 0 && tail && consoleFollowsTail(follow, pinned)) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  });

  function onScroll(event: Event & { currentTarget: HTMLDivElement }) {
    const node = event.currentTarget;
    scrollTop = node.scrollTop;
    pinned = consoleTailPinned(node);
    if (pinned !== follow) {
      onFollowChange?.(pinned);
    }
  }

  function jumpToLatest() {
    pinned = true;
    onFollowChange?.(true);
  }
</script>

<div class="relative grid min-h-0 grid-rows-[minmax(0,1fr)]">
  <div
    class={cn(
      "min-h-0 overflow-y-scroll [overflow-anchor:none] [scrollbar-gutter:stable] [&::-webkit-scrollbar]:w-2.5 [&::-webkit-scrollbar-track]:bg-muted/60 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:bg-muted-foreground/40 hover:[&::-webkit-scrollbar-thumb]:bg-muted-foreground/60",
      className,
    )}
    bind:this={viewport}
    bind:clientHeight={height}
    onscroll={onScroll}
  >
    {#if rows.length}
      <div class="relative" style:height={`${contentHeight}px`}>
        <div class="absolute inset-x-0" style:top={`${slice.before}px`}>
          {#each visible as record (record.record_id)}
            <ConsoleRecordRow {record} heightPx={CONSOLE_ROW_HEIGHT_PX} />
          {/each}
        </div>
      </div>
    {:else}
      <p class="px-[var(--shell-inset)] py-2 text-sm text-muted-foreground">{empty}</p>
    {/if}
  </div>
  {#if onFollowChange && !follow}
    <Button
      class="absolute right-[var(--shell-inset)] bottom-2 shadow-sm"
      data-testid="console-follow-latest"
      onclick={jumpToLatest}
      size="sm"
      variant="secondary"
    >
      <ArrowDownToLine data-icon="inline-start" />
      Jump to Latest
    </Button>
  {/if}
</div>
