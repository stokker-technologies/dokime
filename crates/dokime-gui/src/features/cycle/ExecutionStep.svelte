<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { LoaderCircle } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { stepBadgeVariant, stepFailure, stepRecordLabel } from "../../components/document/step-records";
  import Badge from "../../components/ui/Badge.svelte";
  import { formatDuration } from "../../lib/format";
  import type { StepExecutionRecord } from "../../lib/types";
  import { LIST_STATUS_PILL_CLASS } from "./cycle-workspace-model";
  import { durationText, observeElapsedSeconds } from "./elapsed";

  type Props = { record: StepExecutionRecord; live: boolean; ordinal: string; documentLayout: boolean; showRecordType: boolean };
  let { record, live, ordinal, documentLayout, showRecordType }: Props = $props();
  let elapsed = $state<number | null>(null);
  const running = $derived(record.status === "running");
  const startedAt = $derived(record.started_at);
  const ticking = $derived(live && running && record.record_type === "step");
  const duration = $derived(
    documentLayout
      ? running || record.status === "pending"
        ? null
        : formatDuration(record.duration_s)
      : record.record_type === "step"
        ? durationText(record, live, elapsed)
        : "",
  );
  const message = $derived(
    record.record_type === "criterion" && typeof record.result.details.message === "string" ? record.result.details.message : null,
  );

  $effect(() =>
    observeElapsedSeconds(startedAt, ticking, (seconds) => {
      elapsed = seconds;
    }),
  );
</script>

<li
  class={documentLayout
    ? "grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 border-l-2 border-border/70 py-2 pr-1 pl-3 text-sm"
    : "col-span-4 grid min-w-0 grid-cols-subgrid items-baseline py-1 text-sm leading-5"}
>
  <span
    class={[
      "text-xs text-muted-foreground tabular-nums",
      documentLayout && "grid size-7 shrink-0 place-items-center rounded-lg border bg-muted/40 font-semibold",
    ]}
    >{ordinal}</span
  >
  <div class="min-w-0">
    <span class={["block font-medium [overflow-wrap:break-word]", documentLayout && "min-w-0 leading-tight"]}>
      {stepRecordLabel(record, showRecordType)}
    </span>
    {#if documentLayout && duration}
      <span class="mt-1 block text-xs leading-tight text-muted-foreground">{duration}</span>
    {:else if !documentLayout && message}
      <span class="block text-xs text-muted-foreground [overflow-wrap:anywhere]">{message}</span>
    {/if}
    {#if record.detail}
      <ErrorCard failure={stepFailure(record.detail)} />
    {/if}
  </div>
  {#if !documentLayout}
    <span class="text-xs text-muted-foreground tabular-nums">{duration}</span>
  {/if}
  <Badge
    class={`${documentLayout ? "justify-self-end text-xs" : LIST_STATUS_PILL_CLASS} gap-1 capitalize`}
    variant={stepBadgeVariant(record.status)}
  >
    {#if running}
      <LoaderCircle aria-hidden="true" class="animate-spin motion-reduce:animate-none" size={10} />
    {/if}
    {record.status}
  </Badge>
</li>
