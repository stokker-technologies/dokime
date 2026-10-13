/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle workspace: one cycle and the evidence its cases produced.
 *
 * The layout never changes shape as a cycle runs. The header states what the
 * cycle is and owns every whole-cycle command; the sidebar tree owns selection;
 * the evidence pane has the whole main region and shows what the selection
 * holds. The route owns the URL, so every selection change leaves here through
 * `onSelectionChange`.
 *
 * `live` is the only difference between the application and the frozen report,
 * which mounts this same workspace over a snapshot. With it off the header
 * keeps identity, status and progress and loses every command, the prompt
 * banner is absent, and the reads answer from the snapshot context rather than
 * the backend. There is no second workspace.
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { openLocalPath } from "@/lib/desktop-open";
import { useFocusMode } from "@/shell/focus-mode";
import { useSelectionStore } from "@/stores/selection-store";
import { CycleEvidencePane } from "./CycleEvidencePane";
import { CyclePromptBanner } from "./CyclePromptBanner";
import { CycleWorkspaceHeader } from "./CycleWorkspaceHeader";
import {
  useCycleCommands,
  useCycleDetailQuery,
  useCyclePendingPromptsQuery,
  useCycleRuntimeShellQuery,
} from "./cycle-queries";
import {
  buildCycleCaseRows,
  cycleRunRefusal,
  finishedCaseCount,
  isActiveExecutionStatus,
  progressStepLabel,
  runControlAvailability,
  runControlVisibility,
  startableCaseCount,
} from "./cycle-workspace-model";

/* === Types === */

export type CycleWorkspaceProps = {
  cycleId: string;
  caseId?: string;
  executionId?: string;
  /** False in the frozen report: evidence without commands. */
  live?: boolean;
  onSelectionChange: (next: { caseId?: string; executionId?: string }) => void;
};

type PendingConfirm = { kind: "clear" } | { kind: "stop" } | null;

/* === Components === */

