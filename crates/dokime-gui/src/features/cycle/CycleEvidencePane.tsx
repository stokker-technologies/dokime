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

import { Maximize2, Minimize2, Pause, Play, RotateCcw, Square } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { CaseParameterGrid } from "@/components/CaseParameterGrid";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { HubLiveState, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import { useCycleCommands, useRerunCase } from "./cycle-queries";
import { type CycleExecutionRow, isPlannedExecution } from "./cycle-workspace-model";
import { DocumentView, documentHasWindowedCards, documentWindowElapsedUnit } from "./DocumentView";
import {
  DEFAULT_PLOT_WINDOW_INDEX,
  PLOT_WINDOW_OPTIONS,
  PlotWindowSlider,
  plotWindowValueLabel,
} from "./PlotWindowSlider";
import { useExecutionDetailQuery, useExecutionDocumentQuery } from "./server-state";

/* === Constants === */

const TERMINAL_EXECUTION_STATUSES: readonly TestExecution["status"][] = [
  "passed",
  "failed",
  "stopped",
  "interrupted",
  "disconnected",
];

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
  onSelectExecution: (executionId: string) => void;
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
  onSelectExecution,
  onToggleFocus,
}: CycleEvidencePaneProps) {
  const commands = useCycleCommands(cycleId);
  const [plotWindowIndex, setPlotWindowIndex] = useState(DEFAULT_PLOT_WINDOW_INDEX);
  const [plotWindowOverridden, setPlotWindowOverridden] = useState(false);
  const rerunCase = useRerunCase(cycleId);
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
  const controls = execution?.controls;
  const control = (action: "pause" | "resume" | "stop") => {
    if (!execution) return;
    const id = execution.test_execution_id;
    const mutation = {
      pause: commands.pauseExecution,
      resume: commands.resumeExecution,
      stop: commands.stopExecution,
    }[action];
    const message = {
      pause: "Execution Pause Requested",
      resume: "Execution Resume Requested",
      stop: "Execution Stop Requested",
    }[action];
    void mutation
      .mutateAsync(id)
      .then(() => toast.success(message))
      .catch((error: unknown) => {
        toast.error("Execution Control Failed", {
          description: error instanceof Error ? error.message : String(error),
        });
      });
  };

  /* Re-Run appends a new execution through the case scheduling owner and
     preserves this execution's captured evidence. */
  const rerunnable =
    live &&
    execution !== null &&
    TERMINAL_EXECUTION_STATUSES.includes(execution.status) &&
    !controls;
  const rerun = () => {
    if (!execution) return;
    void rerunCase
      .rerun(execution.case_id)
      .then((created) => {
        if (created) onSelectExecution(created);
        toast.success("Case Re-Run Started");
      })
      .catch((error: unknown) => {
        toast.error("Case Re-Run Failed", {
          description: error instanceof Error ? error.message : String(error),
        });
      });
  };

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
      {/* One 40px toolbar (48px at touch): scope on the left, the plot window
          on the right. Nothing introduces the document a second time. */}
      <div className="grid min-h-[var(--page-bar-height)] grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--bar-gap)] border-b px-4 py-[var(--bar-pad-y)]">
        <div className="grid min-w-0 grid-cols-[auto_minmax(0,auto)] items-center justify-start gap-2">
          <h2 className="min-w-0 truncate font-heading text-sm leading-6 font-semibold tracking-wide uppercase">
            {row ? (selectedExecution ? `Execution ${row.ordinal}` : "Not Run Yet") : "Cycle"}
          </h2>
        </div>
        <div className="flex items-center justify-end gap-[var(--bar-gap)] [&_[data-slot=button]:not([class*='h-['])]:h-[var(--bar-control-height)]">
          {rerunnable ? (
            <Button
              disabled={commands.scheduleRerun.isPending}
              onClick={rerun}
              size="sm"
              variant="outline"
            >
              <RotateCcw data-icon="inline-start" />
              Re-Run Case
            </Button>
          ) : null}
          {live && controls && (controls.canPause || controls.canResume || controls.canStop) ? (
            <fieldset aria-label="Selected Execution Controls" className="flex items-center gap-1">
              {/* A member of a cycle run pauses with its cycle; only an
                  execution-addressed run offers its own Pause and Resume. */}
              {!controls.canPause && !controls.canResume ? null : controls.paused ? (
                <Button
                  disabled={!controls.canResume || commands.resumeExecution.isPending}
                  onClick={() => control("resume")}
                  size="sm"
                  variant="outline"
                >
                  <Play data-icon="inline-start" />
                  Resume Execution
                </Button>
              ) : (
                <Button
                  disabled={!controls.canPause || commands.pauseExecution.isPending}
                  onClick={() => control("pause")}
                  size="sm"
                  variant="outline"
                >
                  <Pause data-icon="inline-start" />
                  Pause Execution
                </Button>
              )}
              <Button
                disabled={
                  !controls.canStop || controls.stopRequested || commands.stopExecution.isPending
                }
                onClick={() => control("stop")}
                size="sm"
                variant="outline"
              >
                <Square data-icon="inline-start" />
                Stop Execution
              </Button>
            </fieldset>
          ) : null}
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
