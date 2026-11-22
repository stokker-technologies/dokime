<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { sortedStepRecords } from "../../components/document/step-records";
  import type { StepExecutionRecord } from "../../lib/types";
  import ExecutionStep from "./ExecutionStep.svelte";

  type Props = { steps: StepExecutionRecord[]; criteria: StepExecutionRecord[]; live?: boolean };
  let { steps, criteria, live = true }: Props = $props();
  const orderedSteps = $derived(sortedStepRecords(steps));
  const orderedCriteria = $derived(sortedStepRecords(criteria));
</script>

{#snippet recordColumn(
  title: string,
  records: StepExecutionRecord[],
  empty: string,
)}
  {@const ordinalWidth = Math.max(2, String(records.length).length)}
  <section aria-label={title} class="grid min-w-0 content-start gap-2">
    <h3
      class="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 border-b pb-2 text-xs font-semibold text-muted-foreground uppercase"
    >
      <span>{title}</span>
      <span class="tabular-nums">{records.length}</span>
    </h3>
    {#if records.length}
      <ol class="grid min-w-0 grid-cols-(--layout-step-record) gap-x-3 gap-y-1">
        {#each records as record, index (`${record.test_execution_id}:${record.record_type}:${record.step_id}:${record.order}`)}
          <ExecutionStep {record} {live} ordinal={String(index + 1).padStart(ordinalWidth, "0")} />
        {/each}
      </ol>
    {:else}
      <p class="text-sm text-muted-foreground">{empty}</p>
    {/if}
  </section>
{/snippet}

<div class="grid min-w-0 gap-x-8 gap-y-4 lg:grid-cols-2">
  {@render recordColumn("Steps", orderedSteps, "No steps recorded yet.")}
  {@render recordColumn("Criteria", orderedCriteria, "No criteria judged yet.")}
</div>
