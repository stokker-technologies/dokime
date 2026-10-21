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
 * The shared `NodeHeader` leads — breadcrumb back to the cycle, "Execution 1 ·
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

import type { ComponentType, ReactNode } from "react";
import { ArtifactTreeView } from "@/components/ArtifactTreeView";
import { TraceabilityDocumentContent } from "@/components/document/DocumentSurfaceView";
import type { DocumentBlockChrome } from "@/components/document/DocumentViewRenderer";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { FrozenOperatorNotes } from "@/components/document/OperatorNotesView";
import { MarkdownText } from "@/components/MarkdownText";
import { MediaGalleryView } from "@/components/MediaGalleryView";
import { TraceabilityJsonDialog } from "@/components/traceability/TraceabilityJsonDialog";
import { Button } from "@/components/ui/button";
import { parseTimestampMs } from "@/lib/time-format";
import type { DocumentViewCard, TestCyclePlan, TestExecution } from "@/lib/types";
import { PageShell } from "@/shell/PageShell";
import { type CycleExecutionRow, cycleRowStatus, isActiveExecutionStatus } from "./cycle-workspace-model";
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
import { NodeHeader } from "./NodeHeader";
import { PlotWindowControl, type usePlotWindow } from "./PlotWindowSlider";
import { PromptHistory } from "./PromptHistory";
import { RecordsSection } from "./RecordsSection";
import { durationText, useElapsedSeconds } from "./use-elapsed";

/* === Types === */

export type ExecutionSectionsViewProps = {
  plan: TestCyclePlan | null;
  row: CycleExecutionRow;
  live: boolean;
  titlePrefix: string;
  label: string;
  onRename: ((label: string) => Promise<unknown>) | null;
  /** Returns to the cycle viewer from the breadcrumb. */
  onSelectCycle?: () => void;
  /** The page bar's breadcrumb, `Cycle › Execution 2`. */
  trail: string[];
  /** The page bar's action cluster; none in a file. */
  pageActions?: ReactNode;
  started: TestExecution | null;
  surface: DocumentSurface;
  pane: ReturnType<typeof evidencePaneState>;
  plotWindow: ReturnType<typeof usePlotWindow>;
  onRetry?: () => void;
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
  onSelectCycle,
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
  const running = started !== null && isActiveExecutionStatus(started.status);
  const elapsed = useElapsedSeconds(started?.started_at, live && running);
  const executionDuration = started
    ? durationText({ duration_s: measuredSeconds(started), status: running ? "running" : started.status }, live, elapsed)
    : "";
  return (
    <div className="grid min-h-0">
      <PageShell actions={pageActions} className="min-h-0 overflow-y-auto" trail={trail}>
        <NodeHeader
          breadcrumb={[{ label: plan?.test_cycle.title ?? "Cycle", onSelect: onSelectCycle }, { label: `Execution ${row.ordinal}` }]}
          description={template?.description ? <MarkdownText text={template.description} /> : null}
          live={live}
          onRename={onRename}
          prefix={titlePrefix}
          progress={[stepProgress(view.stepRecords), executionDuration].filter(Boolean).join(" · ") || null}
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
            {pane.staleReason ? <p className="text-xs text-muted-foreground">Showing the last evidence read. {pane.staleReason}</p> : null}
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
                  <TraceabilityDocumentContent traceability={view.traceability} />
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

/** What the runtime measured: from start to finish, once the execution ended. */
function measuredSeconds(execution: Pick<TestExecution, "started_at" | "finished_at">): number | null {
  const started = parseTimestampMs(execution.started_at);
  const finished = parseTimestampMs(execution.finished_at);
  return started === null || finished === null ? null : Math.max(0, (finished - started) / 1_000);
}

/** The header's progress text: how many of the execution's steps have settled. */
function stepProgress(steps: readonly { status: string }[]): string | null {
  if (!steps.length) return null;
  const settled = steps.filter((step) => step.status !== "pending" && step.status !== "running").length;
  return `${settled} of ${steps.length} steps finished`;
}

/** Report holds the main-region cards no built-in section claims. */
function isReportCard(card: DocumentViewCard): boolean {
  return isDocumentSurfaceCard(card) && executionSectionOf(card.kind) === "report";
}

/** Inside Report a card keeps its title but not a second frame; plots stay flat. */
function reportCardChrome(card: DocumentViewCard): DocumentBlockChrome {
  return card.kind === "plots" ? "flat" : "embedded";
}
