<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { Braces, Clipboard, Clock } from "lucide-svelte";
  import { formatValue, recordTime, recordTitle } from "../lib/dashboard";
  import { failureOf } from "../lib/notification-details";
  import { isPathLike } from "../lib/path-display";
  import { recordToneShellClass } from "../lib/record-display";
  import { absoluteTimestampLabel } from "../lib/time-format";
  import type { HubRecord } from "../lib/types";
  import { cn, isRecord } from "../lib/utils";
  import ErrorCard from "./ErrorCard.svelte";
  import PathPill from "./PathPill.svelte";
  import RecordGlyph from "./RecordGlyph.svelte";
  import { badgeVariants } from "./ui/badge-variants";
  import Button from "./ui/Button.svelte";
  import DialogContent from "./ui/DialogContent.svelte";

  let { selected, onClose }: { selected: HubRecord | null; onClose: () => void } = $props();
  const stacktraceKeys = new Set(["stack", "stacktrace", "stack_trace", "traceback", "trace", "exception_stack", "error_stack"]);
  const details = $derived(selected ? Object.entries(selected.details).filter(([key]) => !stacktraceKeys.has(key.toLowerCase())) : []);

  function changeOpen(open: boolean) {
    if (!open) {
      onClose();
    }
  }

  function recordStacktrace(record: HubRecord): string | null {
    return findStacktrace(record.details) ?? findStacktrace({ message: record.message });
  }

  function findStacktrace(value: unknown): string | null {
    if (typeof value === "string") {
      return looksLikeStacktrace(value) ? value : null;
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        const nested = findStacktrace(item);
        if (nested) {
          return nested;
        }
      }
      return null;
    }
    if (!isRecord(value)) {
      return null;
    }
    for (const [key, nestedValue] of Object.entries(value)) {
      if (stacktraceKeys.has(key.toLowerCase())) {
        return formatDetailValue(nestedValue);
      }
      const nested = findStacktrace(nestedValue);
      if (nested) {
        return nested;
      }
    }
    return null;
  }

  function looksLikeStacktrace(value: string): boolean {
    const lowered = value.toLowerCase();
    return (
      lowered.includes("traceback") || lowered.includes("stack trace") || lowered.includes("\n    at ") || lowered.includes('\n  file "')
    );
  }

  function formatDetailValue(value: unknown): string {
    if (typeof value === "string") {
      return value;
    }
    if (typeof value === "number" || typeof value === "boolean") {
      return String(value);
    }
    if (value === null || value === undefined) {
      return "-";
    }
    return JSON.stringify(value, null, 2);
  }
</script>

{#snippet metadata(
  label: string,
  value: unknown,
)}
  {@const formatted = formatValue(value, undefined, label)}
  <div class="grid min-w-0 gap-2 rounded-lg border bg-muted/20 p-2">
    <dt class="truncate text-xs font-semibold uppercase text-muted-foreground">{label}</dt>
    <dd class="text-sm [overflow-wrap:anywhere]">
      {#if isPathLike(formatted, label)}
        <PathPill value={formatted} />
      {:else}
        {formatted}
      {/if}
    </dd>
  </div>
{/snippet}

{#snippet rawBlock(
  label: string,
  value: string,
)}
  <section class="grid min-w-0 gap-2">
    <div class="flex items-center justify-between gap-3">
      <h3 class="inline-flex min-w-0 items-center gap-2 text-sm font-semibold">
        <Braces class="shrink-0" size={15} />
        <span class="truncate">{label}</span>
      </h3>
      <Button aria-label={`Copy ${label}`} size="sm" variant="outline" onclick={() => void navigator.clipboard?.writeText(value)}>
        <Clipboard size={14} />
        Copy
      </Button>
    </div>
    <pre
      class="max-h-[min(--spacing(96),38vh)] min-h-32 max-w-full overflow-auto rounded-lg border bg-muted/35 p-3 text-xs leading-relaxed"
    ><code>{value}</code></pre>
  </section>
{/snippet}

<Dialog.Root open={selected !== null} onOpenChange={changeOpen}>
  {#if selected}
    {@const record = selected}
    {@const title = recordTitle(record)}
    {@const stacktrace = recordStacktrace(record)}
    {@const variant = record.level === "error" ? "destructive" : "secondary"}
    <DialogContent
      class="grid max-h-[92vh] grid-rows-[auto_minmax(0,1fr)] gap-0 overflow-hidden p-0 sm:max-w-4xl"
      aria-describedby={undefined}
    >
      <header class="grid gap-3 border-b p-4">
        <div class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] gap-3">
          <RecordGlyph class="mt-1" display={record.display} />
          <div class="min-w-0">
            <div class="mb-2 flex flex-wrap items-center gap-2">
              <span
                data-slot="badge"
                data-variant={variant}
                class={cn(badgeVariants({ variant }), recordToneShellClass(record.display.tone))}
              >
                {record.level}
              </span>
              <span class="inline-flex items-center gap-2 text-xs text-muted-foreground">
                <Clock size={13} />
                {recordTime(record)}
              </span>
              {#if record.sequence != null}
                <span class="text-xs text-muted-foreground">#{record.sequence}</span>
              {/if}
            </div>
            <Dialog.Title data-slot="dialog-title" class="text-xl font-semibold leading-tight [overflow-wrap:anywhere]"
              >{title}</Dialog.Title
            >
            <p class="mt-1 text-sm text-muted-foreground [overflow-wrap:anywhere]">{record.record_type}</p>
          </div>
        </div>
      </header>
      <div class="min-h-0 overflow-y-auto p-4">
        <div class="grid min-w-0 gap-4">
          {#if record.level === "error"}
            <ErrorCard failure={failureOf(record.message ?? title, "Record Failed", record.record_id)} />
          {/if}
          <dl class="grid gap-2 text-sm sm:grid-cols-2 xl:grid-cols-4">
            {@render metadata("Source", record.source)}
            {@render metadata("Timestamp", absoluteTimestampLabel(record.timestamp))}
            {@render metadata("Session", record.session_id ?? "-")}
            {@render metadata("Record ID", record.record_id)}
            {@render metadata("Cycle", record.scope?.cycleId ?? "-")}
            {@render metadata("Execution", record.scope?.executionId ?? "-")}
            {@render metadata("Case", record.scope?.caseId ?? "-")}
          </dl>
          {#if details.length}
            <section class="grid gap-2">
              <h3 class="text-sm font-semibold">Details</h3>
              <dl class="grid gap-2 text-sm sm:grid-cols-2">
                {#each details as [key, value] (key)}
                  {@render metadata(key, value ?? formatDetailValue(value))}
                {/each}
              </dl>
            </section>
          {/if}
          {#if stacktrace}
            {@render rawBlock("Stacktrace", stacktrace)}
          {/if}
          {@render rawBlock("Raw Record", JSON.stringify(record, null, 2))}
        </div>
      </div>
    </DialogContent>
  {/if}
</Dialog.Root>
