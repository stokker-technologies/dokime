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
 * decision, so they never disagree. The host admits Start for an unstarted
 * attempt and Cancel for active work. Pause or Resume and Stop belong to a
 * running attempt; Stop is the only destructive action. Once the last step settles the card says it is
 * finalizing and offers nothing until the execution seals; a sealed execution
 * offers Re-Run, which appends a new attempt and selects it. The card exists
 * only while live: a file has nothing to command.
 */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";
import type { PromptRequest, TestExecution } from "@/lib/types";
import { type ExecutionAction, type ExecutionControlState, executionControlState } from "./control-model";
import { useCycleCommands, useRerunCase } from "./cycle-queries";
import { type CycleExecutionRow, caseActionTooltip, isUnstartedExecution } from "./cycle-workspace-model";

const REPORTS: Record<ExecutionAction, { done: string; failed: string }> = {
  Cancel: { done: "Execution Cancelled", failed: "Execution Could Not Be Cancelled" },
  Pause: { done: "Execution Pause Requested", failed: "Execution Control Failed" },
  "Re-Run": { done: "New Attempt Ready to Start", failed: "Case Re-Run Failed" },
  Resume: { done: "Execution Resume Requested", failed: "Execution Control Failed" },
  Start: { done: "Execution Started", failed: "Execution Could Not Start" },
  Stop: { done: "Execution Stopped and Evidence Saved", failed: "Execution Control Failed" },
};

export type ExecutionActions = {
  state: ExecutionControlState;
  progress: { completed: number; phase: string; total: number } | null;
  /** Whether the action's command is in flight or refused right now. */
  disabled: (action: ExecutionAction) => boolean;
  run: (action: ExecutionAction) => void;
  /** Why the runtime refuses the actions, when it does. */
  refusal?: string;
};

/** The one decision both the Control card and the page bar read. */
export function useExecutionActions({
  cycleActive,
  cycleId,
  execution,
  onSelectExecution,
  prompts,
  pluginAvailable,
  row,
}: {
  cycleActive: boolean;
  cycleId: string;
  /** `null` in a file, or for a case that never ran: nothing to command. */
  execution: TestExecution | null;
  onSelectExecution: (executionId: string) => void;
  prompts: readonly PromptRequest[];
  pluginAvailable?: boolean;
  row: CycleExecutionRow;
}): ExecutionActions | null {
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const live = useExecutionProgress(execution && !isUnstartedExecution(execution.status) ? execution.test_execution_id : null);
  if (!execution) {
    return null;
  }
  const progress = live && live.testExecutionId === execution.test_execution_id ? live : null;
  const state = executionControlState({
    pluginAvailable,
    report: execution.report,
    controls: execution.controls,
    progress: progress
      ? {
          completed: progress.completed,
          label: progress.label,
          total: progress.total,
        }
      : null,
    promptWaiting: prompts.some((prompt) => prompt.scope?.executionId === execution.test_execution_id),
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
            if (created) {
              onSelectExecution(created);
            }
          },
        };
    }
  };
  return {
    disabled: (action) => (action !== "Start" && !cycleActive) || commandOf(action).pending,
    progress: state.phase === "running" || state.phase === "waiting" || state.phase === "paused" ? progress : null,
    refusal: cycleActive ? undefined : (caseActionTooltip(row.case) ?? "Only Active Cycles Accept Commands"),
    run: (action) => {
      const report = REPORTS[action];
      void commandOf(action)
        .run()
        .then(() => toast.success(report.done))
        .catch((error: unknown) => toast.error(failureOf(error, report.failed)));
    },
    state,
  };
}
