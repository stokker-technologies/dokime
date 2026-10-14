/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The document region: what the current selection holds.
 *
 * An execution that has run shows its document and captured evidence. One
 * that has not started — planned, or queued and undispatched — and a case
 * without an execution show the planned parameters. The cycle itself, or no
 * selection at all, shows the cycle's own document. The document starts right
 * under the cockpit: the plot window sits in the header of the plots it
 * scales, and Focus is in the page bar. Records, telemetry, steps,
 * criteria, notes, health, artifacts and traceability all arrive through the
 * existing `useDocumentView` and `DocumentWorkspaceView`; nothing is assembled
 * a second time here.
 *
 * With `live={false}` the same selections render from the snapshot the hooks
 * consult: the toolbar states the scope and the plot window as a sentence
 * instead of offering a slider. The execution list owns selection.
 */

/* === External Imports === */

import { useMemo, useState } from "react";

/* === Local Imports === */

import { CaseParameterGrid } from "@/components/CaseParameterGrid";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { HubLiveState, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import { type CycleExecutionRow, isUnstartedExecution } from "./cycle-workspace-model";
import { DocumentView, documentWindowElapsedUnit } from "./DocumentView";
import {
  DEFAULT_PLOT_WINDOW_INDEX,
  PLOT_WINDOW_OPTIONS,
  PlotWindowSlider,
  plotWindowValueLabel,
} from "./PlotWindowSlider";
import {
  useCycleDocumentQuery,
  useExecutionDetailQuery,
  useExecutionDocumentQuery,
} from "./server-state";

/* === Types === */

export type CycleEvidencePaneProps = {
  cycleId: string;
  plan: TestCyclePlan | null;
  row: CycleExecutionRow | null;
  snapshot: HubLiveState | null;
  /** False in the frozen report: no slider, no live plot window. */
  live?: boolean;
};

/* === Components === */

export function CycleEvidencePane({
  cycleId,
  live = true,
  plan,
  row,
  snapshot,
}: CycleEvidencePaneProps) {
  const [plotWindowIndex, setPlotWindowIndex] = useState(DEFAULT_PLOT_WINDOW_INDEX);
  const [plotWindowOverridden, setPlotWindowOverridden] = useState(false);
  const selectedExecution =
    row?.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
  const cycleShown = row === null;
  const scope = useMemo<RecordScope>(
    () =>
      selectedExecution
        ? {
            test_cycle_id: cycleId,
            test_execution_id: selectedExecution.test_execution_id,
            test_case_id: selectedExecution.case_id,
          }
        : row
          ? { test_cycle_id: cycleId, test_case_id: row.case.caseId }
          : { test_cycle_id: cycleId },
    [selectedExecution, cycleId, row],
  );

  const detailQuery = useExecutionDetailQuery(
    selectedExecution?.test_execution_id ?? null,
    Boolean(selectedExecution),
  );
  const execution: TestExecution | null = detailQuery.data ?? selectedExecution ?? null;

  const executionDocumentQuery = useExecutionDocumentQuery({
    cycleId: selectedExecution ? cycleId : null,
    executionId: selectedExecution?.test_execution_id ?? null,
  });
  const cycleDocumentQuery = useCycleDocumentQuery(cycleId, cycleShown);
  const documentQuery = cycleShown ? cycleDocumentQuery : executionDocumentQuery;
  const document = selectedExecution || cycleShown ? (documentQuery.data ?? null) : null;

  const plotWindow =
    PLOT_WINDOW_OPTIONS[plotWindowIndex] ?? PLOT_WINDOW_OPTIONS[DEFAULT_PLOT_WINDOW_INDEX];
  const template =
    plan?.templates.find((item) => item.template_id === row?.case.templateId) ?? null;

  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const documentError = documentQuery.error;
  const windowElapsedUnit = documentWindowElapsedUnit(sections);
  /* The plot window sits in the header of the plots it scales. A frozen plot
     shows the window it was drawn with: saying so is honest, offering a slider
     that changes nothing is not. */
  const plotWindowControl = live ? (
    <div className="w-56 xl:w-80">
      <PlotWindowSlider
        elapsedUnit={windowElapsedUnit}
        index={plotWindowIndex}
        onIndexChange={(index) => {
          setPlotWindowIndex(index);
          setPlotWindowOverridden(true);
        }}
        option={plotWindow}
      />
    </div>
  ) : (
    <span className="text-xs text-muted-foreground">
      {windowElapsedUnit ? "Elapsed window" : "Plot window"}{" "}
      {plotWindowValueLabel(plotWindow, windowElapsedUnit)}
    </span>
  );

  return (
    <div className="grid min-h-0">
      <div className="min-h-0 overflow-y-auto px-4 py-4">
        {documentError ? (
          <PaneNotice
            action={{ label: "Try Again", onClick: () => void documentQuery.refetch() }}
            message={
              documentError instanceof Error
                ? documentError.message
                : "The document could not be read from the backend."
            }
            title="Evidence Unavailable"
          />
        ) : row && !selectedExecution ? (
          <div className="grid gap-4">
            <p className="text-sm text-muted-foreground">
              This case has not run yet. These are the parameters the next attempt will use.
            </p>
            {template?.description ? <MarkdownText text={template.description} /> : null}
            {row.case.testCase ? <CaseParameterGrid testCase={row.case.testCase} /> : null}
          </div>
        ) : documentQuery.isPending && !document ? (
          <PaneNotice message="Reading the document for this selection." title="Loading" />
        ) : (
          <div className="grid gap-6">
            <DocumentView
              document={document}
              execution={execution}
              live={live}
              snapshot={snapshot}
              scope={scope}
              parameters={{
                case: row?.case.testCase?.parameters,
                execution: execution?.parameters,
              }}
              noteElapsedStart={
                selectedExecution?.started_at ?? plan?.test_cycle.started_at ?? null
              }
              plotWindow={plotWindow}
              plotWindowControl={plotWindowControl}
              plotWindowOverridden={plotWindowOverridden}
              traceabilityExportName={
                row && plan
                  ? { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal }
                  : null
              }
            />
          </div>
        )}
      </div>
    </div>
  );
}

/* === Private Components === */

function PaneNotice({
  action,
  message,
  title,
}: {
  action?: { label: string; onClick: () => void };
  message: string;
  title: string;
}) {
  return (
    <div className="grid justify-items-start gap-2 py-8">
      <p className="font-heading text-sm font-semibold">{title}</p>
      <p className="max-w-prose text-sm text-muted-foreground">{message}</p>
      {action ? (
        <Button onClick={action.onClick} size="sm" variant="outline">
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}

/* === Private Functions === */
