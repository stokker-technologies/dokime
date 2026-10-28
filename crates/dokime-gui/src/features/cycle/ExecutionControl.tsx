/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The execution page's Control card and the lifecycle actions it offers.
 *
 * `useExecutionActions` decides once, from the runtime's own statement of the
 * execution (`executionControlState`), what the card says and which actions
 * it offers; the card and the page bar's primary action both read that one
 * decision, so they never disagree. Start and Cancel belong to an attempt that
 * has not started: Cancel skips it, recording it Cancelled while the cycle
 * continues. Pause or Resume and Stop belong to a running attempt; Stop is the
 * only destructive action. Once the last step settles the card says it is
 * finalizing and offers nothing until the attempt ends; an ended attempt
 * offers Re-Run, which appends a new attempt and selects it. The card exists
 * only while live: a file has nothing to command.
 */

import { Ban, Pause, Play, RotateCcw, Square } from "lucide-react";
import type { ReactNode } from "react";
import { toast } from "sonner";

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { stepRecordLabel } from "@/components/document/StepRecordsCard";
import { Button } from "@/components/ui/button";
import type { PromptRequest, TestExecution } from "@/lib/types";
import { ControlCard } from "./ControlCard";
import { type ExecutionAction, type ExecutionControlState, executionControlState } from "./control-model";
import { useCycleCommands, useRerunCase } from "./cycle-queries";
import { type CycleExecutionRow, caseActionTooltip, progressStepLabel } from "./cycle-workspace-model";

const REPORTS: Record<ExecutionAction, { done: string; failed: string }> = {
  Cancel: { done: "Execution Cancelled", failed: "Execution Could Not Be Cancelled" },
  Pause: { done: "Execution Pause Requested", failed: "Execution Control Failed" },
  "Re-Run": { done: "New Attempt Ready To Start", failed: "Case Re-Run Failed" },
  Resume: { done: "Execution Resume Requested", failed: "Execution Control Failed" },
  Start: { done: "Execution Started", failed: "Execution Could Not Start" },
  Stop: { done: "Execution Stopped And Evidence Saved", failed: "Execution Control Failed" },
};

const ACTION_ICONS: Record<ExecutionAction, ReactNode> = {
  Cancel: <Ban data-icon="inline-start" />,
  Pause: <Pause data-icon="inline-start" />,
  "Re-Run": <RotateCcw data-icon="inline-start" />,
  Resume: <Play data-icon="inline-start" />,
  Start: <Play data-icon="inline-start" />,
  Stop: <Square data-icon="inline-start" />,
};

export type ExecutionActions = {
  state: ExecutionControlState;
  progress: { completed: number; total: number } | null;
  /** Whether the action's command is in flight or refused right now. */
  disabled: (action: ExecutionAction) => boolean;
  run: (action: ExecutionAction) => void;
  /** Why the runtime refuses the actions, when it does. */
  refusal?: string;
};

export type ExecutionControlProps = {
  actions: ExecutionActions;
  ordinal: number;
};

export type ExecutionActionButtonProps = {
  action: ExecutionAction;
  actions: ExecutionActions;
};

export function ExecutionControl({ actions, ordinal }: ExecutionControlProps) {
  return (
    <ControlCard
      detail={actions.state.detail}
      actions={actions.state.actions.map((action) => <ExecutionActionButton action={action} actions={actions} key={action} />)}
      label={`Execution #${ordinal} Controls`}
      progress={actions.progress}
      state={actions.state.line}
    />
  );
}

/** One lifecycle action, styled by what it does: Start leads, Stop warns. */
export function ExecutionActionButton({ action, actions }: ExecutionActionButtonProps) {
  const disabled = actions.disabled(action);
  return (
    <Button
      className="h-[var(--bar-control-height)] px-3"
      disabled={disabled}
      onClick={() => actions.run(action)}
      title={disabled ? actions.refusal : undefined}
      variant={action === "Start" ? "default" : action === "Stop" ? "danger" : "outline"}
    >
      {ACTION_ICONS[action]}
      {action === "Start" || action === "Re-Run" ? action : `${action} Execution`}
    </Button>
  );
}

/** The one decision both the Control card and the page bar read. */
export function useExecutionActions({
  cycleActive,
  cycleId,
  execution,
  onSelectExecution,
  prompts,
  row,
}: {
  cycleActive: boolean;
  cycleId: string;
  /** `null` in a file, or for a case that never ran: nothing to command. */
  execution: TestExecution | null;
  onSelectExecution: (executionId: string) => void;
  prompts: readonly PromptRequest[];
  row: CycleExecutionRow;
}): ExecutionActions | null {
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const live = useExecutionProgress(execution?.test_execution_id ?? null);
  if (!execution) return null;
  const progress = live && live.testExecutionId === execution.test_execution_id ? live : null;
  const state = executionControlState({
    controls: execution.controls,
    progress: progress
      ? {
          completed: progress.completed,
          id: progress.stepId,
          label: stepLabel(execution, progress.stepId, progress.phase),
          total: progress.total,
        }
      : null,
    promptWaiting: prompts.some((prompt) => prompt.scope?.test_execution_id === execution.test_execution_id),
    status: execution.status,
    stopping: commands.stopExecution.isPending,
  });
  const id = execution.test_execution_id;
  const commandOf = (action: ExecutionAction): { pending: boolean; run: () => Promise<unknown> } => {
    switch (action) {
      case "Start":
        return { pending: commands.startExecution.isPending, run: () => commands.startExecution.mutateAsync(id) };
      case "Cancel":
        return { pending: commands.cancelExecution.isPending, run: () => commands.cancelExecution.mutateAsync(id) };
      case "Pause":
        return { pending: commands.pauseExecution.isPending, run: () => commands.pauseExecution.mutateAsync(id) };
      case "Resume":
        return { pending: commands.resumeExecution.isPending, run: () => commands.resumeExecution.mutateAsync(id) };
      case "Stop":
        return { pending: commands.stopExecution.isPending, run: () => commands.stopExecution.mutateAsync(id) };
      case "Re-Run":
        return {
          pending: rerunCase.pending,
          run: async () => {
            const created = await rerunCase.rerun(execution.case_id);
            if (created) onSelectExecution(created);
          },
        };
    }
  };
  return {
    disabled: (action) => !cycleActive || commandOf(action).pending,
    progress: state.phase === "running" || state.phase === "waiting" || state.phase === "paused" ? progress : null,
    refusal: cycleActive ? undefined : (caseActionTooltip(row.case) ?? "Only Active Cycles Accept Commands"),
    run: (action) => {
      const report = REPORTS[action];
      void commandOf(action)
        .run()
        .then(() => toast.success(report.done))
        .catch((error: unknown) =>
          toast.error(report.failed, {
            description: error instanceof Error ? error.message : String(error),
          }),
        );
    },
    state,
  };
}

/** The running step's human label, as the Steps card names it; the phase for an opaque id. */
export function stepLabel(execution: Pick<TestExecution, "steps">, stepId: string, phase: string): string {
  const record = execution.steps.find((item) => item.record_type === "step" && item.step_id === stepId);
  return record ? stepRecordLabel(record, false) : progressStepLabel(stepId, phase);
}
