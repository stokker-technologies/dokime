<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "../components/ui/Button.svelte";
  import { recordMessage } from "../lib/dashboard";
  import { recordClockLabel, recordLevelLabel, recordLevelTextClass } from "../lib/record-display";
  import type { HubRecord } from "../lib/types";
  import { cn } from "../lib/utils";

  let { record, heightPx }: { record: HubRecord; heightPx: number } = $props();
  let expanded = $state(false);
  const level = $derived(record.level);
  const label = $derived(typeof record.details.trace_level === "string" ? record.details.trace_level : recordLevelLabel(level));
</script>

<Button
  variant="plain"
  size="inline"
  class={cn(
    "grid w-full grid-cols-[--spacing(20)_--spacing(15)_--spacing(36)_minmax(0,1fr)] items-start gap-2 px-[var(--shell-inset)] text-left font-mono text-xs tabular-nums hover:bg-muted/40",
    expanded ? "py-1" : "overflow-hidden",
  )}
  aria-expanded={expanded}
  onclick={() => {
    expanded = !expanded;
  }}
  style={expanded ? undefined : `height: ${heightPx}px`}
>
  <span class="truncate text-muted-foreground">{recordClockLabel(record.timestamp)}</span>
  <span class={cn("truncate uppercase", recordLevelTextClass(level))}>{label}</span>
  <span class="truncate text-muted-foreground">{record.source}</span>
  <span class={cn("min-w-0", expanded ? "break-words whitespace-pre-wrap" : "truncate")}>{recordMessage(record)}</span>
</Button>
