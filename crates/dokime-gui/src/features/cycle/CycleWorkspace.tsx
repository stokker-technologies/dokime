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
 * Two pages share this workspace. The cycle page keeps its cockpit full width
 * on top — identity, status, progress, the cycle's menu and its prompts — over
 * the cycle document. The execution page (`ExecutionSections`) is one
 * scrolling column of sections under the execution's title: its prompts, step
 * progress and run controls lead it in Control. The page bar carries the
 * cycle's run controls on the cycle page, and Download Report and Focus on
 * both. The sidebar tree owns selection. The route owns the URL, so every
 * selection change leaves here through `onSelectionChange`.
 *
 * The report supplies its captured data directly to the shared evidence and
 * execution presentation views. This wrapper owns live queries and commands;
 * frozen rendering does not construct those resources.
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
import { focusPresentation, useFocusMode } from "@/shell/focus-mode";
import { usePageChrome } from "@/shell/page-chrome";
import { useSelectionStore } from "@/stores/selection-store";
import { CycleEvidencePane } from "./CycleEvidencePane";
import { CyclePromptBanner } from "./CyclePromptBanner";
import { CycleReportButton } from "./CycleReportButton";
import { CycleRunControls } from "./CycleRunControls";
import {
  type ArchiveAction,
  CycleWorkspaceHeader,
  type ExecutionMenu,
} from "./CycleWorkspaceHeader";
import {
  useCycleCommands,
  useCycleDetailQuery,
  useCyclePendingPromptsQuery,
  useCycleRuntimeShellQuery,
  useRerunCase,
} from "./cycle-queries";
import {
  archiveRefusal,
  buildCycleCases,
  buildCycleExecutionRows,
  type CycleExecutionRow,
  type CycleSelection,
  caseActionTooltip,
  cycleStatusLabel,
  cycleStatusVariant,
  finishedCaseCount,
  followSelection,
  isActiveExecutionStatus,
  isCycleTerminal,
  isTerminalExecutionStatus,
  progressStepLabel,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { ExecutionSections } from "./ExecutionSections";
import { answerablePrompts } from "./prompt-banner-model";

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

type PendingConfirm = { kind: "stop" } | null;

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

  /* The page bar's action slot carries the cycle's run controls on the cycle
     page — the execution page holds its own in Control — with Download Report
     and Focus beside them. The node is memoized on identities, so it is published only when
     the selection, focus or liveness changes. */
  const executionSelected = selectedRow !== null;
  /* The node whose report Download Report saves: the selected execution, or
     the cycle. A case that has not run yet has no report of its own. */
  const selectedExecutionId = selectedRow?.execution?.test_execution_id ?? null;
  const selectedLabel = selectedRow ? executionTitle(selectedRow) : null;
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
  const focusLabel = focusPresentation(focused).toggleLabel;
  const pageActions = useMemo(
    () =>
      live ? (
        <div className="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]">
          {executionSelected ? null : (
            <CycleRunControls cycleId={cycleId} onStop={() => setConfirm({ kind: "stop" })} />
          )}
          {reportSubject ? (
            <CycleReportButton
              cycleId={cycleId}
              executionId={reportSubject.executionId}
              label={reportSubject.label}
            />
          ) : null}
          {/* Focus names its exit in text: the bar is the only chrome left,
              and Escape is a shortcut, not the way out. */}
          <Button
            aria-label={focusLabel}
            aria-pressed={focused}
            className={focused ? undefined : "size-[var(--bar-control-height)]"}
            onClick={() => toggleFocus(!focused)}
            size={focused ? "default" : "icon"}
            title={focused ? `${focusLabel} (Escape)` : focusLabel}
            variant={focused ? "secondary" : "outline"}
          >
            {focused ? (
              <>
                <Minimize2 />
                {focusLabel}
              </>
            ) : (
              <Maximize2 />
            )}
          </Button>
        </div>
      ) : null,
    [cycleId, executionSelected, focusLabel, focused, live, reportSubject, toggleFocus],
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
          description: origin === "Current" ? "Generated from the run so far." : undefined,
        }),
      )
      .catch((error: unknown) =>
        toast.error("The report could not be opened.", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };
  /* Archive and Unarchive say what they did; neither changes the lifecycle. */
  const archiveAction = (
    archived: boolean,
    terminal: boolean,
    commit: (next: boolean) => Promise<unknown>,
  ): ArchiveAction => ({
    archived,
    onToggle: () =>
      run(
        archived
          ? { done: "Unarchived", failed: "The archive mark could not be removed." }
          : { done: "Archived", failed: "The archive mark could not be set." },
        () => commit(!archived),
      ),
    refusal: archiveRefusal(terminal),
  });
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
        message="This cycle is no longer loaded. A sealed cycle leaves the runtime when the next one starts; History keeps it."
        title="Cycle Not Found"
      />
    );
  }

  if (selectedRow) {
    const executionMenu: ExecutionMenu = {
      kind: "execution",
      archive: selectedRow.execution
        ? archiveAction(
            Boolean(selectedRow.execution.archived),
            isTerminalExecutionStatus(selectedRow.execution.status),
            (archived) =>
              commands.archiveExecution.mutateAsync({
                archived,
                executionId: selectedRow.execution?.test_execution_id ?? "",
              }),
          )
        : null,
      onOpenReport: selectedRow.execution
        ? () => openReport(selectedRow.execution?.test_execution_id ?? null)
        : null,
      onRerun:
        selectedRow.execution && isTerminalExecutionStatus(selectedRow.execution.status)
          ? () =>
              run({ done: "Case Re-Run Started", failed: "Case Re-Run Failed" }, async () => {
                const created = await rerunCase.rerun(selectedRow.case.caseId);
                if (created) onSelectionChange({ executionId: created });
              })
          : null,
      rerunRefusal:
        selectedRow.case.actions && !selectedRow.case.actions.schedule_rerun
          ? caseActionTooltip(selectedRow.case)
          : undefined,
    };
    return (
      <div className="grid min-h-0">
        <ExecutionSections
          cycleId={cycleId}
          live={live}
          menu={executionMenu}
          onSelectExecution={(next) => onSelectionChange({ executionId: next })}
          plan={plan}
          prompts={prompts}
          row={selectedRow}
          selection={selection}
          label={selectedRow.case.label}
          onRename={
            selectedRow.execution && !caseSealed(executions, selectedRow.case.caseId)
              ? (label) =>
                  commands.renameExecution.mutateAsync({
                    executionId: selectedRow.execution?.test_execution_id ?? "",
                    label,
                  })
              : null
          }
          snapshot={runtimeQuery.data ?? null}
          titlePrefix={executionTitlePrefix(selectedRow)}
        />
      </div>
    );
  }

  return (
    <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
      <CycleWorkspaceHeader
        identity={{
          status: {
            activity,
            label: cycleStatusLabel(status),
            progress: `${finished} of ${cases.length} cases finished`,
            variant: cycleStatusVariant(status),
          },
          title: plan.test_cycle.title,
        }}
        live={live}
        onRename={
          plan.test_cycle.sealed || plan.test_cycle.archived
            ? undefined
            : (label) => commands.renameCycle.mutateAsync(label)
        }
        menu={{
          archive: archiveAction(
            Boolean(plan.test_cycle.archived),
            isCycleTerminal(status),
            (archived) => commands.archiveCycle.mutateAsync(archived),
          ),
          followActive,
          kind: "cycle",
          onAddCases: () => void navigate({ to: "/library", search: { appendTo: cycleId } }),
          onCopyCycle: () =>
            run({ done: "Cycle copied", failed: "The cycle could not be copied." }, async () => {
              const response = await commands.copyCycle.mutateAsync();
              await navigate({
                to: "/cycles/$cycleId",
                params: { cycleId: response.plan.test_cycle.test_cycle_id },
              });
            }),
          onOpenReport: () => openReport(null),
          onRuntimePolicy: () => setPolicyOpen(true),
          onToggleFollowActive: () => setSelectionState({ followActive: !followActive }),
        }}
        prompts={<CyclePromptBanner prompts={prompts} />}
      />

      <CycleEvidencePane
        cycleId={cycleId}
        live={live}
        plan={plan}
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
            <DialogTitle>Stop Cycle</DialogTitle>
            <DialogDescription>
              {`Stopping the cycle stops every running and queued execution of its ${cases.length} cases. To stop only one execution, select it and use Stop Execution.`}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setConfirm(null)} variant="outline">
              Cancel
            </Button>
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
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/* === Private Functions === */

/** The execution page's title: its ordinal and its case. */
function executionTitle(row: CycleExecutionRow): string {
  return `${executionTitlePrefix(row)}${row.case.label}`;
}

/** A case whose attempt sealed keeps the label its report froze. */
function caseSealed(executions: TestExecution[], caseId: string): boolean {
  return executions.some((execution) => execution.case_id === caseId && execution.sealed);
}

function executionTitlePrefix(row: CycleExecutionRow): string {
  return `Execution ${row.ordinal} · `;
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
