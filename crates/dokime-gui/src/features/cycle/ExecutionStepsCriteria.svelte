<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { sortedStepRecords } from "../../components/document/step-records";
  import type { StepExecutionRecord } from "../../lib/types";
  import ExecutionStep from "./ExecutionStep.svelte";

  type Props =
    | { layout?: "execution"; steps: StepExecutionRecord[]; criteria: StepExecutionRecord[]; live?: boolean }
    | { layout: "document"; records: StepExecutionRecord[]; emptyMessage: string };
  let props: Props = $props();
</script>

{#snippet recordList(records: StepExecutionRecord[], live: boolean, empty: string, documentLayout = false)}
  {@const ordered = sortedStepRecords(records)}
  {@const ordinalWidth = Math.max(2, String(ordered.length).length)}
  {@const showRecordType = documentLayout && new Set(ordered.map((record) => record.record_type)).size > 1}
  {#if ordered.length}
    <ol class={documentLayout ? "grid gap-2" : "grid min-w-0 grid-cols-(--layout-step-record) gap-x-3 gap-y-1"}>
      {#each ordered as record, index (`${record.test_execution_id}:${record.record_type}:${record.step_id}:${record.order}:${record.started_at}`)}
        <ExecutionStep {record} {live} {documentLayout} {showRecordType} ordinal={String(index + 1).padStart(ordinalWidth, "0")} />
      {/each}
    </ol>
  {:else}
    <p class={["text-sm text-muted-foreground", documentLayout && "border-l-2 border-dashed border-border/70 pl-3"]}>{empty}</p>
  {/if}
{/snippet}

{#snippet recordColumn(title: string, records: StepExecutionRecord[], live: boolean, empty: string)}
  <section aria-label={title} class="grid min-w-0 content-start gap-2">
    <h3
      class="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 border-b pb-2 text-xs font-semibold text-muted-foreground uppercase"
    >
      <span>{title}</span>
      <span class="tabular-nums">{records.length}</span>
    </h3>
    {@render recordList(records, live, empty)}
  </section>
{/snippet}

{#if props.layout === "document"}
  {@render recordList(props.records, false, props.emptyMessage, true)}
{:else}
  <div class="grid min-w-0 gap-x-8 gap-y-4 lg:grid-cols-2">
    {@render recordColumn("Steps", props.steps, props.live ?? true, "No steps recorded yet.")}
    {@render recordColumn("Criteria", props.criteria, props.live ?? true, "No criteria judged yet.")}
  </div>
{/if}