export function CycleWorkspace({
  cycleId,
  caseId,
  executionId,
  live = true,
  onSelectionChange,
}: CycleWorkspaceProps) {
  const navigate = useNavigate();
  const session = useNativeSessionOptional();
  const detailQuery = useCycleDetailQuery(cycleId);
  const runtimeQuery = useCycleRuntimeShellQuery(cycleId);
  const promptsQuery = useCyclePendingPromptsQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const focus = useFocusMode();
  const followActive = useSelectionStore((state) => state.followActive);
  const setSelectionState = useSelectionStore((state) => state.setSelectionState);
  const [policyOpen, setPolicyOpen] = useState(false);
  const [confirm, setConfirm] = useState<PendingConfirm>(null);
  const autoResolvedCase = useRef<string | null>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const rows = useMemo(() => buildCycleCaseRows(plan, executions), [executions, plan]);
  const selectedRow = rows.find((row) => row.caseId === caseId) ?? null;
  const finished = finishedCaseCount(rows);
  const startable = startableCaseCount(rows);
  const refusal = cycleRunRefusal(rows);
  const status = plan?.test_cycle.status ?? "planned";
  const availability = runControlAvailability(status, startable);
  const visibility = runControlVisibility(status, startable, refusal !== null);
  const activeExecutionId =
    plan?.test_cycle.current_test_execution_id ??
    rows.find((row) => row.status === "running")?.latest?.test_execution_id ??
    null;
  const progress = useExecutionProgress(activeExecutionId);
  const activity = useMemo(() => {
    const activeRow = rows.find(
      (row) => row.latest?.test_execution_id === progress?.testExecutionId,
    );
    if (!progress || !activeRow) return null;
    const step = progressStepLabel(progress.stepId, progress.phase);
    return `${activeRow.label} · ${step} ${progress.completed}/${progress.total}`;
  }, [progress, rows]);

  /* Selection resolves to the newest attempt only when the operator picks a
     different case. A refresh that adds an attempt must never move them. */
  useEffect(() => {
    if (!caseId || !selectedRow) return;
    const known = selectedRow.browsable.some(
      (attempt) => attempt.test_execution_id === executionId,
    );
    if (known) {
      autoResolvedCase.current = caseId;
      return;
    }
    if (executionId && autoResolvedCase.current === caseId) return;
    if (selectedRow.browsable.length === 0) return;
    autoResolvedCase.current = caseId;
    onSelectionChange({
      caseId,
      executionId: selectedRow.browsable.at(-1)?.test_execution_id,
    });
  }, [caseId, executionId, onSelectionChange, selectedRow]);

  /* Follow Active, and only Follow Active, moves the operator to an attempt
     they did not choose: the newest one, while it is still running. With the
     flag off a new attempt arriving leaves the shown evidence alone. */
  useEffect(() => {
    if (!followActive || !caseId || !selectedRow) return;
    const latest = selectedRow.latest;
    if (!latest || latest.test_execution_id === executionId) return;
    if (!isActiveExecutionStatus(latest.status)) return;
    autoResolvedCase.current = caseId;
    onSelectionChange({ caseId, executionId: latest.test_execution_id });
  }, [caseId, executionId, followActive, onSelectionChange, selectedRow]);

  /* A command reports what happened, not what was attempted: the operator
     needs the outcome and, when it failed, what the backend said. */
  const run = (report: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(report.done))
      .catch((error: unknown) =>
        toast.error(report.failed, {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  if (detailQuery.isError && !plan) {
    return (
      <WorkspaceNotice
        action={{ label: "Try Again", onClick: () => void detailQuery.refetch() }}
        message={
          detailQuery.error instanceof Error
            ? detailQuery.error.message
            : "The cycle could not be read from the backend."
        }
        title="Cycle Unavailable"
      />
    );
  }

  if (!plan) {
    return detailQuery.isPending ? (
      <WorkspaceNotice message="Reading this cycle from the backend." title="Loading" />
    ) : (
      <WorkspaceNotice
        action={{ label: "Go To Cycles", onClick: () => void navigate({ to: "/cycles" }) }}
        message="This cycle is no longer loaded. It may have been cleared."
        title="Cycle Not Found"
      />
    );
  }

  return (
    <div className="grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)]">
      <CycleWorkspaceHeader
        live={live}
        activity={activity}
        availability={availability}
        visibility={visibility}
        pending={{
          pause: commands.pauseCycle.isPending,
          resume: commands.resumeCycle.isPending,
          run: commands.runQueued.isPending,
          stop: commands.stopCycle.isPending,
        }}
        caseTotal={rows.length}
        caseId={caseId}
        cycleId={cycleId}
        finishedCases={finished}
        label={plan.test_cycle.title}
        pendingPrompts={live ? (promptsQuery.data ?? []) : []}
        onAddCases={() => void navigate({ to: "/library", search: { appendTo: cycleId } })}
        followActive={followActive}
        onClearCycle={() => setConfirm({ kind: "clear" })}
        onCopyCycle={() =>
          run({ done: "Cycle copied", failed: "The cycle could not be copied." }, async () => {
            const response = await commands.copyCycle.mutateAsync();
            await navigate({
              to: "/cycles/$cycleId",
              params: { cycleId: response.plan.test_cycle.test_cycle_id },
            });
          })
        }
        onDownloadBundle={() =>
          run({ done: "Bundle saved", failed: "The bundle could not be saved." }, () =>
            commands.downloadBundle.mutateAsync(),
          )
        }
        onOpenFolder={() =>
          run(
            { done: "Folder opened", failed: "The cycle folder could not be opened." },
            async () => {
              const path = await commands.openFolder.mutateAsync();
              if (!(await openLocalPath(path))) throw new Error(`Could not open ${path}.`);
            },
          )
        }
        onPause={() =>
          run({ done: "Cycle paused", failed: "The cycle could not be paused." }, () =>
            commands.pauseCycle.mutateAsync(),
          )
        }
        onResume={() =>
          run({ done: "Cycle resumed", failed: "The cycle could not be resumed." }, () =>
            commands.resumeCycle.mutateAsync(),
          )
        }
        onRunQueued={() =>
          run({ done: "Queued cases started", failed: "The queued cases could not start." }, () =>
            commands.runQueued.mutateAsync(),
          )
        }
        onRuntimePolicy={() => setPolicyOpen(true)}
        onToggleFollowActive={() => setSelectionState({ followActive: !followActive })}
        onStop={() => setConfirm({ kind: "stop" })}
        onWriteReport={() =>
          run({ done: "Report written", failed: "The report could not be written." }, () =>
            commands.writeReport.mutateAsync(),
          )
        }
        runRefusal={refusal}
        showOpenFolder={session !== null && !session.snapshot.connection.remote}
        status={status}
      />

      {live ? <CyclePromptBanner prompts={promptsQuery.data ?? []} /> : null}

      <CycleEvidencePane
        cycleId={cycleId}
        executionId={executionId}
        live={live}
        focused={focus.focused}
        onSelectAttempt={(next) => {
          autoResolvedCase.current = caseId ?? null;
          onSelectionChange({ caseId, executionId: next });
        }}
        onToggleFocus={() => focus.setFocused(!focus.focused)}
        plan={plan}
        row={selectedRow}
        snapshot={runtimeQuery.data ?? null}
      />

      <RuntimePolicyDialog
        continueOnFailure={plan.test_cycle.continue_on_failure}
        onOpenChange={setPolicyOpen}
        onSubmit={(value) =>
          run(
            { done: "Runtime policy updated", failed: "The runtime policy could not be updated." },
            async () => {
              await commands.updatePolicy.mutateAsync(value);
              setPolicyOpen(false);
            },
          )
        }
        open={live && policyOpen}
      />

      <Dialog onOpenChange={(next) => !next && setConfirm(null)} open={live && confirm !== null}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{confirm?.kind === "stop" ? "Stop Cycle" : "Clear Cycle"}</DialogTitle>
            <DialogDescription>
              {confirm?.kind === "stop"
                ? `Stopping the cycle stops every running and queued execution of its ${rows.length} cases. To stop only one execution, open it and use Stop Execution.`
                : confirm?.kind === "clear"
                  ? `Clearing stops this cycle and removes its ${rows.length} cases, every attempt, and the evidence they produced from the loaded runtime. Saved bundles and written reports are not affected.`
                  : ""}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setConfirm(null)} variant="outline">
              Cancel
            </Button>
            {confirm?.kind === "stop" ? (
              <Button
                onClick={() => {
                  setConfirm(null);
                  run({ done: "Cycle stopping", failed: "The cycle could not be stopped." }, () =>
                    commands.stopCycle.mutateAsync(),
                  );
                }}
                variant="danger"
              >
                Stop Cycle
              </Button>
            ) : (
              <Button
                onClick={() => {
                  setConfirm(null);
                  run(
                    { done: "Cycle cleared", failed: "The cycle could not be cleared." },
                    async () => {
                      await commands.clearCycle.mutateAsync();
                      await navigate({ to: "/cycles" });
                    },
                  );
                }}
                variant="danger"
              >
                Clear Cycle
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/* === Private Components === */

function RuntimePolicyDialog({
  continueOnFailure,
  onOpenChange,
  onSubmit,
  open,
}: {
  continueOnFailure: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (continueOnFailure: boolean) => void;
  open: boolean;
}) {
  const [value, setValue] = useState(continueOnFailure);

  useEffect(() => {
    if (open) setValue(continueOnFailure);
  }, [continueOnFailure, open]);

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Runtime Policy</DialogTitle>
          <DialogDescription>How this cycle reacts when a case fails.</DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4">
          <Label htmlFor="cycle-continue-on-failure">Continue after a failed case</Label>
          <Switch checked={value} id="cycle-continue-on-failure" onCheckedChange={setValue} />
        </div>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} variant="outline">
            Cancel
          </Button>
          <Button onClick={() => onSubmit(value)}>Save Policy</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function WorkspaceNotice({
  action,
  message,
  title,
}: {
  action?: { label: string; onClick: () => void };
  message: string;
  title: string;
}) {
  return (
    <div className="grid justify-items-center gap-3 px-4 py-16 text-center">
      <h1 className="font-heading text-lg font-semibold">{title}</h1>
      <p className="max-w-prose text-sm text-muted-foreground">{message}</p>
      {action ? (
        <Button onClick={action.onClick} variant="outline">
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}
