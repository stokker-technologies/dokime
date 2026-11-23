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
 * `executionActions` decides once, from the runtime's own statement of the
 * execution (`executionControlState`), what the card says and which actions
 * it offers; the card and the page bar's primary action both read that one
 * decision, so they never disagree. The host admits Start for an unstarted
 * attempt and Cancel for active work. Pause or Resume and Stop belong to a
 * running attempt; Stop is the only destructive action. Once the last step settles the card says it is
 * finalizing and offers nothing until the execution seals; a sealed execution
 * offers Re-Run, which appends a new attempt and selects it. The card exists
 * only while live: a file has nothing to command.
 */

import type { LiveExecutionProgress } from "@/app/live/execution-progress";
import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";
import type { PromptRequest, TestExecution } from "@/lib/types";
import { type ExecutionAction, type ExecutionControlState, executionControlState } from "./control-model";
import type { createCycleCommands } from "./cycle-queries";
import { type CycleExecutionRow, caseActionTooltip } from "./cycle-workspace-model";

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

type ExecutionActionInput = {
  cycleActive: boolean;
  execution: TestExecution | null;
  onSelectExecution: (executionId: string) => void;
  prompts: readonly PromptRequest[];
  pluginAvailable?: boolean;
  row: CycleExecutionRow;
};

/** One Action Policy For the Control Card and the Page Bar. */
export function executionActions(
  { cycleActive, execution, onSelectExecution, prompts, pluginAvailable, row }: ExecutionActionInput,
  commands: ReturnType<typeof createCycleCommands>,
  live: LiveExecutionProgress | null,
): ExecutionActions | null {
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
          pending: commands.appendAttempt.isPending,
          run: async () => {
            const created = await commands.appendAttempt.mutateAsync(execution.case_id);
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
