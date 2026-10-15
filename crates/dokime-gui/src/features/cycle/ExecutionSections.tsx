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
import {
  RecordListContent,
  TraceabilityDocumentContent,
} from "@/components/document/DocumentSurface";
import type { DocumentBlockChrome } from "@/components/document/DocumentViewRenderer";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { OperatorNotesContent } from "@/components/document/OperatorNotesContent";
import { MarkdownText } from "@/components/MarkdownText";
import { MediaCaptureControls } from "@/components/MediaCaptureControls";
import { MediaGallery } from "@/components/MediaGallery";
import { TraceabilityJsonDialog } from "@/components/traceability/TraceabilityJsonDialog";
import { Button } from "@/components/ui/button";
import type {
  DocumentViewCard,
  HubLiveState,
  PromptRequest,
  RecordScope,
  TestCyclePlan,
  TestExecution,
} from "@/lib/types";
import { type ExecutionMenu, NodeMenu } from "./CycleWorkspaceHeader";
import {
  type CycleExecutionRow,
  type CycleSelection,
  isActiveExecutionStatus,
  isUnstartedExecution,
} from "./cycle-workspace-model";
import {
  DocumentView,
  documentHasWindowedCards,
  documentWindowElapsedUnit,
  FrozenDocumentConsole,
  useDocumentSurface,
} from "./DocumentView";
import { EditableLabel } from "./EditableLabel";
import { ExecutionControl } from "./ExecutionControl";
import { ExecutionParameters } from "./ExecutionParameters";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { ExecutionStepsCriteria } from "./ExecutionStepsCriteria";
import {
  executionDocumentSections,
  executionPagePrompts,
  executionParameterRows,
  executionSectionOf,
} from "./execution-sections-model";
import { PlotWindowControl, usePlotWindow } from "./PlotWindowSlider";
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
  /** Renames the execution; `null` for a case that has not run. */
  onRename: ((label: string) => Promise<unknown>) | null;
  menu: ExecutionMenu;
  /** The cycle's answerable prompts; the page keeps its own and the cycle's. */
  prompts: PromptRequest[];
  selection: CycleSelection;
  onSelectExecution: (executionId: string) => void;
};

/* === Constants === */

