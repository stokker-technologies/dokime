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
 * Two viewers share this workspace and stay separate components with their
 * own data hooks, composed from the same blocks in the same order: the shared
 * `NodeSummary` (breadcrumb, title with its pencil inline, status pill,
 * progress), the `ControlCard` with the node's lifecycle actions in its fixed
 * slot, the node's `PromptQueue`, the evidence, and the shared Records
 * section. The page bar carries the shared `NodeActionCluster` on both: the
 * primary lifecycle action, Export Evidence, Download Report, expand and the
 * "…" menu. The execution viewer is `ExecutionSections`; the cycle viewer is
 * composed here. A prompt is answered on the execution that asked it: the
 * cycle viewer only names the executions waiting on input and links to them,
 * and answers the prompts the cycle itself asked. The sidebar tree owns
 * selection. The route owns the URL, so every selection change leaves here
 * through `onSelectionChange`.
 *
 * The report supplies its captured data directly to the shared evidence and
 * execution presentation views. This wrapper owns live queries and commands;
 * frozen rendering does not construct those resources.
 */

/* === External Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { Link, useNavigate } from "@tanstack/react-router";
import { Archive, LockKeyhole, RotateCcw } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { TestExecution } from "@/lib/types";
import { useRerunNoticeStore } from "@/stores/rerun-notice-store";
import { useSelectionStore } from "@/stores/selection-store";
import { ControlCard } from "./ControlCard";
import { CycleEvidencePane } from "./CycleEvidencePane";
import { cycleControlState, inputNeededLabel, rerunNoticeLines } from "./control-model";
import {
  useCycleCommands,
  useCycleDetailQuery,
  useCycleLabelQuery,
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
  cycleState,
  type FollowState,
  finishedCaseCount,
  followSelection,
  isActiveExecutionStatus,
  isCycleArchivable,
  isTerminalExecutionStatus,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { ExecutionSections } from "./ExecutionSections";
import { executionPagePrompts } from "./execution-sections-model";
import { NodeActionCluster } from "./NodeActionCluster";
import type { ArchiveAction, CycleMenu, ExecutionMenu } from "./NodeMenu";
import { NodeSummary } from "./NodeSummary";
import { answerablePrompts } from "./prompt-banner-model";
import { cycleSettingsReadOnly, SettingsDialog } from "./SettingsForm";

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

type PendingConfirm = { kind: "seal" | "retry" | "archive" } | null;

/* What each confirmed cycle action says before it runs, and what it reports after. */
const CONFIRMS = {
  archive: {
    action: "Archive Cycle",
    description:
      "Archiving hides this cycle from the default lists. Its unsealed evidence and the recorded seal failure are kept, and Unarchive brings it back.",
    done: "Archived",
    failed: "The Archive Mark Could Not Be Set",
    pending: "Archiving",
    title: "Archive Cycle",
  },
  retry: {
    action: "Retry Seal",
    description:
      "Retrying seals this cycle again from its saved evidence. If the evidence still cannot be read, the cycle returns to Seal Failed with the new cause.",
    done: "Cycle Sealed",
    failed: "Cycle Seal Failed Again",
    pending: "Sealing — Saving Evidence",
    title: "Retry Seal",
  },
  seal: {
    action: "Seal Cycle",
    description:
      "Sealing closes this cycle permanently. Its running work stops, unstarted executions are cancelled, and all evidence is saved. Other cycles keep running. This can take time while evidence drains.",
    done: "Cycle Sealed",
    failed: "Cycle Seal Could Not Be Confirmed",
    pending: "Sealing — Saving Evidence",
    title: "Seal Cycle",
  },
} as const;

/* === Components === */

