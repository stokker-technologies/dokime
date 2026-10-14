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
 * Two regions that never change shape as a cycle runs. The execution cockpit,
 * full width on top, states what the cycle is and holds every run control —
 * the cycle's and the selected execution's — and the operator prompts. The
 * document below it shows what the selection holds, with only its own plot
 * window and Focus on its toolbar. The sidebar tree owns selection. The route owns the URL, so every selection change leaves here through
 * `onSelectionChange`.
 *
 * `live` is the only difference between the application and the frozen report,
 * which mounts this same workspace over a snapshot. With it off the header
 * keeps identity, status and progress and loses every command and every prompt, and the reads answer from the snapshot context rather than
 * the backend. There is no second workspace.
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { Maximize2, Minimize2 } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
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
import type { TestExecution } from "@/lib/types";
import { useFocusMode } from "@/shell/focus-mode";
import { usePageChrome } from "@/shell/page-chrome";
import { useSelectionStore } from "@/stores/selection-store";
import { CycleEvidencePane } from "./CycleEvidencePane";
import { CycleExecutionControls } from "./CycleExecutionControls";
import { CyclePromptBanner } from "./CyclePromptBanner";
import { CycleReportButton } from "./CycleReportButton";
import { CycleRunControls } from "./CycleRunControls";
import { CycleWorkspaceHeader } from "./CycleWorkspaceHeader";
import {
  useCycleCommands,
  useCycleDetailQuery,
  useCyclePendingPromptsQuery,
  useCycleRuntimeShellQuery,
  useRerunCase,
} from "./cycle-queries";
import {
  buildCycleCases,
  buildCycleExecutionRows,
  type CycleExecutionRow,
  type CycleSelection,
  caseActionTooltip,
  caseStatusLabel,
  caseStatusVariant,
  cycleStatusLabel,
  cycleStatusVariant,
  executionStatusLabel,
  executionStatusVariant,
  finishedCaseCount,
  followSelection,
  isActiveExecutionStatus,
  isCycleTerminal,
  isTerminalExecutionStatus,
  progressStepLabel,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { answerablePrompts } from "./prompt-banner-model";
import { useExecutionDetailQuery } from "./server-state";

/* === Types === */

export type CycleWorkspaceProps = {
  cycleId: string;
  caseId?: string;
  executionId?: string;
  /** `"cycle"` when the operator selected the cycle itself. */
  view?: "cycle";
  /** False in the frozen report: evidence without commands. */
  live?: boolean;
  onSelectionChange: (next: CycleSelection) => void;
};

type PendingConfirm = { kind: "clear" } | { kind: "stop" } | null;

/* === Components === */

export function CycleWorkspace({
  cycleId,
  caseId,
  executionId,
  view,
  live = true,
  onSelectionChange,
}: CycleWorkspaceProps) {
  const navigate = useNavigate();
  const detailQuery = useCycleDetailQuery(cycleId);
  const runtimeQuery = useCycleRuntimeShellQuery(cycleId);
  const promptsQuery = useCyclePendingPromptsQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const focus = useFocusMode();
  const followActive = useSelectionStore((state) => state.followActive);
  const setSelectionState = useSelectionStore((state) => state.setSelectionState);
  const [policyOpen, setPolicyOpen] = useState(false);
  const [confirm, setConfirm] = useState<PendingConfirm>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const cases = useMemo(() => buildCycleCases(plan), [plan]);
  const rows = useMemo(() => buildCycleExecutionRows(cases, executions), [cases, executions]);
  const selection = useMemo<CycleSelection>(
    () => ({ caseId, executionId, view }),
    [caseId, executionId, view],
  );
  const selectedRow = selectedCycleRow(rows, selection);
  const finished = finishedCaseCount(cases);
  const status = plan?.test_cycle.status ?? "planned";
  const activeExecutionId =
    plan?.test_cycle.current_test_execution_id ??
    executions.find((execution) => execution.status === "running")?.test_execution_id ??
    null;
  const progress = useExecutionProgress(activeExecutionId);
  const selectedDetail = useExecutionDetailQuery(
    selectedRow?.execution?.test_execution_id ?? null,
    Boolean(selectedRow?.execution),
  );
  /* Only prompts whose owner still runs are offered: a prompt never outlives
     its execution, here as in the runtime that settles it. */
  const prompts = useMemo(() => {
    if (!live) return [];
    const statuses = new Map(
      executions.map((execution) => [execution.test_execution_id, execution.status]),
    );
    return answerablePrompts(promptsQuery.data ?? [], {
      cycleEnded: isCycleTerminal(status),
      executionActive: (id) => {
        const executionStatus = statuses.get(id);
        return executionStatus !== undefined && isActiveExecutionStatus(executionStatus);
      },
    });
  }, [executions, live, promptsQuery.data, status]);
  const activity = useMemo(() => {
    const activeRow = rows.find(
      (row) => row.execution?.test_execution_id === progress?.testExecutionId,
    );
    if (!progress || !activeRow) return null;
    const step = progressStepLabel(progress.stepId, progress.phase);
    return `${activeRow.case.label} · ${step} ${progress.completed}/${progress.total}`;
  }, [progress, rows]);

  /* The page bar's action slot carries the run controls of what is
     selected — the cycle's, or the selected execution's — and Focus beside
     them. The node is memoized on identities, so it is published only when
     the selection, focus or liveness changes. */
  const executionSelected = selectedRow !== null;
  /* The node whose report Download Report saves: the selected execution, or
     the cycle. A case that has not run yet has no report of its own. */
  const selectedExecutionId = selectedRow?.execution?.test_execution_id ?? null;
  const selectedLabel = selectedRow
    ? `Execution ${selectedRow.ordinal} · ${selectedRow.case.label}`
    : null;
  const cycleTitle = plan?.test_cycle.title ?? null;
  const reportSubject = useMemo(() => {
    if (selectedLabel !== null) {
      return selectedExecutionId
        ? { executionId: selectedExecutionId, label: selectedLabel }
        : null;
    }
    return cycleTitle !== null ? { executionId: null, label: cycleTitle } : null;
  }, [cycleTitle, selectedExecutionId, selectedLabel]);
  const toggleFocus = focus.setFocused;
  const focused = focus.focused;
  const pageActions = useMemo(
    () =>
      live ? (
        <div className="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]">
          {executionSelected ? (
            <CycleExecutionControls
              cycleId={cycleId}
              onSelectExecution={(next) => onSelectionChange({ executionId: next })}
              selection={selection}
            />
          ) : (
            <CycleRunControls cycleId={cycleId} onStop={() => setConfirm({ kind: "stop" })} />
          )}
          {reportSubject ? (
            <CycleReportButton
              cycleId={cycleId}
              executionId={reportSubject.executionId}
              label={reportSubject.label}
            />
          ) : null}
          <Button
            aria-label={focused ? "Leave Focus" : "Focus"}
            aria-pressed={focused}
            className="size-[var(--bar-control-height)]"
            onClick={() => toggleFocus(!focused)}
            size="icon"
            title={focused ? "Leave Focus (Escape)" : "Focus"}
            variant={focused ? "secondary" : "outline"}
          >
            {focused ? <Minimize2 /> : <Maximize2 />}
          </Button>
        </div>
      ) : null,
    [
      cycleId,
      executionSelected,
      focused,
      live,
      onSelectionChange,
      reportSubject,
      selection,
      toggleFocus,
    ],
  );
  /* The page bar names the selected node as a breadcrumb; Back from an
     execution returns to the cycle (see `useParentRoute`). */
  usePageChrome({
    actions: pageActions,
    title: selectedRow ? `Cycle › Execution ${selectedRow.ordinal}` : "Cycle",
  });

  /* Follow Active selects the execution the runtime names, until the
     operator chooses something else; `followSelection` owns the rule. */
  const previousActive = useRef<string | null>(null);
  const previousSelectedStatus = useRef<{ id?: string; ended: boolean }>({ ended: false });
  const selectedStatus = selectedRow?.execution?.status;
  useEffect(() => {
    const ended = selectedStatus !== undefined && isTerminalExecutionStatus(selectedStatus);
    const earlier = previousSelectedStatus.current;
    const selectedEnded = earlier.id === executionId && !earlier.ended && ended;
    previousSelectedStatus.current = { id: executionId, ended };
    const target =
      live && followActive
        ? followSelection({
            active: activeExecutionId,
            previousActive: previousActive.current,
            selectedEnded,
            selection,
          })
        : null;
    previousActive.current = activeExecutionId;
    if (target) onSelectionChange({ executionId: target });
  }, [
    activeExecutionId,
    executionId,
    followActive,
    live,
    onSelectionChange,
    selectedStatus,
    selection,
  ]);

  /* A command reports what happened, not what was attempted: the operator
     needs the outcome and, when it failed, what the backend said. */
  const openReport = (executionId: string | null) => {
    commands.openReport
      .mutateAsync(executionId)
      .then((origin) =>
        toast.success("Report Opened", {
          description:
            origin === "Current"
              ? "Generated from the run so far."
              : origin === "Regenerated"
                ? "The sealed report was missing, so it was generated again from its evidence."
                : undefined,
        }),
      )
      .catch((error: unknown) =>
        toast.error("The report could not be opened.", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };
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
    <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
      <CycleWorkspaceHeader
        identity={
          selectedRow
            ? cockpitSubject(selectedRow, selectedDetail.data ?? null)
            : {
                activity,
                progress: `${finished} of ${cases.length} cases finished`,
                statusLabel: cycleStatusLabel(status),
                statusVariant: cycleStatusVariant(status),
                title: plan.test_cycle.title,
              }
        }
        live={live}
        menu={
          selectedRow
            ? {
                kind: "execution",
                onOpenReport: selectedRow.execution
                  ? () => openReport(selectedRow.execution?.test_execution_id ?? null)
                  : null,
                onRerun:
                  selectedRow.execution && isTerminalExecutionStatus(selectedRow.execution.status)
                    ? () =>
                        run(
                          { done: "Case Re-Run Started", failed: "Case Re-Run Failed" },
                          async () => {
                            const created = await rerunCase.rerun(selectedRow.case.caseId);
                            if (created) onSelectionChange({ executionId: created });
                          },
                        )
                    : null,
                rerunRefusal:
                  selectedRow.case.actions && !selectedRow.case.actions.schedule_rerun
                    ? caseActionTooltip(selectedRow.case)
                    : undefined,
              }
            : {
                followActive,
                kind: "cycle",
                onAddCases: () => void navigate({ to: "/library", search: { appendTo: cycleId } }),
                onClearCycle: () => setConfirm({ kind: "clear" }),
                onCopyCycle: () =>
                  run(
                    { done: "Cycle copied", failed: "The cycle could not be copied." },
                    async () => {
                      const response = await commands.copyCycle.mutateAsync();
                      await navigate({
                        to: "/cycles/$cycleId",
                        params: { cycleId: response.plan.test_cycle.test_cycle_id },
                      });
                    },
                  ),
                onOpenReport: () => openReport(null),
                onRuntimePolicy: () => setPolicyOpen(true),
                onToggleFollowActive: () => setSelectionState({ followActive: !followActive }),
              }
        }
        prompts={<CyclePromptBanner prompts={prompts} />}
      />

      <CycleEvidencePane
        cycleId={cycleId}
        live={live}
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
                ? `Stopping the cycle stops every running and queued execution of its ${cases.length} cases. To stop only one execution, select it and use Stop Execution.`
                : confirm?.kind === "clear"
                  ? `Clearing stops this cycle and removes its ${cases.length} cases, every attempt, and the evidence they produced from the loaded runtime. Saved reports are not affected.`
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

/* === Private Functions === */

/** The execution page's identity: its case, its status and its steps. */
function cockpitSubject(row: CycleExecutionRow, detail: TestExecution | null) {
  const execution = detail ?? row.execution;
  const title = `Execution ${row.ordinal} · ${row.case.label}`;
  if (!execution) {
    return {
      progress: "Not run yet",
      statusLabel: caseStatusLabel(row.case.status),
      statusVariant: caseStatusVariant(row.case.status),
      title,
    };
  }
  const steps = execution.steps ?? [];
  const done = steps.filter((step) => !["pending", "running"].includes(step.status)).length;
  return {
    progress: steps.length ? `${done} of ${steps.length} steps finished` : "No steps recorded",
    statusLabel: executionStatusLabel(execution.status),
    statusVariant: executionStatusVariant(execution.status),
    title,
  };
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
