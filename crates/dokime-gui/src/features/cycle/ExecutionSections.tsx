/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The execution viewer: one execution's run and evidence as one column of
 * sections in a fixed order, composed from the blocks the cycle viewer uses.
 *
 * This live wrapper owns the reads and the commands; `ExecutionSectionsView`
 * lays the column out — the shared header, Control, Prompt, Parameters and the
 * document's sections down to Records — and the frozen report drives that
 * same view from captured data. The page bar carries the shared action
 * cluster: the primary lifecycle action, which is the Control card's own first
 * action, then Export Evidence, Download Report, expand and the "…" menu. A
 * case that has not run yet shows Control and Parameters only: it has no
 * document.
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { ArtifactTree } from "@/components/ArtifactTree";
import { OperatorNotesContent } from "@/components/document/OperatorNotesContent";
import { MediaCaptureControls } from "@/components/MediaCaptureControls";
import { MediaGallery } from "@/components/MediaGallery";
import type { HubLiveState, PromptRequest, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import { type CycleExecutionRow, isUnstartedExecution } from "./cycle-workspace-model";
import { DocumentView, useDocumentSurface } from "./DocumentView";
import { evidencePaneState } from "./document-query-model";
import { ExecutionActionButton, type ExecutionActions, ExecutionControl, useExecutionActions } from "./ExecutionControl";
import { ExecutionSectionsView } from "./ExecutionSectionsView";
import { executionPagePrompts } from "./execution-sections-model";
import { NodeActionCluster } from "./NodeActionCluster";
import type { ExecutionMenu } from "./NodeMenu";
import { usePlotWindow } from "./PlotWindowSlider";
import { PromptQueue } from "./PromptQueue";
import { useExecutionDetailQuery, useExecutionDocumentQuery } from "./server-state";

/* === Types === */

export type ExecutionSectionsProps = {
  cycleId: string;
  plan: TestCyclePlan | null;
  row: CycleExecutionRow;
  snapshot: HubLiveState | null;
  /** False in the frozen report: no Control, no menu, no live reads. */
  live: boolean;
  /** Names the execution ahead of its label, "Execution 1 · ". */
  titlePrefix: string;
  /** The execution's label: its case's, which the pencil renames. */
  label: string;
  /** Renames the execution; `null` for a case that has not run or has sealed. */
  onRename: ((label: string) => Promise<unknown>) | null;
  menu: ExecutionMenu;
  /** The cycle's answerable prompts; the page answers only its own. */
  prompts: PromptRequest[];
  onSelectExecution: (executionId: string) => void;
  /** Returns to the cycle viewer. */
};

/* === Components === */

export function ExecutionSections({
  cycleId,
  plan,
  row,
  snapshot,
  live,
  titlePrefix,
  label,
  onRename,
  menu,
  prompts,
  onSelectExecution,
}: ExecutionSectionsProps) {
  const plotWindow = usePlotWindow();
  const started = row.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
  const scope = useMemo<RecordScope>(
    () =>
      started
        ? {
            test_cycle_id: cycleId,
            test_execution_id: started.test_execution_id,
            test_case_id: started.case_id,
          }
        : { test_cycle_id: cycleId, test_case_id: row.case.caseId },
    [started, cycleId, row.case.caseId],
  );

  // Read The Execution And Its Document
  const detailQuery = useExecutionDetailQuery(started?.test_execution_id ?? null, Boolean(started));
  const execution: TestExecution | null = detailQuery.data ?? started ?? null;
  const documentQuery = useExecutionDocumentQuery({
    cycleId: started ? cycleId : null,
    executionId: started?.test_execution_id ?? null,
  });
  const document = started ? (documentQuery.data ?? null) : null;
  const surface = useDocumentSurface({
    document,
    execution,
    live,
    snapshot,
    scope,
    parameters: { case: row.case.testCase?.parameters, execution: execution?.parameters },
    noteElapsedStart: started?.started_at ?? plan?.test_cycle.started_at ?? null,
    plotWindow: plotWindow.option,
    plotWindowOverridden: plotWindow.overridden,
    traceabilityExportName: plan ? { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal } : null,
  });

  const executionId = row.execution?.test_execution_id ?? null;
  const pagePrompts = executionPagePrompts(prompts, executionId);
  const liveScope = live && scope.test_execution_id ? scope : null;
  const pane = evidencePaneState({
    hasDocument: document !== null,
    error: documentQuery.error,
    failureReason: documentQuery.failureReason,
    pending: documentQuery.isPending,
  });
  const commanded = live ? (execution ?? row.execution) : null;
  const actions = useExecutionActions({
    cycleActive: plan?.test_cycle.phase === "active",
    cycleId,
    execution: commanded,
    onSelectExecution,
    prompts: pagePrompts,
    row,
  });
  const reportSubject = useMemo(
    () => (executionId ? { executionId, label: `${titlePrefix}${label}`, sealed: commanded?.sealed === true } : null),
    [commanded?.sealed, executionId, label, titlePrefix],
  );
  const primary = actions ? primaryAction(actions) : null;
  /* Republished as the decision changes: the cluster's primary action is the
     card's own, so the two can never disagree. */
  const pageActions = live ? (
    <NodeActionCluster
      cycleId={cycleId}
      menu={menu}
      primary={primary && actions ? <ExecutionActionButton action={primary} actions={actions} /> : null}
      subject={reportSubject}
    />
  ) : null;

  return (
    <ExecutionSectionsView
      plan={plan}
      row={row}
      live={live}
      titlePrefix={titlePrefix}
      label={label}
      onRename={onRename}
      pageActions={pageActions}
      trail={["Cycle", `Execution ${row.ordinal}`]}
      started={started}
      surface={surface}
      pane={pane}
      plotWindow={plotWindow}
      onRetry={() => void documentQuery.refetch()}
      documentView={DocumentView}
      control={actions ? <ExecutionControl actions={actions} ordinal={row.ordinal} /> : null}
      promptQueue={live ? <PromptQueue prompts={pagePrompts} records={surface.view.records} steps={surface.view.stepRecords} /> : undefined}
      notesContent={live ? <OperatorNotesContent notes={surface.view.notes} scope={liveScope} /> : undefined}
      mediaContent={
        live ? (
          <>
            {liveScope ? <MediaCaptureControls scope={liveScope} /> : null}
            {surface.shared.mediaArtifacts.length ? <MediaGallery artifacts={surface.shared.mediaArtifacts} emptyMessage="" /> : null}
          </>
        ) : undefined
      }
      artifactsContent={live ? <ArtifactTree artifacts={surface.shared.artifacts} emptyMessage="" /> : undefined}
    />
  );
}

/* === Private Functions === */

/** The bar's primary action: the card's first, unless it is destructive. */
function primaryAction(actions: ExecutionActions) {
  const first = actions.state.actions[0];
  return first && first !== "Stop" && first !== "Cancel" ? first : null;
}