export function CycleWorkspace({ cycleId, caseId, executionId, view, live = true, onSelectionChange }: CycleWorkspaceProps) {
  const navigate = useNavigate();
  const detailQuery = useCycleDetailQuery(cycleId);
  const runtimeQuery = useCycleRuntimeShellQuery(cycleId);
  const promptsQuery = useCyclePendingPromptsQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const followActive = useSelectionStore((state) => state.followActive);
  const setSelectionState = useSelectionStore((state) => state.setSelectionState);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [confirm, setConfirm] = useState<PendingConfirm>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const cases = useMemo(() => buildCycleCases(plan), [plan]);
  const rows = useMemo(() => buildCycleExecutionRows(cases, executions), [cases, executions]);
  const selection = useMemo<CycleSelection>(() => ({ caseId, executionId, view }), [caseId, executionId, view]);
  const selectedRow = selectedCycleRow(rows, selection);
  const finished = finishedCaseCount(cases);
  const rerunOf = plan?.test_cycle.rerun_of ?? null;
  const rerunOfLabel = useCycleLabelQuery(rerunOf).data ?? rerunOf;
  const rerunNotices = useRerunNoticeStore((state) => state.notices[cycleId]);
  const setRerunNotices = useRerunNoticeStore((state) => state.setRerunNotices);
  const status = plan?.test_cycle.status ?? "planned";
  const phase = plan?.test_cycle.phase ?? "active";
  const activeExecutionId =
    plan?.test_cycle.current_test_execution_id ?? executions.find((execution) => execution.status === "running")?.test_execution_id ?? null;
  const progress = useExecutionProgress(activeExecutionId);
  /* Only prompts whose owner still runs are offered: a prompt never outlives
     its execution, here as in the runtime that settles it. */
  const prompts = useMemo(() => {
    if (!live) return [];
    const statuses = new Map(executions.map((execution) => [execution.test_execution_id, execution.status]));
    return answerablePrompts(promptsQuery.data ?? [], {
      cycleEnded: phase !== "active",
      executionActive: (id) => {
        const executionStatus = statuses.get(id);
        return executionStatus !== undefined && isActiveExecutionStatus(executionStatus);
      },
    });
  }, [executions, live, phase, promptsQuery.data]);
  const runningRow = rows.find((row) => row.execution !== null && row.execution.test_execution_id === progress?.testExecutionId) ?? null;
  /* The cycle viewer names where input is needed and links there; it answers
     only the prompts the cycle itself asked. */
  const waitingRows = useMemo(
    () =>
      rows.filter(
        (row) => row.execution !== null && prompts.some((prompt) => prompt.scope?.test_execution_id === row.execution?.test_execution_id),
      ),
    [prompts, rows],
  );
  const cyclePrompts = useMemo(() => executionPagePrompts(prompts, null), [prompts]);

  /* Follow Active selects the execution the runtime names, until the
     operator chooses something else; `followSelection` owns the rule. */
  const previousActive = useRef<string | null>(null);
  const previousSelectedStatus = useRef<FollowState["previousSelected"]>(null);
  const selectedStatus = selectedRow?.execution?.status;
  useEffect(() => {
    const earlier = previousSelectedStatus.current;
    previousSelectedStatus.current = { executionId, status: selectedStatus };
    const target =
      live && followActive
        ? followSelection({
            active: activeExecutionId,
            previousActive: previousActive.current,
            selectedStatus,
            previousSelected: earlier,
            selection,
          })
        : null;
    previousActive.current = activeExecutionId;
    if (target) onSelectionChange({ executionId: target });
  }, [activeExecutionId, executionId, followActive, live, onSelectionChange, selectedStatus, selection]);

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
      .catch((error: unknown) => toast.error(failureOf(error, "The Report Could Not Be Opened")));
  };
  /* Archive and Unarchive say what they did; neither changes the lifecycle. */
  const archiveAction = (
    archived: boolean,
    allowed: boolean,
    subject: "cycle" | "execution",
    commit: (next: boolean) => Promise<unknown>,
  ): ArchiveAction => ({
    archived,
    onToggle: () =>
      run(
        archived
          ? { done: "Unarchived", failed: "The Archive Mark Could Not Be Removed" }
          : { done: "Archived", failed: "The Archive Mark Could Not Be Set" },
        () => commit(!archived),
      ),
    refusal: archiveRefusal(allowed, subject),
  });
  const run = (report: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(report.done))
      .catch((error: unknown) => toast.error(failureOf(error, report.failed)));
  };

  if (detailQuery.isError && !plan) {
    return <ErrorCard failure={failureOf(detailQuery.error, "Cycle Unavailable")} onRetry={() => void detailQuery.refetch?.()} />;
  }

  if (!plan) {
    return detailQuery.isPending ? (
      <WorkspaceNotice message="Reading this cycle from the backend." title="Loading" />
    ) : (
      <ErrorCard
        failure={failureOf("This cycle could not be found on the connected backend.", "Cycle Not Found", `missing-cycle:${cycleId}`)}
        onRetry={() => void detailQuery.refetch()}
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
            "execution",
            (archived) =>
              commands.archiveExecution.mutateAsync({
                archived,
                executionId: selectedRow.execution?.test_execution_id ?? "",
              }),
          )
        : null,
      onOpenReport: selectedRow.execution ? () => openReport(selectedRow.execution?.test_execution_id ?? null) : null,
      onRerun:
        selectedRow.execution && isTerminalExecutionStatus(selectedRow.execution.status)
          ? () =>
              run({ done: "New Attempt Ready to Start", failed: "Case Re-Run Failed" }, async () => {
                const created = await rerunCase.rerun(selectedRow.case.caseId);
                if (created) onSelectionChange({ executionId: created });
              })
          : null,
      rerunRefusal: plan.test_cycle.phase === "active" ? undefined : "Only Active Cycles Accept New Attempts",
      onSettings: selectedRow.execution ? () => setSettingsOpen(true) : null,
    };
    const execution = selectedRow.execution;
    const executionSealed = caseSealed(executions, selectedRow.case.caseId);
    return (
      <div className="grid min-h-0">
        {live && execution ? (
          <SettingsDialog
            description="The execution's own settings. Its title is its case's label; a sealed attempt keeps the label its report froze."
            onOpenChange={setSettingsOpen}
            onSubmit={async (values) => {
              if (values.title !== selectedRow.case.label) {
                await commands.renameExecution.mutateAsync({ executionId: execution.test_execution_id, label: values.title });
              }
              toast.success("Execution Settings Saved");
            }}
            open={settingsOpen}
            readOnly={executionSealed ? { title: "Sealed: the report froze this label." } : {}}
            title="Execution Settings"
            values={{ title: selectedRow.case.label }}
          />
        ) : null}
        <ExecutionSections
          cycleId={cycleId}
          live={live}
          menu={executionMenu}
          onSelectExecution={(next) => onSelectionChange({ executionId: next })}
          plan={plan}
          prompts={prompts}
          row={selectedRow}
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

  const cycleMenu: CycleMenu = {
    archive: archiveAction(Boolean(plan.test_cycle.archived), isCycleArchivable(phase), "cycle", (archived) =>
      commands.archiveCycle.mutateAsync(archived),
    ),
    followActive,
    kind: "cycle",
    onAddCases: () => void navigate({ to: "/library", search: { appendTo: cycleId } }),
    onRerunCycle: () =>
      run({ done: "Cycle Rerun Planned", failed: "The Cycle Could Not Be Rerun" }, async () => {
        const response = await commands.rerunCycle.mutateAsync();
        setRerunNotices(response.cycleId, response);
        await navigate({ to: "/cycles/$cycleId", params: { cycleId: response.cycleId } });
      }),
    onOpenReport: () => openReport(null),
    onSettings: () => setSettingsOpen(true),
    onToggleFollowActive: () => setSelectionState({ followActive: !followActive }),
  };
  const control = cycleControlState({
    phase,
    running:
      runningRow?.execution && progress
        ? {
            ordinal: runningRow.ordinal,
            step: progress.label,
          }
        : null,
  });
  const seal = control.actions.includes("Seal Cycle") ? (
    <Button className="h-[var(--bar-control-height)] px-3" onClick={() => setConfirm({ kind: "seal" })} variant="outline">
      <LockKeyhole data-icon="inline-start" />
      Seal Cycle
    </Button>
  ) : control.actions.includes("Retry Seal") ? (
    <>
      <Button className="h-[var(--bar-control-height)] px-3" onClick={() => setConfirm({ kind: "retry" })} variant="outline">
        <RotateCcw data-icon="inline-start" />
        Retry Seal
      </Button>
      {control.actions.includes("Archive") && !plan.test_cycle.archived ? (
        <Button className="h-[var(--bar-control-height)] px-3" onClick={() => setConfirm({ kind: "archive" })} variant="outline">
          <Archive data-icon="inline-start" />
          Archive
        </Button>
      ) : null}
    </>
  ) : null;
  const confirming = confirm ? CONFIRMS[confirm.kind] : null;
  const confirmPending = commands.sealCycle.isPending || commands.archiveCycle.isPending;
  const waiting = inputNeededLabel(waitingRows.map((row) => row.ordinal));
  const state = cycleState(phase, status, prompts.length > 0);

  return (
    <div className="grid min-h-0">
      <CycleEvidencePane
        cycleId={cycleId}
        pageActions={
          live ? (
            <NodeActionCluster
              cycleId={cycleId}
              menu={cycleMenu}
              primary={seal}
              subject={{ executionId: null, label: plan.test_cycle.title, sealed: phase === "sealed" }}
            />
          ) : null
        }
        trail={["Cycles", plan.test_cycle.title]}
        lead={
          <>
            <NodeSummary
              description={
                typeof plan.test_cycle.metadata.description === "string" && plan.test_cycle.metadata.description
                  ? plan.test_cycle.metadata.description
                  : null
              }
              live={live}
              onRename={plan.test_cycle.sealed || plan.test_cycle.archived ? null : (label) => commands.renameCycle.mutateAsync(label)}
              progress={
                <>
                  {`${finished} of ${cases.length} cases finished`}
                  {rerunOf ? (
                    <>
                      {" · Rerun of "}
                      <Link className="underline" params={{ cycleId: rerunOf }} to="/cycles/$cycleId">
                        {rerunOfLabel}
                      </Link>
                    </>
                  ) : null}
                </>
              }
              status={state}
              subject="Cycle"
              title={plan.test_cycle.title}
            />
            {live ? (
              <ControlCard
                actions={seal}
                label="Cycle Controls"
                progress={runningRow && progress ? { completed: progress.completed, phase: progress.phase, total: progress.total } : null}
                state={control.line}
              >
                {phase === "seal_failed" && plan.test_cycle.seal_failure ? (
                  <ErrorCard
                    failure={failureOf(
                      plan.test_cycle.seal_failure,
                      "Cycle Finalization Failed",
                      [cycleId, "seal", plan.test_cycle.seal_failure].join(":"),
                    )}
                  />
                ) : null}
                {rerunNotices
                  ? rerunNoticeLines(rerunNotices).map((line) => (
                      <p className="text-muted-foreground text-sm" data-rerun-notice key={line}>
                        {line}
                      </p>
                    ))
                  : null}
                {waiting ? (
                  <p className="grid auto-cols-max grid-flow-col items-baseline gap-x-2 text-sm" data-input-needed>
                    <span className="text-warning">{waiting}</span>
                    {waitingRows.map((row) => (
                      <Button
                        className="h-auto px-0"
                        key={row.key}
                        onClick={() => row.execution && onSelectionChange({ executionId: row.execution.test_execution_id })}
                        variant="link"
                      >
                        Answer On #{row.ordinal}
                      </Button>
                    ))}
                  </p>
                ) : null}
              </ControlCard>
            ) : null}
          </>
        }
        live={live}
        plan={plan}
        prompts={cyclePrompts}
        snapshot={runtimeQuery.data ?? null}
      />

      {live ? (
        <SettingsDialog
          description="The cycle's own settings. Continuation starts the next unstarted execution after one completes, whatever its verdict."
          onOpenChange={setSettingsOpen}
          onSubmit={async (values) => {
            if (values.title !== plan.test_cycle.title) await commands.renameCycle.mutateAsync(values.title);
            if (values.continueAutomatically !== undefined && values.continueAutomatically !== plan.test_cycle.continue_automatically) {
              await commands.updatePolicy.mutateAsync({ continueAutomatically: values.continueAutomatically });
            }
            toast.success("Cycle Settings Saved");
          }}
          open={settingsOpen}
          readOnly={cycleSettingsReadOnly(phase, Boolean(plan.test_cycle.archived))}
          title="Cycle Settings"
          values={{
            continueAutomatically: plan.test_cycle.continue_automatically,
            description: typeof plan.test_cycle.metadata.description === "string" ? plan.test_cycle.metadata.description : "",
            title: plan.test_cycle.title,
          }}
        />
      ) : null}

      <Dialog onOpenChange={(next) => !next && !confirmPending && setConfirm(null)} open={live && confirming !== null}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{confirming?.title}</DialogTitle>
            <DialogDescription>{confirming?.description}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button disabled={confirmPending} onClick={() => setConfirm(null)} variant="outline">
              Cancel
            </Button>
            <Button
              disabled={confirmPending}
              onClick={() => {
                if (!confirm || !confirming) return;
                run({ done: confirming.done, failed: confirming.failed }, async () => {
                  try {
                    await (confirm.kind === "archive" ? commands.archiveCycle.mutateAsync(true) : commands.sealCycle.mutateAsync());
                  } finally {
                    setConfirm(null);
                  }
                });
              }}
              variant="danger"
            >
              {confirmPending ? confirming?.pending : confirming?.action}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/* === Private Functions === */

/** A case whose attempt sealed keeps the label its report froze. */
function caseSealed(executions: TestExecution[], caseId: string): boolean {
  return executions.some((execution) => execution.case_id === caseId && execution.sealed);
}

function executionTitlePrefix(row: CycleExecutionRow): string {
  return `Execution ${row.ordinal} · `;
}

/* === Private Components === */

function WorkspaceNotice({ action, message, title }: { action?: { label: string; onClick: () => void }; message: string; title: string }) {
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
