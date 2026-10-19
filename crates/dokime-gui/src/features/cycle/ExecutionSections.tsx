/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The execution page: one execution's run and evidence as one column of
 * sections in a fixed order.
 *
 * The title — "Execution 1 · <case>" with a pencil that renames the case label
 * and the execution's "…" menu — is the plain header of the column, not a card. Below it, every section is the same
 * `ExecutionSectionCard`: Control (prompts, progress, run controls), Parameters
 * (the materialized case), Steps & Criteria, Operator Notes, Media, Artifacts,
 * Report (whatever else the document authored), Records and Traceability.
 * `execution-sections-model` decides which card kind belongs where; the cards'
 * data comes from the same `useDocumentSurface` reads the cycle document uses,
 * and Report is that same document view filtered to the authored kinds.
 *
 * Everything scrolls as one region, so a long prompt form pushes the page down
 * instead of being clipped by a fixed band. A case that has not run yet shows
 * Control and Parameters only: it has no document.
 *
 * With `live={false}` — the frozen report — Control is gone, a Media or
 * Artifacts section that is empty is left out because a file can never fill
 * it, and the console the report prints follows the sections.
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { ArtifactTree } from "@/components/ArtifactTree";
import { OperatorNotesContent } from "@/components/document/OperatorNotesContent";
import { MediaCaptureControls } from "@/components/MediaCaptureControls";
import { MediaGallery } from "@/components/MediaGallery";
import type { HubLiveState, PromptRequest, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import type { ExecutionMenu } from "./CycleWorkspaceHeader";
import { type CycleExecutionRow, type CycleSelection, isActiveExecutionStatus, isUnstartedExecution } from "./cycle-workspace-model";
import { DocumentView, useDocumentSurface } from "./DocumentView";
import { evidencePaneState } from "./document-query-model";
import { ExecutionControl } from "./ExecutionControl";
import { ExecutionSectionsView } from "./ExecutionSectionsView";
import { executionPagePrompts } from "./execution-sections-model";
import { usePlotWindow } from "./PlotWindowSlider";
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
  /** The cycle's answerable prompts; the page keeps its own and the cycle's. */
  prompts: PromptRequest[];
  selection: CycleSelection;
  onSelectExecution: (executionId: string) => void;
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
  selection,
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
  const running = execution && isActiveExecutionStatus(execution.status) ? executionId : null;
  const liveScope = live && scope.test_execution_id ? scope : null;
  const pane = evidencePaneState({
    hasDocument: document !== null,
    error: documentQuery.error,
    failureReason: documentQuery.failureReason,
    pending: documentQuery.isPending,
  });
  return (
    <ExecutionSectionsView
      plan={plan}
      row={row}
      live={live}
      titlePrefix={titlePrefix}
      label={label}
      onRename={onRename}
      menu={menu}
      started={started}
      surface={surface}
      pane={pane}
      plotWindow={plotWindow}
      onRetry={() => void documentQuery.refetch()}
      documentView={DocumentView}
      control={
        live ? (
          <ExecutionControl
            cycleId={cycleId}
            executionId={running}
            onSelectExecution={onSelectExecution}
            prompts={pagePrompts}
            selection={selection}
          />
        ) : null
      }
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
