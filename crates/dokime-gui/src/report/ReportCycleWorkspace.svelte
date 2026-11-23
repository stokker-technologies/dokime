<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import CycleEvidenceView from "../features/cycle/CycleEvidenceView.svelte";
  import {
    buildCycleCases,
    buildCycleExecutionRows,
    isUnstartedExecution,
    sealedCaseCount,
    selectedCycleRow,
    type CycleSelection,
  } from "../features/cycle/cycle-workspace-model";
  import { evidencePaneState } from "../features/cycle/document-query-model";
  import { documentSurface, documentView, type DocumentSurfaceOptions } from "../features/cycle/document-view-model";
  import ExecutionSectionsView from "../features/cycle/ExecutionSectionsView.svelte";
  import NodeSummary from "../features/cycle/NodeSummary.svelte";
  import type { CycleSnapshotSource } from "../features/cycle/snapshot-source";
  import { reportOutcome } from "./native-report";

  let { selection, source }: { source: CycleSnapshotSource; selection: CycleSelection } = $props();
  const captured = $derived.by(() => {
    const plan = source.plan;
    if (!plan) {
      return null;
    }
    const cases = buildCycleCases(plan);
    const rows = buildCycleExecutionRows(cases, source.executions);
    const row = selectedCycleRow(rows, selection);
    const started = row?.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
    const executionId = started?.test_execution_id ?? null;
    const execution = executionId ? (source.executions.find((item) => item.test_execution_id === executionId) ?? started) : null;
    const document = row && !started ? null : source.document(executionId);
    const cycleId = plan.test_cycle.test_cycle_id;
    const scope = started ? { cycleId, executionId: started.test_execution_id } : row ? { cycleId, caseId: row.case.caseId } : { cycleId };
    const options: DocumentSurfaceOptions = {
      document,
      execution,
      live: false,
      snapshot: source.runtimeShell,
      scope,
      parameters: { case: row?.case.testCase?.parameters, execution: execution?.parameters },
      noteElapsedStart: started?.started_at ?? plan.test_cycle.started_at ?? null,
      traceabilityExportName: row ? { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal } : null,
    };
    const view = documentView(
      { ...options, document, liveSnapshot: source.runtimeShell },
      {
        artifacts: source.executionFiles(executionId),
        notes: source.notes,
        health: source.health,
        records: source.records,
        traceability: source.traceability(executionId),
        reportPayloads: source.plotPayloads(executionId),
      },
    );
    const surface = documentSurface(options, view, false);
    const pane = evidencePaneState({ hasDocument: document !== null, error: null, failureReason: null, pending: false });
    const progress = `${sealedCaseCount(cases)} of ${cases.length} cases sealed`;
    return { plan, row, started, surface, pane, progress };
  });
</script>

{#if !captured}
  <div class="grid justify-items-center gap-3 px-4 py-16 text-center">
    <h1 class="font-heading text-lg font-semibold">Cycle Not Found</h1>
    <p class="max-w-prose text-sm text-muted-foreground">
      This cycle is no longer loaded. A sealed cycle leaves the runtime when the next one starts; History keeps it.
    </p>
  </div>
{:else}
  {@const { plan, row, started, surface, pane, progress } = captured}
  {#if row}
    <ExecutionSectionsView
      {plan}
      {row}
      live={false}
      titlePrefix={`Execution ${row.ordinal} · `}
      label={row.case.label}
      onRename={null}
      {started}
      {surface}
      {pane}
    />
  {:else}
    <CycleEvidenceView live={false} {surface} {pane}>
      {#snippet lead()}
        <NodeSummary live={false} onRename={null} {progress} status={reportOutcome(source)} subject="Cycle" title={plan.test_cycle.title} />
      {/snippet}
    </CycleEvidenceView>
  {/if}
{/if}
