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
 * Start or Stop for an unstarted attempt, Re-Run for an ended attempt,
 * Pause or Resume for an execution-addressed run, and Stop Execution, in that
 * order (`executionControlSet`); the group is absent when the execution offers
 * none. Eligibility is the runtime's: the execution's own `controls` say what
 * it admits while it runs, and its case's actions say whether it may start or
 * a new attempt may be appended. Start uses the shared StartExecution operation.
 * Re-Run appends a new execution and keeps this attempt's captured evidence;
 * the new attempt becomes the selection and waits for an explicit Start.
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
  executionControlSet,
  isUnstartedExecution,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { useExecutionDetailQuery } from "./server-state";

/* === Constants === */

const CONTROL_REPORT: Record<ControlAction, string> = {
  pause: "Execution Pause Requested",
  resume: "Execution Resume Requested",
  stop: "Execution Stopped And Evidence Saved",
};

/* === Types === */

export type CycleExecutionControlsProps = {
  cycleId: string;
  selection: CycleSelection;
  onSelectExecution: (executionId: string) => void;
};

type ControlAction = "pause" | "resume" | "stop";

/* === Components === */

export function CycleExecutionControls({ cycleId, selection, onSelectExecution }: CycleExecutionControlsProps) {
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const detail = useCycleDetailQuery(cycleId).data;
  const rows = useMemo(() => buildCycleExecutionRows(buildCycleCases(detail?.plan ?? null), detail?.executions ?? []), [detail]);
  const row = selectedCycleRow(rows, selection);
  const started = row?.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null;
  const detailQuery = useExecutionDetailQuery(started?.test_execution_id ?? null, Boolean(started));
  if (!row?.execution) return null;

  const execution = (started && detailQuery.data) || row.execution;
  const controls = execution.controls;
  const startAllowed = detail?.plan?.test_cycle.phase === "active";
  const rerunAllowed = startAllowed;
  const stopping = commands.stopExecution.isPending;
  const set = stopping ? ["Stop Execution"] : executionControlSet(execution.status, controls);

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
        toast.success("New Attempt Ready To Start");
      })
      .catch((error: unknown) =>
        toast.error("Case Re-Run Failed", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  const start = () => {
    void commands.startExecution
      .mutateAsync(execution.test_execution_id)
      .then(() => toast.success("Execution Started"))
      .catch((error: unknown) =>
        toast.error("Execution Could Not Start", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  if (!set.length) return null;

  return (
    <fieldset aria-label={`Execution #${row.ordinal} Controls`} className={PAGE_BAR_CONTROLS_CLASS}>
      {set.includes("Start") ? (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!startAllowed || commands.startExecution.isPending}
          onClick={start}
          title={startAllowed ? undefined : caseActionTooltip(row.case)}
          variant="default"
        >
          <Play data-icon="inline-start" />
          Start
        </Button>
      ) : null}
      {set.includes("Re-Run") ? (
        <Button
          aria-label="Re-Run Case"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={!rerunAllowed || commands.appendAttempt.isPending}
          onClick={rerun}
          title={rerunAllowed ? undefined : caseActionTooltip(row.case)}
          variant="outline"
        >
          <RotateCcw data-icon="inline-start" />
          Re-Run
        </Button>
      ) : null}
      {/* Pause And Resume Follow The Selected Execution's Admission */}
      {!controls || !(set.includes("Pause Execution") || set.includes("Resume Execution")) ? null : controls.paused ? (
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
      {set.includes("Stop Execution") ? (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={
            !startAllowed || Boolean(controls?.stopRequested) || stopping || (!isUnstartedExecution(execution.status) && !controls?.canStop)
          }
          onClick={() => control("stop")}
          variant="danger"
        >
          <Square data-icon="inline-start" />
          {stopping ? "Stopping — Saving Evidence" : "Stop Execution"}
        </Button>
      ) : null}
    </fieldset>
  );
}
