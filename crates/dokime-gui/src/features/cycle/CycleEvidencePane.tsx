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
 * instead of offering a slider, the attempt selector becomes the list of
 * attempts, and the attempt's records follow the document as a static console.
 */

/* === External Imports === */

import { ChevronLeft, ChevronRight, Maximize2, Minimize2, Pause, Play, Square } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { CaseParameterGrid } from "@/components/CaseParameterGrid";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { HubLiveState, RecordScope, TestCyclePlan, TestExecution } from "@/lib/types";
import { useCycleCommands } from "./cycle-queries";
import {
  attemptIndexOf,
  attemptLabel,
  type CycleCaseRow,
  executionStatusLabel,
} from "./cycle-workspace-model";
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
  executionId: string | undefined;
  plan: TestCyclePlan | null;
  row: CycleCaseRow | null;
  snapshot: HubLiveState | null;
  /** True while the shell has given the evidence the whole window. */
  focused: boolean;
  /** False in the frozen report: no slider, no focus, no live plot window. */
  live?: boolean;
  onSelectAttempt: (executionId: string) => void;
  onToggleFocus: () => void;
};

/* === Components === */

export function CycleEvidencePane({
  cycleId,
  executionId,
  focused,
  live = true,
  plan,
  row,
  snapshot,
  onSelectAttempt,
  onToggleFocus,
}: CycleEvidencePaneProps) {
  const commands = useCycleCommands(cycleId);
  const [plotWindowIndex, setPlotWindowIndex] = useState(DEFAULT_PLOT_WINDOW_INDEX);
  const [plotWindowOverridden, setPlotWindowOverridden] = useState(false);
  /* Only the attempts with evidence behind them: a planned placeholder is the
     case's parameters, not an attempt, and browsing to it would open a
     document that does not exist. */
  const attempts = row?.browsable ?? [];
  // While the route catches up with a freshly selected case, show its newest
  // attempt until its execution document is available.
  const attemptIndex = attemptIndexOf(attempts, executionId);
  const attempt = attemptIndex >= 0 ? (attempts[attemptIndex] ?? null) : null;
  const scopeAttemptIndex = attemptIndex >= 0 ? attemptIndex : attempts.length - 1;
  const scope = useMemo<RecordScope>(
    () =>
      attempt
        ? {
            test_cycle_id: cycleId,
            test_execution_id: attempt.test_execution_id,
            test_case_id: attempt.case_id,
          }
        : row
          ? { test_cycle_id: cycleId, test_case_id: row.caseId }
          : { test_cycle_id: cycleId },
    [attempt, cycleId, row],
  );

  const detailQuery = useExecutionDetailQuery(attempt?.test_execution_id ?? null, Boolean(attempt));
  const execution: TestExecution | null = detailQuery.data ?? attempt ?? null;
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

  const executionDocumentQuery = useExecutionDocumentQuery({
    cycleId: attempt ? cycleId : null,
    executionId: attempt?.test_execution_id ?? null,
  });
  const document = attempt ? (executionDocumentQuery.data ?? null) : null;
  const documentQuery = executionDocumentQuery;

  const plotWindow =
    PLOT_WINDOW_OPTIONS[plotWindowIndex] ?? PLOT_WINDOW_OPTIONS[DEFAULT_PLOT_WINDOW_INDEX];
  const template = plan?.templates.find((item) => item.template_id === row?.templateId) ?? null;

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
            {row
              ? attempts.length > 0
                ? attemptLabel(scopeAttemptIndex, attempts.length)
                : "No Attempts Yet"
              : "Cycle"}
          </h2>
          {row && attempts.length > 0 ? (
            <AttemptSelector
              attempts={attempts}
              index={attemptIndex}
              live={live}
              onSelect={onSelectAttempt}
            />
          ) : null}
        </div>
        <div className="flex items-center justify-end gap-[var(--bar-gap)] [&_[data-slot=button]:not([class*='h-['])]:h-[var(--bar-control-height)]">
          {live && controls && (controls.canPause || controls.canResume || controls.canStop) ? (
            <fieldset aria-label="Selected Execution Controls" className="flex items-center gap-1">
              {controls.paused ? (
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
        ) : row && attempts.length === 0 ? (
          <div className="grid gap-4">
            <p className="text-sm text-muted-foreground">
              This case has not run yet. These are the parameters the next attempt will use.
            </p>
            {template?.description ? <MarkdownText text={template.description} /> : null}
            {row.testCase ? <CaseParameterGrid testCase={row.testCase} /> : null}
          </div>
        ) : !attempt ? (
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
              parameters={{ case: row?.testCase?.parameters, execution: execution?.parameters }}
              noteElapsedStart={attempt?.started_at ?? plan?.test_cycle.started_at ?? null}
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

function AttemptSelector({
  attempts,
  index,
  live,
  onSelect,
}: {
  attempts: TestExecution[];
  index: number;
  live: boolean;
  onSelect: (executionId: string) => void;
}) {
  const current = index >= 0 ? index : attempts.length - 1;

  /* A frozen file offers the attempts it contains as a list, not a select: the
     reader is navigating an outline, not choosing what the backend shows. */
  if (!live) {
    if (attempts.length < 2) return null;
    return (
      <div className="flex flex-wrap items-center gap-1">
        {attempts.map((attempt, position) => (
          <Button
            key={attempt.test_execution_id}
            onClick={() => onSelect(attempt.test_execution_id)}
            size="sm"
            variant={position === current ? "secondary" : "ghost"}
          >
            {attemptLabel(position, attempts.length)}
          </Button>
        ))}
      </div>
    );
  }

  const previous = attempts[current - 1];
  const next = attempts[current + 1];

  return (
    <div className="grid grid-cols-[auto_auto_auto] items-center gap-1">
      <Button
        aria-label="Previous Attempt"
        disabled={!previous}
        onClick={() => previous && onSelect(previous.test_execution_id)}
        size="icon-sm"
        variant="outline"
      >
        <ChevronLeft />
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button aria-label="Choose Attempt" size="sm" variant="outline">
            All Attempts
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuRadioGroup
            onValueChange={onSelect}
            value={attempts[current]?.test_execution_id ?? ""}
          >
            {attempts.map((attempt, position) => (
              <DropdownMenuRadioItem
                key={attempt.test_execution_id}
                value={attempt.test_execution_id}
              >
                {attemptLabel(position, attempts.length)} — {executionStatusLabel(attempt.status)}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      <Button
        aria-label="Next Attempt"
        disabled={!next}
        onClick={() => next && onSelect(next.test_execution_id)}
        size="icon-sm"
        variant="outline"
      >
        <ChevronRight />
      </Button>
    </div>
  );
}

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
