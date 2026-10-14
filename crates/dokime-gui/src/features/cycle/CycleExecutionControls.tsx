/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The selected execution's run controls, published into the page bar's
 * action slot while an execution is selected.
 *
 * Re-Run for an ended attempt, Pause or Resume for an execution-addressed run,
 * and Stop Execution; the group is absent when the execution offers none. Eligibility is the runtime's: the
 * execution's own `controls` say what it admits while it runs, and its case's
 * actions say whether a re-run may be scheduled. Re-Run appends a new
 * execution through the case scheduling owner and keeps this attempt's
 * captured evidence; the new attempt becomes the selection.
 */

/* === External Imports === */

import { Pause, Play, RotateCcw, Square } from "lucide-react";
import { useMemo } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { PAGE_BAR_CONTROL_CLASS, PAGE_BAR_CONTROLS_CLASS } from "./CycleRunControls";
import { useCycleCommands, useCycleDetailQuery, useRerunCase } from "./cycle-queries";
import {
  buildCycleCases,
  buildCycleExecutionRows,
  type CycleSelection,
  caseActionTooltip,
  isTerminalExecutionStatus,
  isUnstartedExecution,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { useExecutionDetailQuery } from "./server-state";

/* === Types === */

export type CycleExecutionControlsProps = {
  cycleId: string;
  selection: CycleSelection;
  onSelectExecution: (executionId: string) => void;
};

type ControlAction = "pause" | "resume" | "stop";

/* === Constants === */

const CONTROL_REPORT: Record<ControlAction, string> = {
  pause: "Execution Pause Requested",
  resume: "Execution Resume Requested",
  stop: "Execution Stop Requested",
};

/* === Components === */

export function CycleExecutionControls({
  cycleId,
  selection,
  onSelectExecution,
}: CycleExecutionControlsProps) {
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const detail = useCycleDetailQuery(cycleId).data;
  const rows = useMemo(
    () => buildCycleExecutionRows(buildCycleCases(detail?.plan ?? null), detail?.executions ?? []),
    [detail],
  );
  const row = selectedCycleRow(rows, selection);
  const selected =
    row?.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
  const detailQuery = useExecutionDetailQuery(
    selected?.test_execution_id ?? null,
    Boolean(selected),
  );
  if (!row || !selected) return null;

  const execution = detailQuery.data ?? selected;
  const controls = execution.controls;
  const caseActions = row.case.actions;
  const rerunAllowed = caseActions === null || caseActions.schedule_rerun;
  const rerunnable = isTerminalExecutionStatus(execution.status) && !controls;
  const pauseable = controls && (controls.canPause || controls.canResume);

  const control = (action: ControlAction) => {
    const mutation = {
      pause: commands.pauseExecution,
      resume: commands.resumeExecution,
      stop: commands.stopExecution,
    }[action];
    void mutation
      .mutateAsync(execution.test_execution_id)
      .then(() => toast.success(CONTROL_REPORT[action]))
      .catch((error: unknown) =>
        toast.error("Execution Control Failed", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  const rerun = () => {
    void rerunCase
      .rerun(execution.case_id)
      .then((created) => {
        if (created) onSelectExecution(created);
        toast.success("Case Re-Run Started");
      })
      .catch((error: unknown) =>
        toast.error("Case Re-Run Failed", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  const stoppable = Boolean(controls?.canStop || controls?.stopRequested);
  if (!rerunnable && !pauseable && !stoppable) return null;

  return (
    <fieldset aria-label={`Execution #${row.ordinal} Controls`} className={PAGE_BAR_CONTROLS_CLASS}>
      {rerunnable ? (
        <Button
          aria-label="Re-Run Case"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!rerunAllowed || commands.scheduleRerun.isPending}
          onClick={rerun}
          title={rerunAllowed ? undefined : caseActionTooltip(row.case)}
          variant="outline"
        >
          <RotateCcw data-icon="inline-start" />
          Re-Run
        </Button>
      ) : null}
      {/* A member of a cycle run pauses with its cycle; only an
          execution-addressed run offers its own Pause and Resume. */}
      {!pauseable ? null : controls.paused ? (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!controls.canResume || commands.resumeExecution.isPending}
          onClick={() => control("resume")}
          variant="outline"
        >
          <Play data-icon="inline-start" />
          Resume Execution
        </Button>
      ) : (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!controls.canPause || commands.pauseExecution.isPending}
          onClick={() => control("pause")}
          variant="outline"
        >
          <Pause data-icon="inline-start" />
          Pause Execution
        </Button>
      )}
      {stoppable && controls ? (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!controls.canStop || controls.stopRequested || commands.stopExecution.isPending}
          onClick={() => control("stop")}
          variant="danger"
        >
          <Square data-icon="inline-start" />
          Stop Execution
        </Button>
      ) : null}
    </fieldset>
  );
}
