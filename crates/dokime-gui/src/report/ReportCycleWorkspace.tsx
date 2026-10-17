/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


/**
 * TODO: Describe this module's responsibility.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { CycleEvidenceView } from "@/features/cycle/CycleEvidenceView";
import { CycleWorkspaceHeader } from "@/features/cycle/CycleWorkspaceHeader";
import { buildCycleCases, buildCycleExecutionRows, cycleStatusLabel, cycleStatusVariant,
  finishedCaseCount, isUnstartedExecution, selectedCycleRow,
  type CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { documentSurface, documentView, type DocumentSurfaceOptions } from "@/features/cycle/document-view-model";
import { evidencePaneState } from "@/features/cycle/document-query-model";
import { ExecutionSectionsView } from "@/features/cycle/ExecutionSectionsView";
import { usePlotWindow } from "@/features/cycle/PlotWindowSlider";
import type { CycleSnapshotSource } from "@/features/cycle/snapshot-source";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type ReportCycleWorkspaceProps = {
  source: CycleSnapshotSource;
  selection: CycleSelection;
};

/* === Classes === */

/* === Components === */

/** Captured data drives the same cycle and execution presentation used live. */
export function ReportCycleWorkspace({ source, selection }: ReportCycleWorkspaceProps) {
  const plotWindow = usePlotWindow();
  const plan = source.plan;
  const cases = buildCycleCases(plan);
  const rows = buildCycleExecutionRows(cases, source.executions);
  const row = selectedCycleRow(rows, selection);
  if (!plan) {
    return <div className="grid justify-items-center gap-3 px-4 py-16 text-center">
      <h1 className="font-heading text-lg font-semibold">Cycle Not Found</h1>
      <p className="max-w-prose text-sm text-muted-foreground">
        This cycle is no longer loaded. A sealed cycle leaves the runtime when the next one starts; History keeps it.
      </p>
    </div>;
  }
  const started = row?.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
  const executionId = started?.test_execution_id ?? null;
  const execution = executionId
    ? source.executions.find((item) => item.test_execution_id === executionId) ?? started
    : null;
  const document = row && !started ? null : source.document(executionId);
  const options: DocumentSurfaceOptions = {
    document, execution, live: false, snapshot: source.runtimeShell,
    scope: started ? {
      test_cycle_id: plan.test_cycle.test_cycle_id,
      test_execution_id: started.test_execution_id,
      test_case_id: started.case_id,
    } : row ? {
      test_cycle_id: plan.test_cycle.test_cycle_id, test_case_id: row.case.caseId,
    } : { test_cycle_id: plan.test_cycle.test_cycle_id },
    parameters: { case: row?.case.testCase?.parameters, execution: execution?.parameters },
    noteElapsedStart: started?.started_at ?? plan.test_cycle.started_at ?? null,
    plotWindow: plotWindow.option,
    plotWindowOverridden: plotWindow.overridden,
    traceabilityExportName: row ? { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal } : null,
  };
  const view = documentView({ ...options, document, liveSnapshot: source.runtimeShell }, {
    artifacts: source.executionFiles(executionId), notes: source.notes, health: source.health,
    records: source.records, traceability: source.traceability(executionId),
    reportPayloads: source.plotPayloads(executionId),
  });
  const surface = documentSurface(options, view, false);
  const pane = evidencePaneState({ hasDocument: document !== null, error: null,
    failureReason: null, pending: false });
  if (row) {
    return <ExecutionSectionsView plan={plan} row={row} live={false}
      titlePrefix={`Execution ${row.ordinal} · `} label={row.case.label} onRename={null}
      menu={null} started={started} surface={surface} pane={pane} plotWindow={plotWindow} />;
  }
  return <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
    <CycleWorkspaceHeader live={false} menu={null} identity={{ title: plan.test_cycle.title,
      status: { label: cycleStatusLabel(plan.test_cycle.status),
        variant: cycleStatusVariant(plan.test_cycle.status),
        progress: `${finishedCaseCount(cases)} of ${cases.length} cases finished`, activity: null } }} />
    <CycleEvidenceView live={false} surface={surface} pane={pane} plotWindow={plotWindow} />
  </div>;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */
