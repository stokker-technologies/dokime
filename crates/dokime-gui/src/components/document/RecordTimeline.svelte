<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { HubRecord } from "../../lib/types";
  import { recordMessage, recordTitle } from "../../lib/dashboard";

  export type RecordTimelineProps = { live?: boolean; records: HubRecord[] };

  function compareRecordTimelineOrder(left: HubRecord, right: HubRecord): number {
    const timestamp = right.timestamp.localeCompare(left.timestamp);
    if (timestamp !== 0) {
      return timestamp;
    }
    return (right.sequence ?? 0) - (left.sequence ?? 0);
  }

  function stringifyRecordDetails(details: Record<string, unknown>): string {
    try {
      return JSON.stringify(details);
    } catch {
      return "";
    }
  }

  /**
   * The one line a record row prints.
   *
   * A record carries a title and a message, and printing both put the same
   * sentence on the row twice. The message is the record's own words; the title
   * is what stands in when it has none.
   */
  function recordLine(record: HubRecord): string {
    return recordMessage(record) || recordTitle(record);
  }
</script>

<script lang="ts">
  import { Select } from "bits-ui";
  import { ChevronDownIcon, Search } from "lucide-svelte";
  import { onDestroy, tick } from "svelte";
  import { recordClockLabel, recordLevelLabel, recordLevelVariant, recordToneRowClass } from "../../lib/record-display";
  import { observeRowWindow, ROW_WINDOW_THRESHOLD, rowWindow, type RowWindowMeasurements } from "../../lib/row-window";
  import { cn } from "../../lib/utils";
  import RecordDetailsDialog from "../RecordDetailsDialog.svelte";
  import { badgeVariants } from "../ui/badge-variants";
  import Button from "../ui/Button.svelte";

  let { live = true, records }: RecordTimelineProps = $props();
  const searchText = new Map<string, string[]>();
  let query = $state("");
  let deferredQuery = $state("");
  let queryFrame: number | undefined;
  let recordType = $state("all");
  let selectedRecord = $state.raw<HubRecord | null>(null);
  let selectedTrigger: HTMLButtonElement | undefined;
  let rowsElement: HTMLOListElement | undefined = $state();
  let measurements = $state.raw<RowWindowMeasurements>({ pitch: null, offset: 0, height: 0 });

  const recordTypes = $derived(
    [...new Set(records.map((record) => record.record_type))]
      .filter((type) => type.trim().length > 0)
      .sort((left, right) => left.localeCompare(right)),
  );
  const orderedRecords = $derived([...records].sort(compareRecordTimelineOrder));
  const visibleRecords = $derived.by(() => {
    if (!live) {
      return orderedRecords;
    }
    // Only The Asterisk And Question Mark Are Wildcards.
    const pattern = deferredQuery
      .trim()
      .split(/([*?])/)
      .map((part) => {
        if (part === "*") {
          return ".*";
        }
        if (part === "?") {
          return ".";
        }
        return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      })
      .join("");
    const matcher = pattern ? new RegExp(pattern, "is") : null;
    return orderedRecords.filter((record) => {
      if (recordType !== "all" && record.record_type !== recordType) {
        return false;
      }
      if (!matcher) {
        return true;
      }
      let fields = searchText.get(record.record_id);
      if (!fields) {
        fields = [record.record_type, record.source, recordTitle(record), recordMessage(record), stringifyRecordDetails(record.details)];
        searchText.set(record.record_id, fields);
      }
      return fields.some((value) => matcher.test(value));
    });
  });
  const windowed = $derived(visibleRecords.length >= ROW_WINDOW_THRESHOLD);
  const slice = $derived(rowWindow(visibleRecords.length, measurements.pitch, measurements.offset, measurements.height));

  $effect(() => {
    const retained = new Set(records.map((record) => record.record_id));
    for (const id of searchText.keys()) {
      if (!retained.has(id)) {
        searchText.delete(id);
      }
    }
  });

  $effect(() => {
    if (!windowed || !rowsElement) {
      return;
    }
    return observeRowWindow(rowsElement, (next) => {
      measurements = next;
    });
  });

  function search(event: Event & { currentTarget: HTMLInputElement }) {
    query = event.currentTarget.value;
    if (queryFrame !== undefined) {
      return;
    }
    queryFrame = requestAnimationFrame(() => {
      queryFrame = undefined;
      deferredQuery = query;
    });
  }

  onDestroy(() => {
    if (queryFrame !== undefined) {
      cancelAnimationFrame(queryFrame);
    }
  });

  function openRecord(record: HubRecord, trigger: HTMLButtonElement) {
    selectedTrigger = trigger;
    selectedRecord = record;
  }

  async function closeRecord() {
    selectedRecord = null;
    await tick();
    selectedTrigger?.focus();
    selectedTrigger = undefined;
  }
