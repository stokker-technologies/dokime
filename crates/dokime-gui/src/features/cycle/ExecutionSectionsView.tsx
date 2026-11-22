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
 * The shared `NodeSummary` leads — breadcrumb back to the cycle, "Execution 1 ·
 * <case>" with its rename pencil inline, the status pill and step progress —
 * as it leads the cycle viewer. Below it, every section is the same
 * `ExecutionSectionCard`: Control (state line and lifecycle actions), Prompt
 * (the waiting prompt, directly under Control), Parameters (the materialized
 * case), Steps & Criteria, Operator Notes, Media, Artifacts, Report (whatever
 * else the document authored), Records and Traceability.
 * `execution-sections-model` decides which card kind belongs where; the cards'
 * data comes from the same `useDocumentSurface` reads the cycle document uses,
 * and Report is that same document view filtered to the authored kinds.
 *
 * Everything scrolls as one region, so a long prompt form pushes the page down
 * instead of being clipped by a fixed band. A case that has not run yet shows
 * Control and Parameters only: it has no document.
 *
 * With `live={false}` — the frozen report — Control and Prompt are gone, a Media or
 * Artifacts section that is empty is left out because a file can never fill
 * it, and the console the report prints follows the sections.
 */

/* === External Imports === */

/* === Local Imports === */

import { DocumentRetryContext } from "../../components/document/DocumentWorkspacePresentation";
import { ErrorCard } from "@/components/ErrorCard";
import type { ComponentType, ReactNode } from "react";
import ArtifactTreeView from "../../components/ArtifactTreeView.svelte";
import DocumentTraceability from "../../components/document/DocumentTraceability.svelte";
import type { DocumentBlockChrome } from "../../components/document/DocumentViewRenderer.svelte";
import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
import OperatorNotes from "../../components/document/OperatorNotes.svelte";
import { SvelteMount } from "../../components/SvelteMount";
import MediaGalleryView from "../../components/MediaGalleryView.svelte";
import { TraceabilityJsonDialog } from "../../components/traceability/TraceabilityJsonDialog";
import type { DocumentViewCard, TestCyclePlan, TestExecution } from "@/lib/types";
import { PageShell } from "../../shell/PageShell";
import { type CycleExecutionRow, cycleRowStatus } from "./cycle-workspace-model";
import {
  DocumentViewPresentation,
  type DocumentViewProps,
  documentHasWindowedCards,
  documentWindowElapsedUnit,
  FrozenDocumentConsole,
} from "./DocumentViewPresentation";
import { BUSY_NOTICE, type evidencePaneState } from "./document-query-model";
import type { DocumentSurface } from "./document-view-model";
import { ExecutionParameters } from "./ExecutionParameters";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { ExecutionStepsCriteria } from "./ExecutionStepsCriteria";
import { executionDocumentSections, executionParameterRows, executionSectionOf } from "./execution-sections-model";
import { NodeSummary } from "./NodeSummary";
import { PlotWindowControl } from "./PlotWindowControl";
import { type usePlotWindow } from "./use-plot-window";
import { PromptHistory } from "./PromptHistory";
import { RecordsSection } from "./RecordsSection";

/* === Types === */