const UNSTARTED_NOTICE =
  "This case has not run yet. These are the parameters the next attempt will use.";

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
  const started =
    row.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
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
    traceabilityExportName: plan
      ? { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal }
      : null,
  });

  // Divide The Document Into Sections
  const { shared, view } = surface;
  const mainCards = view.sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
  const present = new Set(executionDocumentSections(mainCards));
  const reportSections = view.sections.map((section) => ({
    ...section,
    cards: section.cards.filter(isReportCard),
  }));
  const template = plan?.templates.find((item) => item.template_id === row.case.templateId);
  const parameterRows = executionParameterRows(
    row.case.testCase?.parameters,
    template?.parameters ?? [],
  );
  const executionId = row.execution?.test_execution_id ?? null;
  const pagePrompts = executionPagePrompts(prompts, executionId);
  const running = execution && isActiveExecutionStatus(execution.status) ? executionId : null;
  const liveScope = live && scope.test_execution_id ? scope : null;
  const documentError = documentQuery.error;

  return (
    <div className="grid min-h-0">
      <div className="min-h-0 overflow-y-auto px-4 py-4">
        <div className="grid min-w-0 gap-4">
          <header className="grid min-h-[var(--bar-control-height)] min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1">
            <EditableLabel
              label={label}
              onRename={live ? onRename : null}
              prefix={titlePrefix}
              subject="Execution"
            />
            {live ? <NodeMenu menu={menu} /> : <span />}
            {template?.description ? (
              <div className="col-span-2 text-sm text-muted-foreground">
                <MarkdownText text={template.description} />
              </div>
            ) : null}
          </header>

          {live ? (
            <ExecutionControl
              cycleId={cycleId}
              executionId={running}
              onSelectExecution={onSelectExecution}
              prompts={pagePrompts}
              selection={selection}
            />
          ) : null}

          <ExecutionSectionCard
            description={started ? null : UNSTARTED_NOTICE}
            empty={parameterRows.length ? null : "This case has no parameters."}
            title="Parameters"
          >
            {parameterRows.length ? <ExecutionParameters rows={parameterRows} /> : null}
          </ExecutionSectionCard>

          {!started ? null : documentError ? (
            <ExecutionSectionCard
              action={
                <Button onClick={() => void documentQuery.refetch()} size="sm" variant="outline">
                  Try Again
                </Button>
              }
              empty={
                documentError instanceof Error
                  ? documentError.message
                  : "The document could not be read from the backend."
              }
              title="Evidence Unavailable"
            />
          ) : documentQuery.isPending && !document ? (
            <ExecutionSectionCard
              empty="Reading the document for this execution."
              title="Loading"
            />
          ) : (
            <>
              {present.has("steps-criteria") ? (
                <ExecutionSectionCard
                  empty={
                    view.stepRecords.length || view.criterionRecords.length
                      ? null
                      : "No steps or criteria recorded yet."
                  }
                  title="Steps & Criteria"
                >
                  {view.stepRecords.length || view.criterionRecords.length ? (
                    <ExecutionStepsCriteria
                      criteria={view.criterionRecords}
                      steps={view.stepRecords}
                    />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("notes") ? (
                <ExecutionSectionCard
                  empty={liveScope || view.notes.length ? null : "No operator notes recorded."}
                  title="Operator Notes"
                >
                  {liveScope || view.notes.length ? (
                    <OperatorNotesContent notes={view.notes} scope={liveScope} />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("media") && (live || shared.mediaArtifacts.length) ? (
                <ExecutionSectionCard
                  empty={shared.mediaArtifacts.length ? null : "No media recorded."}
                  title="Media"
                >
                  {liveScope ? <MediaCaptureControls scope={liveScope} /> : null}
                  {shared.mediaArtifacts.length ? (
                    <MediaGallery artifacts={shared.mediaArtifacts} emptyMessage="" />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("artifacts") && (live || shared.artifacts.length) ? (
                <ExecutionSectionCard
                  empty={shared.artifacts.length ? null : "No artifacts recorded."}
                  title="Artifacts"
                >
                  {shared.artifacts.length ? (
                    <ArtifactTree artifacts={shared.artifacts} emptyMessage="" />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("report") || live ? (
                <ExecutionSectionCard
                  action={
                    documentHasWindowedCards(reportSections) ? (
                      <PlotWindowControl
                        elapsedUnit={documentWindowElapsedUnit(reportSections)}
                        live={live}
                        window={plotWindow}
                      />
                    ) : null
                  }
                  empty={
                    present.has("report")
                      ? null
                      : "This execution's document authors no report content."
                  }
                  title="Report"
                >
                  {present.has("report") ? (
                    <DocumentView
                      cardChrome={reportCardChrome}
                      cardFilter={isReportCard}
                      surface={surface}
                    />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("records") ? (
                <ExecutionSectionCard
                  empty={view.records.length ? null : "No records yet."}
                  title="Records"
                >
                  {view.records.length ? (
                    <RecordListContent live={live} records={view.records} />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("traceability") ? (
                <ExecutionSectionCard
                  action={
                    <TraceabilityJsonDialog
                      exportName={shared.traceabilityExportName}
                      traceability={view.traceability}
                    />
                  }
                  empty={
                    view.traceability?.groups.length || view.traceability?.warnings.length
                      ? null
                      : "No traceability information available."
                  }
                  title="Traceability"
                >
                  {view.traceability?.groups.length || view.traceability?.warnings.length ? (
                    <TraceabilityDocumentContent traceability={view.traceability} />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              <FrozenDocumentConsole surface={surface} />
            </>
          )}
        </div>
      </div>
    </div>
  );
}

/* === Private Functions === */

/** Report holds the main-region cards no built-in section claims. */
function isReportCard(card: DocumentViewCard): boolean {
  return isDocumentSurfaceCard(card) && executionSectionOf(card.kind) === "report";
}

/** Inside Report a card keeps its title but not a second frame; plots stay flat. */
function reportCardChrome(card: DocumentViewCard): DocumentBlockChrome {
  return card.kind === "plots" || card.kind === "histograms" || card.kind === "box_summaries"
    ? "flat"
    : "embedded";
}
