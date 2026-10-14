/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The evidence the current selection holds.
 *
 * Execution selection owns the document and captured evidence. A case without
 * an execution shows its planned parameters. Records, telemetry, steps,
 * criteria, notes, health, artifacts and traceability all arrive through the
 * existing `useDocumentView` and `DocumentWorkspaceView`; nothing is assembled
 * a second time here.
 *
 * With `live={false}` the same selections render from the snapshot the hooks
 * consult: the toolbar states the scope and the plot window as a sentence
 * instead of offering a slider. The execution list owns selection.
 */

/* === External Imports === */

import { Maximize2, Minimize2 } from "lucide-react";
import { useMemo, useState } from "react";

/* === Local Imports === */

import { CaseParameterGrid } from "@/components/CaseParameterGrid";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { HubLiveState, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import { type CycleExecutionRow, isPlannedExecution } from "./cycle-workspace-model";
import { DocumentView, documentHasWindowedCards, documentWindowElapsedUnit } from "./DocumentView";
import {
  DEFAULT_PLOT_WINDOW_INDEX,
  PLOT_WINDOW_OPTIONS,
  PlotWindowSlider,
  plotWindowValueLabel,
} from "./PlotWindowSlider";
import { useExecutionDetailQuery, useExecutionDocumentQuery } from "./server-state";

/* === Types === */

export type CycleEvidencePaneProps = {
  cycleId: string;
  plan: TestCyclePlan | null;
  row: CycleExecutionRow | null;
  snapshot: HubLiveState | null;
  /** True while the shell has given the evidence the whole window. */
  focused: boolean;
  /** False in the frozen report: no slider, no focus, no live plot window. */
  live?: boolean;
  onToggleFocus: () => void;
};

/* === Components === */

export function CycleEvidencePane({
  cycleId,
  focused,
  live = true,
  plan,
  row,
  snapshot,
  onToggleFocus,
}: CycleEvidencePaneProps) {
  const [plotWindowIndex, setPlotWindowIndex] = useState(DEFAULT_PLOT_WINDOW_INDEX);
  const [plotWindowOverridden, setPlotWindowOverridden] = useState(false);
  const selectedExecution =
    row?.execution && !isPlannedExecution(row.execution.status) ? row.execution : null;
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
  const document = selectedExecution ? (executionDocumentQuery.data ?? null) : null;
  const documentQuery = executionDocumentQuery;

  const plotWindow =
    PLOT_WINDOW_OPTIONS[plotWindowIndex] ?? PLOT_WINDOW_OPTIONS[DEFAULT_PLOT_WINDOW_INDEX];
  const template =
    plan?.templates.find((item) => item.template_id === row?.case.templateId) ?? null;

  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const documentError = documentQuery.error;
  const windowed = documentHasWindowedCards(sections);
  const windowElapsedUnit = documentWindowElapsedUnit(sections);

  return (
    <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
      {/* The document's own toolbar: the plot window and Focus, nothing
          else; running the cycle belongs to the cockpit above. A frozen
          report has no cockpit group naming the execution, so there the
          toolbar names the scope instead. */}
      <div className="grid min-h-[var(--page-bar-height)] grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--bar-gap)] border-b px-4 py-[var(--bar-pad-y)]">
        {live ? (
          <span />
        ) : (
          <h2 className="min-w-0 truncate font-heading text-sm leading-6 font-semibold tracking-wide uppercase">
            {row ? (selectedExecution ? `Execution ${row.ordinal}` : "Not Run Yet") : "Cycle"}
          </h2>
        )}
        <div className="flex items-center justify-end gap-[var(--bar-gap)] [&_[data-slot=button]:not([class*='h-['])]:h-[var(--bar-control-height)]">
          {windowed && live ? (
            <div className="h-full w-full min-w-56 xl:w-80">
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
          ) : null}
          {/* A frozen plot shows the window it was drawn with. Saying so is
              honest; offering a slider that changes nothing is not. */}
          {windowed && !live ? (
            <span className="text-xs text-muted-foreground">
              {windowElapsedUnit ? "Elapsed window" : "Plot window"}{" "}
              {plotWindowValueLabel(plotWindow, windowElapsedUnit)}
            </span>
          ) : null}
          {/* The toolbar's right end: the evidence takes the window, and says
              how to give it back. Escape leaves too. */}
          {live ? (
            <Button onClick={onToggleFocus} size="sm" variant={focused ? "secondary" : "outline"}>
              {focused ? (
                <Minimize2 data-icon="inline-start" />
              ) : (
                <Maximize2 data-icon="inline-start" />
              )}
              {focused ? "Leave Focus" : "Focus"}
            </Button>
          ) : null}
        </div>
      </div>

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
        ) : !selectedExecution ? (
          <PaneNotice
            message="Select an execution to read its document and evidence."
            title="Select Execution"
          />
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
