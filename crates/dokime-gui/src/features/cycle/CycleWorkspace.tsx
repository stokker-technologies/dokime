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
 * `NodeHeader` (breadcrumb, title with its pencil inline, status pill,
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

import { useNavigate } from "@tanstack/react-router";
import { LockKeyhole } from "lucide-react";
import { type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { TestExecution } from "@/lib/types";
import { usePageChrome } from "@/shell/page-chrome";
import { useSelectionStore } from "@/stores/selection-store";
import { ControlCard } from "./ControlCard";
import { CycleEvidencePane } from "./CycleEvidencePane";
import { cycleControlState, inputNeededLabel } from "./control-model";
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
  cycleState,
  finishedCaseCount,
  followSelection,
  isActiveExecutionStatus,
  isCycleSealed,
  isTerminalExecutionStatus,
  selectedCycleRow,
} from "./cycle-workspace-model";
import { stepLabel } from "./ExecutionControl";
import { ExecutionSections } from "./ExecutionSections";
import { executionPagePrompts } from "./execution-sections-model";
import { NodeActionCluster } from "./NodeActionCluster";
import { NodeHeader } from "./NodeHeader";
import type { ArchiveAction, CycleMenu, ExecutionMenu } from "./NodeMenu";
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

type PendingConfirm = { kind: "seal" } | null;

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
      .catch((error: unknown) =>
        toast.error("The report could not be opened.", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };
  /* Archive and Unarchive say what they did; neither changes the lifecycle. */
  const archiveAction = (archived: boolean, terminal: boolean, commit: (next: boolean) => Promise<unknown>): ArchiveAction => ({
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
        message={detailQuery.error instanceof Error ? detailQuery.error.message : "The cycle could not be read from the backend."}
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
        message="This cycle could not be found on the connected backend. Return to Cycles or inspect its saved history."
        title="Cycle Not Found"
      />
    );
  }

  if (selectedRow) {
    const executionMenu: ExecutionMenu = {
      kind: "execution",
      archive: selectedRow.execution
        ? archiveAction(Boolean(selectedRow.execution.archived), isTerminalExecutionStatus(selectedRow.execution.status), (archived) =>
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
              run({ done: "New Attempt Ready To Start", failed: "Case Re-Run Failed" }, async () => {
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
          onSelectCycle={() => onSelectionChange({ view: "cycle" })}
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
    archive: archiveAction(Boolean(plan.test_cycle.archived), isCycleSealed(phase), (archived) =>
      commands.archiveCycle.mutateAsync(archived),
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
    onSettings: () => setSettingsOpen(true),
    onToggleFollowActive: () => setSelectionState({ followActive: !followActive }),
  };
  const control = cycleControlState({
    phase,
    running:
      runningRow?.execution && progress
        ? {
            ordinal: runningRow.ordinal,
            step: stepLabel(runningRow.execution, progress.stepId, progress.phase),
            stepId: progress.stepId,
          }
        : null,
  });
  const seal = control.actions.includes("Seal Cycle") ? (
    <Button className="h-[var(--bar-control-height)] px-3" onClick={() => setConfirm({ kind: "seal" })} variant="outline">
      <LockKeyhole data-icon="inline-start" />
      Seal Cycle
    </Button>
  ) : null;
  const waiting = inputNeededLabel(waitingRows.map((row) => row.ordinal));
  const state = cycleState(phase, status, prompts.length > 0);

  return (
    <div className="grid min-h-0">
      {live ? (
        <CyclePageChrome>
          <NodeActionCluster
            cycleId={cycleId}
            menu={cycleMenu}
            primary={seal}
            subject={{ executionId: null, label: plan.test_cycle.title, sealed: phase === "sealed" }}
          />
        </CyclePageChrome>
      ) : null}
      <CycleEvidencePane
        cycleId={cycleId}
        lead={
          <>
            <NodeHeader
              breadcrumb={[{ label: "Cycles", onSelect: () => void navigate({ to: "/cycles" }) }, { label: plan.test_cycle.title }]}
              description={
                typeof plan.test_cycle.metadata.description === "string" && plan.test_cycle.metadata.description
                  ? plan.test_cycle.metadata.description
                  : null
              }
              live={live}
              onRename={plan.test_cycle.sealed || plan.test_cycle.archived ? null : (label) => commands.renameCycle.mutateAsync(label)}
              progress={`${finished} of ${cases.length} cases finished`}
              status={state}
              subject="Cycle"
              title={plan.test_cycle.title}
            />
            {live ? (
              <ControlCard
                actions={seal}
                detail={control.detail}
                label="Cycle Controls"
                progress={runningRow && progress ? { completed: progress.completed, total: progress.total } : null}
                state={control.line}
              >
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

      <Dialog onOpenChange={(next) => !next && !commands.sealCycle.isPending && setConfirm(null)} open={live && confirm !== null}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Seal Cycle</DialogTitle>
            <DialogDescription>
              Sealing closes this cycle permanently. Its running work stops, unstarted executions are cancelled, and all evidence is saved.
              Other cycles keep running. This can take time while evidence drains.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button disabled={commands.sealCycle.isPending} onClick={() => setConfirm(null)} variant="outline">
              Cancel
            </Button>
            <Button
              disabled={commands.sealCycle.isPending}
              onClick={() => {
                run({ done: "Cycle Sealed", failed: "Cycle Seal Could Not Be Confirmed" }, async () => {
                  await commands.sealCycle.mutateAsync();
                  setConfirm(null);
                });
              }}
              variant="danger"
            >
              {commands.sealCycle.isPending ? "Sealing — Saving Evidence" : "Seal Cycle"}
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

/** Publishes the cycle viewer's page bar; mounted only while the cycle itself is shown. */
function CyclePageChrome({ children }: { children: ReactNode }) {
  usePageChrome({ actions: children, title: "Cycle" });
  return null;
}

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
