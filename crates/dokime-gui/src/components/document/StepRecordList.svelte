<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { LoaderCircle } from "lucide-svelte";
  import { formatDuration } from "../../lib/format";
  import type { StepExecutionRecord } from "../../lib/types";
  import ErrorCard from "../ErrorCard.svelte";
  import Badge from "../ui/Badge.svelte";
  import { sortedStepRecords, stepBadgeVariant, stepFailure, stepRecordLabel } from "./step-records";

  type Props = {
    records: StepExecutionRecord[];
    emptyMessage: string;
    maxItems?: number | null;
  };

  let { records, emptyMessage, maxItems }: Props = $props();
  const sorted = $derived(sortedStepRecords(records));
  const visible = $derived(maxItems == null ? sorted : sorted.slice(0, Math.max(0, maxItems)));
  const orderWidth = $derived(Math.max(2, String(visible.length).length));
  const showRecordType = $derived(new Set(visible.map((record) => record.record_type)).size > 1);
</script>

{#if visible.length}
  <ol class="grid gap-2">
    {#each visible as record, index (`${record.test_execution_id}:${record.record_type}:${record.step_id}:${record.order}:${record.started_at}`)}
      {@const duration = record.status === "pending" || record.status === "running" ? null : formatDuration(record.duration_s)}
      <li class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 border-l-2 border-border/70 py-2 pr-1 pl-3 text-sm">
        <span class="grid size-7 shrink-0 place-items-center rounded-lg border bg-muted/40 text-xs font-semibold tabular-nums text-muted-foreground">
          {String(index + 1).padStart(orderWidth, "0")}
        </span>
        <div class="min-w-0">
          <span class="block min-w-0 font-medium leading-tight [overflow-wrap:break-word]">
            {stepRecordLabel(record, showRecordType)}
          </span>
          {#if duration}
            <span class="mt-1 block text-xs leading-tight text-muted-foreground">{duration}</span>
          {/if}
          {#if record.detail}
            <ErrorCard failure={stepFailure(record.detail)} />
          {/if}
        </div>
        <Badge class="justify-self-end gap-1 text-xs capitalize" variant={stepBadgeVariant(record.status)}>
          {#if record.status === "running"}
            <LoaderCircle aria-hidden="true" class="animate-spin" size={10} />
          {/if}
          {record.status}
        </Badge>
      </li>
    {/each}
  </ol>
{:else}
  <p class="border-l-2 border-dashed border-border/70 pl-3 text-sm text-muted-foreground">{emptyMessage}</p>
{/if}