</script>

<div class="grid gap-2">
  {#if live}
    <div class="grid items-center gap-x-3 text-sm sm:grid-cols-[minmax(0,1fr)_--spacing(56)]">
      <div class="relative">
        <Search aria-hidden="true" class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
        <input
          aria-label="Search Records"
          class="h-[var(--bar-control-height)] w-full min-w-0 rounded-lg border border-input bg-transparent py-1 pl-8 pr-3 outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30"
          data-slot="input"
          oninput={search}
          placeholder="Search Records (* Any Text, ? One Character)"
          value={query}
        >
      </div>
      <Select.Root type="single" bind:value={recordType}>
        <Select.Trigger
          aria-label="Record Type"
          data-slot="select-trigger"
          class="flex h-[var(--bar-control-height)] w-full items-center justify-between gap-2 rounded-lg border border-input bg-background py-2 pr-2 pl-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
        >
          {recordType === "all" ? "All Records" : recordType}
          <ChevronDownIcon class="size-4 text-muted-foreground" />
        </Select.Trigger>
        <Select.Portal>
          <Select.Content class="z-50 min-w-36 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border" align="start">
            <Select.Item value="all" label="All Records" class="rounded-lg px-2 py-1 text-sm data-highlighted:bg-accent"
              >All Records</Select.Item
            >
            {#each recordTypes as value (value)}
              <Select.Item {value} label={value} class="rounded-lg px-2 py-1 text-sm data-highlighted:bg-accent">{value}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Portal>
      </Select.Root>
    </div>
  {/if}

  <div class="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-x-3 border-b pb-1 text-xs text-muted-foreground">
    <span class="truncate">
      {live
        ? `Showing ${visibleRecords.length} of ${records.length} records`
        : `${records.length} ${records.length === 1 ? "record" : "records"}`}
    </span>
    <span class="justify-self-end">Newest First</span>
  </div>

  {#if visibleRecords.length}
    <ol class="grid border-t" data-testid="record-rows" bind:this={rowsElement}>
      {#if slice.before}
        <li aria-hidden="true" style:height={`${slice.before}px`}></li>
      {/if}
      {#each visibleRecords.slice(slice.first, slice.last) as record, index (record.record_id)}
        {@const variant = recordLevelVariant(record.level)}
        <li
          class={cn("border-b border-l-2 border-b-border/60 last:border-b-0", recordToneRowClass(record.display.tone))}
          data-window-row={slice.first + index}
          aria-posinset={slice.first + index + 1}
          aria-setsize={visibleRecords.length}
        >
          <Button
            variant="plain"
            size="inline"
            aria-haspopup="dialog"
            class="grid w-full grid-cols-[--spacing(24)_--spacing(22)_--spacing(40)_minmax(0,1fr)_auto] items-start gap-x-3 px-3 py-2 text-left font-mono text-xs tabular-nums hover:bg-muted/40"
            onclick={(event) => openRecord(record, event.currentTarget)}
          >
            <time class="truncate text-muted-foreground" datetime={record.timestamp}>{recordClockLabel(record.timestamp)}</time>
            <span data-slot="badge" data-variant={variant} class={cn(badgeVariants({ variant }), "h-4 justify-self-start px-1 text-xs")}>
              {recordLevelLabel(record.level)}
            </span>
            <span class="truncate text-muted-foreground">{record.source}</span>
            <span class="min-w-0 truncate">{recordLine(record)}</span>
            <span class="truncate text-muted-foreground">{record.record_type}</span>
          </Button>
        </li>
      {/each}
      {#if slice.after}
        <li aria-hidden="true" style:height={`${slice.after}px`}></li>
      {/if}
    </ol>
  {:else}
    <p class="py-6 text-center text-sm text-muted-foreground">
      {live && (query.trim() || recordType !== "all") ? "No records match the current filters." : "No records yet."}
    </p>
  {/if}
</div>
<RecordDetailsDialog selected={selectedRecord} onClose={closeRecord} />