export type ExecutionSectionsViewProps = {
  plan: TestCyclePlan | null;
  row: CycleExecutionRow;
  live: boolean;
  titlePrefix: string;
  label: string;
  onRename: ((label: string) => Promise<unknown>) | null;
  /** Returns to the cycle viewer from the breadcrumb. */
  /** The page bar's breadcrumb, `Cycle › Execution 2`. */
  trail: string[];
  /** The page bar's action cluster; none in a file. */
  pageActions?: ReactNode;
  started: TestExecution | null;
  surface: DocumentSurface;
  pane: ReturnType<typeof evidencePaneState>;
  plotWindow: ReturnType<typeof usePlotWindow>;
  onRetry?: (cardId?: string) => void;
  control?: ReactNode;
  /** The execution's answerable prompts, queued directly under Control; none in a file. */
  promptQueue?: ReactNode;
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
  trail,
  pageActions,
  started,
  surface,
  pane,
  plotWindow,
  onRetry,
  control,
  promptQueue,
  notesContent,
  mediaContent,
  artifactsContent,
  documentView: DocumentContent = DocumentViewPresentation,
}: ExecutionSectionsViewProps) {
  // Divide the Document into Sections
  const { shared, view } = surface;
  const mainCards = view.sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
  const present = new Set(executionDocumentSections(mainCards));
  if (view.sections.some((section) => section.loading || section.unavailable)) {
    present.add("report");
  }
  const reportSections = view.sections.map((section) => ({
    ...section,
    cards: section.cards.filter(isReportCard),
  }));
  const template = plan?.templates.find((item) => item.template_id === row.case.templateId);
  const parameterRows = executionParameterRows(row.case.testCase?.parameters, template?.parameters ?? []);
  return (
    <div className="grid min-h-0">
      <PageShell actions={pageActions} className="min-h-0 overflow-y-auto" trail={[...trail.slice(0, -1), `${titlePrefix ?? ""}${label}`]}>
        <NodeSummary
          description={template?.description}
          markdown
          live={live}
          onRename={onRename}
          prefix={titlePrefix}
          executionProgress={{ execution: started, steps: view.stepRecords }}
          status={cycleRowStatus({ ...row, execution: started ?? row.execution })}
          subject="Execution"
          title={label}
        />

        {control}
        {promptQueue ?? <PromptHistory records={view.records} />}

        <ExecutionSectionCard
          description={started ? null : UNSTARTED_NOTICE}
          empty={parameterRows.length ? null : "This case has no parameters."}
          title="Parameters"
        >
          {parameterRows.length ? <ExecutionParameters rows={parameterRows} /> : null}
        </ExecutionSectionCard>

        {!started ? null : pane.kind === "failed" ? (
          <ErrorCard failure={pane.failure} onRetry={onRetry} />
        ) : pane.kind === "busy" ? (
          <ExecutionSectionCard empty={BUSY_NOTICE} title="Loading" />
        ) : pane.kind === "loading" ? (
          <ExecutionSectionCard empty="Reading the document for this execution." title="Loading" />
        ) : (
          <>
            {pane.failure ? <ErrorCard failure={pane.failure} onRetry={onRetry} /> : null}
            {present.has("steps-criteria") ? (
              <ExecutionSectionCard
                empty={view.stepRecords.length || view.criterionRecords.length ? null : "No steps or criteria recorded yet."}
                title="Steps & Criteria"
              >
                {view.stepRecords.length || view.criterionRecords.length ? (
                  <ExecutionStepsCriteria criteria={view.criterionRecords} live={live} steps={view.stepRecords} />
                ) : null}
              </ExecutionSectionCard>
            ) : null}

            {present.has("notes") ? (
              <ExecutionSectionCard empty={notesContent || view.notes.length ? null : "No operator notes recorded."} title="Operator Notes">
                {notesContent || view.notes.length
                  ? (notesContent ?? <SvelteMount component={OperatorNotes} props={{ notes: view.notes }} />)
                  : null}
              </ExecutionSectionCard>
            ) : null}

            {present.has("media") && (live || shared.mediaArtifacts.length) ? (
              <ExecutionSectionCard empty={shared.mediaArtifacts.length ? null : "No media recorded."} title="Media">
                {mediaContent ??
                  (shared.mediaArtifacts.length ? (
                    <SvelteMount component={MediaGalleryView} props={{ artifacts: shared.mediaArtifacts, emptyMessage: "" }} />
                  ) : null)}
              </ExecutionSectionCard>
            ) : null}

            {present.has("artifacts") && (live || shared.artifacts.length) ? (
              <ExecutionSectionCard empty={shared.artifacts.length ? null : "No artifacts recorded."} title="Artifacts">
                {shared.artifacts.length
                  ? (artifactsContent ?? (
                      <SvelteMount component={ArtifactTreeView} props={{ artifacts: shared.artifacts, emptyMessage: "" }} />
                    ))
                  : null}
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
                  <DocumentRetryContext.Provider value={onRetry}>
                    <DocumentContent cardChrome={reportCardChrome} cardFilter={isReportCard} surface={surface} />
                  </DocumentRetryContext.Provider>
                ) : null}
              </ExecutionSectionCard>
            ) : null}

            <RecordsSection live={live} records={view.records} />

            {present.has("traceability") ? (
              <ExecutionSectionCard
                action={<TraceabilityJsonDialog exportName={shared.traceabilityExportName} traceability={view.traceability} />}
                empty={
                  view.traceability?.groups.length || view.traceability?.warnings.length ? null : "No traceability information available."
                }
                title="Traceability"
              >
                {view.traceability?.groups.length || view.traceability?.warnings.length ? (
                  <SvelteMount component={DocumentTraceability} props={{ traceability: view.traceability }} />
                ) : null}
              </ExecutionSectionCard>
            ) : null}

            <FrozenDocumentConsole surface={surface} />
          </>
        )}
      </PageShell>
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
