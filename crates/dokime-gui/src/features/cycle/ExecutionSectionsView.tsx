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

/* === Local Imports === */

import { ArtifactTreeView } from "@/components/ArtifactTreeView";
import { RecordListContent, TraceabilityDocumentContent } from "@/components/document/DocumentSurfaceView";
import type { DocumentBlockChrome } from "@/components/document/DocumentViewRenderer";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { FrozenOperatorNotes } from "@/components/document/OperatorNotesView";
import { MarkdownText } from "@/components/MarkdownText";
import { MediaGalleryView } from "@/components/MediaGalleryView";
import { TraceabilityJsonDialog } from "@/components/traceability/TraceabilityJsonDialog";
import { Button } from "@/components/ui/button";
import type { DocumentViewCard, TestCyclePlan, TestExecution } from "@/lib/types";
import { type ExecutionMenu, NodeMenu } from "./CycleWorkspaceHeader";
import type { CycleExecutionRow } from "./cycle-workspace-model";
import type { ComponentType, ReactNode } from "react";
import type { DocumentSurface } from "./document-view-model";
import { DocumentViewPresentation, type DocumentViewProps } from "./DocumentViewPresentation";
import { documentHasWindowedCards, documentWindowElapsedUnit, FrozenDocumentConsole } from "./DocumentViewPresentation";
import { BUSY_NOTICE, evidencePaneState } from "./document-query-model";
import { EditableLabel } from "./EditableLabel";
import { ExecutionParameters } from "./ExecutionParameters";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { ExecutionStepsCriteria } from "./ExecutionStepsCriteria";
import { executionDocumentSections, executionParameterRows, executionSectionOf } from "./execution-sections-model";
import { PlotWindowControl, usePlotWindow } from "./PlotWindowSlider";

/* === Types === */

export type ExecutionSectionsViewProps = {
  plan: TestCyclePlan | null;
  row: CycleExecutionRow;
  live: boolean;
  titlePrefix: string;
  label: string;
  onRename: ((label: string) => Promise<unknown>) | null;
  menu: ExecutionMenu | null;
  started: TestExecution | null;
  surface: DocumentSurface;
  pane: ReturnType<typeof evidencePaneState>;
  plotWindow: ReturnType<typeof usePlotWindow>;
  onRetry?: () => void;
  control?: ReactNode;
  notesContent?: ReactNode;
  mediaContent?: ReactNode;
  artifactsContent?: ReactNode;
  documentView?: ComponentType<DocumentViewProps>;
};

/* === Constants === */

const UNSTARTED_NOTICE = "This case has not run yet. These are the parameters the next attempt will use.";

/* === Components === */

export function ExecutionSectionsView({
  plan,
  row,
  live,
  titlePrefix,
  label,
  onRename,
  menu,
  started,
  surface,
  pane,
  plotWindow,
  onRetry,
  control,
  notesContent,
  mediaContent,
  artifactsContent,
  documentView: DocumentContent = DocumentViewPresentation,
}: ExecutionSectionsViewProps) {
  // Divide The Document Into Sections
  const { shared, view } = surface;
  const mainCards = view.sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
  const present = new Set(executionDocumentSections(mainCards));
  const reportSections = view.sections.map((section) => ({
    ...section,
    cards: section.cards.filter(isReportCard),
  }));
  const template = plan?.templates.find((item) => item.template_id === row.case.templateId);
  const parameterRows = executionParameterRows(row.case.testCase?.parameters, template?.parameters ?? []);
  return (
    <div className="grid min-h-0">
      <div className="min-h-0 overflow-y-auto px-4 py-4">
        <div className="grid min-w-0 gap-4">
          <header className="grid min-h-[var(--bar-control-height)] min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1">
            <EditableLabel label={label} onRename={live ? onRename : null} prefix={titlePrefix} subject="Execution" />
            {live && menu ? <NodeMenu menu={menu} /> : <span />}
            {template?.description ? (
              <div className="col-span-2 text-sm text-muted-foreground">
                <MarkdownText text={template.description} />
              </div>
            ) : null}
          </header>

          {control}

          <ExecutionSectionCard
            description={started ? null : UNSTARTED_NOTICE}
            empty={parameterRows.length ? null : "This case has no parameters."}
            title="Parameters"
          >
            {parameterRows.length ? <ExecutionParameters rows={parameterRows} /> : null}
          </ExecutionSectionCard>

          {!started ? null : pane.kind === "failed" ? (
            <ExecutionSectionCard
              action={
                <Button onClick={onRetry} size="sm" variant="outline">
                  Try Again
                </Button>
              }
              empty={pane.message}
              title="Evidence Unavailable"
            />
          ) : pane.kind === "busy" ? (
            <ExecutionSectionCard empty={BUSY_NOTICE} title="Loading" />
          ) : pane.kind === "loading" ? (
            <ExecutionSectionCard empty="Reading the document for this execution." title="Loading" />
          ) : (
            <>
              {pane.staleReason ? (
                <p className="text-xs text-muted-foreground">Showing the last evidence read. {pane.staleReason}</p>
              ) : null}
              {present.has("steps-criteria") ? (
                <ExecutionSectionCard
                  empty={view.stepRecords.length || view.criterionRecords.length ? null : "No steps or criteria recorded yet."}
                  title="Steps & Criteria"
                >
                  {view.stepRecords.length || view.criterionRecords.length ? (
                    <ExecutionStepsCriteria criteria={view.criterionRecords} steps={view.stepRecords} />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("notes") ? (
                <ExecutionSectionCard
                  empty={notesContent || view.notes.length ? null : "No operator notes recorded."}
                  title="Operator Notes"
                >
                  {notesContent || view.notes.length ? (notesContent ?? <FrozenOperatorNotes notes={view.notes} />) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("media") && (live || shared.mediaArtifacts.length) ? (
                <ExecutionSectionCard empty={shared.mediaArtifacts.length ? null : "No media recorded."} title="Media">
                  {mediaContent ??
                    (shared.mediaArtifacts.length ? <MediaGalleryView artifacts={shared.mediaArtifacts} emptyMessage="" /> : null)}
                </ExecutionSectionCard>
              ) : null}

              {present.has("artifacts") && (live || shared.artifacts.length) ? (
                <ExecutionSectionCard empty={shared.artifacts.length ? null : "No artifacts recorded."} title="Artifacts">
                  {shared.artifacts.length ? (artifactsContent ?? <ArtifactTreeView artifacts={shared.artifacts} emptyMessage="" />) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("report") || live ? (
                <ExecutionSectionCard
                  action={
                    documentHasWindowedCards(reportSections) ? (
                      <PlotWindowControl elapsedUnit={documentWindowElapsedUnit(reportSections)} live={live} window={plotWindow} />
                    ) : null
                  }
                  empty={present.has("report") ? null : "This execution's document authors no report content."}
                  title="Report"
                >
                  {present.has("report") ? (
                    <DocumentContent cardChrome={reportCardChrome} cardFilter={isReportCard} surface={surface} />
                  ) : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("records") ? (
                <ExecutionSectionCard empty={view.records.length ? null : "No records yet."} title="Records">
                  {view.records.length ? <RecordListContent live={live} records={view.records} /> : null}
                </ExecutionSectionCard>
              ) : null}

              {present.has("traceability") ? (
                <ExecutionSectionCard
                  action={<TraceabilityJsonDialog exportName={shared.traceabilityExportName} traceability={view.traceability} />}
                  empty={
                    view.traceability?.groups.length || view.traceability?.warnings.length ? null : "No traceability information available."
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
  return card.kind === "plots" ? "flat" : "embedded";
}
