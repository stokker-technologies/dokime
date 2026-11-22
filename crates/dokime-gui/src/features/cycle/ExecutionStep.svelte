<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { LoaderCircle } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { stepBadgeVariant, stepFailure, stepRecordLabel } from "../../components/document/step-records";
  import Badge from "../../components/ui/Badge.svelte";
  import type { StepExecutionRecord } from "../../lib/types";
  import { LIST_STATUS_PILL_CLASS } from "./cycle-workspace-model";
  import { durationText, observeElapsedSeconds } from "./elapsed";

  type Props = { record: StepExecutionRecord; live: boolean; ordinal: string };
  let { record, live, ordinal }: Props = $props();
  let elapsed = $state<number | null>(null);
  const running = $derived(record.status === "running");
  const startedAt = $derived(record.started_at);
  const ticking = $derived(live && running && record.record_type === "step");
  const duration = $derived(record.record_type === "step" ? durationText(record, live, elapsed) : "");
  const message = $derived(
    record.record_type === "criterion" && typeof record.result.details.message === "string" ? record.result.details.message : null,
  );

  $effect(() =>
    observeElapsedSeconds(startedAt, ticking, (seconds) => {
      elapsed = seconds;
    }),
  );
</script>

<li class="col-span-4 grid min-w-0 grid-cols-subgrid items-baseline py-1 text-sm leading-5">
  <span class="text-xs text-muted-foreground tabular-nums">{ordinal}</span>
  <div class="min-w-0">
    <span class="block font-medium [overflow-wrap:break-word]">{stepRecordLabel(record, false)}</span>
    {#if message}
      <span class="block text-xs text-muted-foreground [overflow-wrap:anywhere]">{message}</span>
    {/if}
    {#if record.detail}
      <ErrorCard failure={stepFailure(record.detail)} />
    {/if}
  </div>
  <span class="text-xs text-muted-foreground tabular-nums">{duration}</span>
  <Badge class={`${LIST_STATUS_PILL_CLASS} gap-1 capitalize`} variant={stepBadgeVariant(record.status)}>
    {#if running}
      <LoaderCircle aria-hidden="true" class="animate-spin motion-reduce:animate-none" size={10} />
    {/if}
    {record.status}
  </Badge>
</li>
