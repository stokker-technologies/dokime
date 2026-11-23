<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { FocusMode } from "../../shell/focus-mode";
  import type { CycleSelection } from "./cycle-workspace-model";
  import type { LiveCycleContext } from "./LiveCycleDocument.svelte";

  export type CycleWorkspaceProps = {
    cycleId: string;
    context: LiveCycleContext;
    focus: FocusMode;
    selection: CycleSelection;
    onSelectionChange: (next: CycleSelection) => void;
    openCycle: (cycleId: string) => void;
    addCases: (cycleId: string) => void;
  };
</script>

<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { Dialog } from "bits-ui";
  import { Archive, LockKeyhole, RotateCcw } from "lucide-svelte";
  import { createExecutionProgress } from "../../app/live/execution-progress.svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { useRerunNoticeStore } from "../../stores/rerun-notice-store";
  import { useSelectionStore } from "../../stores/selection-store";
  import { storeState } from "../../stores/store-state";
  import ControlCard from "./ControlCard.svelte";
  import { cycleControlState, inputNeededLabel, rerunNoticeLines } from "./control-model";
  import {
    createCycleCommands,
    cycleDetailQueryOptions,
    cycleLabelQueryOptions,
    cyclePromptQueryOptions,
    cycleRuntimeQueryOptions,
  } from "./cycle-queries";
  import {
    archiveRefusal,
    buildCycleCases,
    buildCycleExecutionRows,
    type CycleExecutionRow,
    cycleState,
    cycleReportSealed,
    type FollowState,
    sealedCaseCount,
    followSelection,
    isActiveExecutionStatus,
    isCycleArchivable,
    isTerminalExecutionStatus,
    selectedCycleRow,
  } from "./cycle-workspace-model";
  import CycleEvidencePane from "./CycleEvidencePane.svelte";
  import ExecutionSections from "./ExecutionSections.svelte";
  import { executionPagePrompts } from "./execution-sections-model";
  import NodeActionCluster from "./NodeActionCluster.svelte";
  import type { ArchiveAction, CycleMenu, ExecutionMenu } from "./node-menu";
  import NodeSummary from "./NodeSummary.svelte";
  import { answerablePrompts } from "./prompt-banner-model";
  import { cycleSettingsReadOnly } from "./settings-fields";
  import SettingsDialog from "./SettingsDialog.svelte";

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
        "Sealing closes this cycle permanently and retains its report and complete evidence archive. Never-started executions are sealed as not run. Wait for any running execution to seal before continuing.",
      done: "Cycle Sealed",
      failed: "Cycle Seal Could Not Be Confirmed",
      pending: "Sealing — Saving Evidence",
      title: "Seal Cycle",
    },
  } as const;

  let { cycleId, context, focus, selection, onSelectionChange, openCycle, addCases }: CycleWorkspaceProps = $props();
  let confirm = $state<keyof typeof CONFIRMS | null>(null);
  let deleteOpen = $state(false);
  let settingsOpen = $state(false);
  const session = () => context.session;
  const client = () => session().queryClient;
  const detail = createQuery(() => cycleDetailQueryOptions(session().identity, cycleId), client);
  const runtime = createQuery(() => cycleRuntimeQueryOptions(session().identity, cycleId), client);
  const pendingPrompts = createQuery(() => cyclePromptQueryOptions(session().identity, cycleId), client);
  const commands = createCycleCommands(session, () => cycleId);
  const follow = storeState(useSelectionStore, (state) => state.followActive);
  const notices = storeState(useRerunNoticeStore, (state) => state.notices);
  const plan = $derived(detail.data?.plan ?? null);
  const executions = $derived(detail.data?.executions ?? []);
  const cases = $derived(buildCycleCases(plan));
  const rows = $derived(buildCycleExecutionRows(cases, executions));
  const selected = $derived(selectedCycleRow(rows, selection));
  const sealed = $derived(sealedCaseCount(cases));
  const rerunOf = $derived(plan?.test_cycle.rerun_of ?? null);
  const rerunLabel = createQuery(() => cycleLabelQueryOptions(session().identity, rerunOf), client);
  const rerunNotices = $derived(notices()[cycleId]);
  const status = $derived(plan?.test_cycle.status ?? "planned");
  const phase = $derived(plan?.test_cycle.phase ?? "active");
  const activeId = $derived(
    plan?.test_cycle.current_test_execution_id ?? executions.find((execution) => execution.status === "running")?.test_execution_id ?? null,
  );
  const activeProgress = createExecutionProgress(session, () => activeId);
  const progress = $derived(activeProgress());
  const prompts = $derived.by(() => {
    const statuses = new Map(executions.map((execution) => [execution.test_execution_id, execution.status]));
    return answerablePrompts(pendingPrompts.data ?? [], {
      cycleUnavailable: phase !== "active",
      executionActive: (id) => {
        const executionStatus = statuses.get(id);
        return executionStatus !== undefined && isActiveExecutionStatus(executionStatus);
      },
    });
  });
  const runningRow = $derived(rows.find((row) => row.execution?.test_execution_id === progress?.testExecutionId && row.execution) ?? null);
  const waitingRows = $derived(
    rows.filter(
      (row) => row.execution !== null && prompts.some((prompt) => prompt.scope?.executionId === row.execution?.test_execution_id),
    ),
  );
  const waiting = $derived(inputNeededLabel(waitingRows.map((row) => row.ordinal)));
  const cyclePrompts = $derived(executionPagePrompts(prompts, null));
  const cycleStatus = $derived(cycleState(phase, status, prompts.length > 0));
  const control = $derived(
    cycleControlState({
      phase,
      running: runningRow?.execution && progress ? { ordinal: runningRow.ordinal, step: progress.label } : null,
    }),
  );
  const selectedStatus = $derived(selected?.execution?.status);
  const snapshot = $derived(runtime.data ?? null);
  const confirming = $derived(confirm ? CONFIRMS[confirm] : null);
  const confirmPending = $derived(commands.sealCycle.isPending || commands.archiveCycle.isPending);
  let previousActive: string | null = null;
  let previousSelected: FollowState["previousSelected"] = null;

  $effect(() => {
    const earlier = previousSelected;
    previousSelected = { executionId: selection.executionId, status: selectedStatus };
    const target = follow()
      ? followSelection({
          active: activeId,
          previousActive,
          selectedStatus,
          previousSelected: earlier,
          selection,
        })
      : null;
    previousActive = activeId;
    if (target) {
      onSelectionChange({ executionId: target });
    }
  });

  function selectExecution(executionId: string) {
    onSelectionChange({ executionId });
  }

  function run(report: { done: string; failed: string }, action: () => Promise<unknown>) {
    void action()
      .then(() => toast.success(report.done))
      .catch((error: unknown) => toast.error(failureOf(error, report.failed)));
  }

  function archiveAction(
    archived: boolean,
    allowed: boolean,
    subject: "cycle" | "execution",
    commit: (next: boolean) => Promise<unknown>,
  ): ArchiveAction {
    return {
      archived,
      onToggle: () =>
        run(
          archived
            ? { done: "Unarchived", failed: "The Archive Mark Could Not Be Removed" }
            : { done: "Archived", failed: "The Archive Mark Could Not Be Set" },
          () => commit(!archived),
        ),
      refusal: archiveRefusal(allowed, subject),
    };
  }

  function executionMenu(row: CycleExecutionRow): ExecutionMenu {
    const execution = row.execution;
    const capability = row.case.actions;
    const id = execution?.test_execution_id;
    const canStart = execution?.controls?.canStart;
    const terminal = execution !== null && isTerminalExecutionStatus(execution.status);
    return {
      kind: "execution",
      onStart:
        id && canStart
          ? () => run({ done: "Execution Started", failed: "Execution Could Not Start" }, () => commands.startExecution.mutateAsync(id))
          : null,
      startRefusal: canStart ? undefined : "Reload Execution Controls or Select a New Plugin Execution",
      onDeleteCase:
        capability === null || capability.delete
          ? () => {
              deleteOpen = true;
            }
          : null,
      deleteRefusal: capability?.delete === false ? capability.unavailable_reasons.join(" ") : undefined,
      archive: execution
        ? archiveAction(Boolean(execution.archived), terminal, "execution", (archived) =>
            commands.archiveExecution.mutateAsync({ archived, executionId: execution.test_execution_id }),
          )
        : null,
      onRerun: terminal
        ? () =>
            run({ done: "New Attempt Ready to Start", failed: "Case Re-Run Failed" }, async () => {
              const created = await commands.appendAttempt.mutateAsync(row.case.caseId);
              if (created) {
                selectExecution(created);
              }
            })
        : null,
      rerunRefusal: phase === "active" ? undefined : "Only Active Cycles Accept New Attempts",
      onSettings: execution
        ? () => {
            settingsOpen = true;
          }
        : null,
    };
  }

  const cycleMenu = $derived<CycleMenu>({
    archive: archiveAction(Boolean(plan?.test_cycle.archived), isCycleArchivable(phase), "cycle", (archived) =>
      commands.archiveCycle.mutateAsync(archived),
    ),
    followActive: follow(),
    kind: "cycle",
    onAddCases: () => addCases(cycleId),
    onRerunCycle: () =>
      run({ done: "Cycle Rerun Planned", failed: "The Cycle Could Not Be Rerun" }, async () => {
        const response = await commands.rerunCycle.mutateAsync();
        useRerunNoticeStore.getState().setRerunNotices(response.cycleId, response);
        openCycle(response.cycleId);
      }),
    onSettings: () => {
      settingsOpen = true;
    },
    onToggleFollowActive: () => useSelectionStore.getState().setSelectionState({ followActive: !follow() }),
    onSeal:
      !runningRow && (phase === "active" || phase === "seal_failed")
        ? () => {
            confirm = phase === "seal_failed" ? "retry" : "seal";
          }
        : null,
    sealFailed: phase === "seal_failed",
  });

  function deleteCase(caseId: string) {
    deleteOpen = false;
    run({ done: "Case Deleted", failed: "The Case Could Not Be Deleted" }, async () => {
      await commands.deleteCase.mutateAsync(caseId);
      onSelectionChange({ view: "cycle" });
    });
  }

  function confirmCycleAction() {
    const action = confirm;
    if (!action) {
      return;
    }
    confirm = null;
    const { done, failed } = CONFIRMS[action];
    run({ done, failed }, () => (action === "archive" ? commands.archiveCycle.mutateAsync(true) : commands.sealCycle.mutateAsync()));
  }
</script>

{#snippet sealActions()}
  {#if control.actions.includes("Seal Cycle")}
    <Button class="h-[var(--bar-control-height)] px-3" onclick={() => (confirm = "seal")} variant="outline">
      <LockKeyhole data-icon="inline-start" />
      Seal Cycle
    </Button>
  {:else if control.actions.includes("Retry Seal")}
    <Button class="h-[var(--bar-control-height)] px-3" onclick={() => (confirm = "retry")} variant="outline">
      <RotateCcw data-icon="inline-start" />
      Retry Seal
    </Button>
    {#if control.actions.includes("Archive") && !plan?.test_cycle.archived}
      <Button class="h-[var(--bar-control-height)] px-3" onclick={() => (confirm = "archive")} variant="outline">
        <Archive data-icon="inline-start" />
        Archive
      </Button>
    {/if}
  {/if}
{/snippet}

{#if !plan && detail.isError}
  <ErrorCard failure={failureOf(detail.error, "Cycle Unavailable")} onRetry={() => void detail.refetch()} />
{:else if !plan && detail.isPending}
  <div class="grid justify-items-center gap-3 px-4 py-16 text-center">
    <h1 class="font-heading text-lg font-semibold">Loading</h1>
    <p class="max-w-prose text-sm text-muted-foreground">Reading this cycle from the backend.</p>
  </div>
{:else if !plan}
  <ErrorCard
    failure={failureOf("This cycle could not be found on the connected backend.", "Cycle Not Found", `missing-cycle:${cycleId}`)}
    onRetry={() => void detail.refetch()}
  />
{:else if selected}
  {@const row = selected}
  {@const execution = row.execution}
  {@const executionSealed = executions.some((attempt) => attempt.case_id === row.case.caseId && attempt.sealed)}
  <div class="grid min-h-0">
    <Dialog.Root bind:open={deleteOpen}>
      <DialogContent>
        <div class="flex flex-col gap-2 pr-8" data-slot="dialog-header">
          <Dialog.Title class="font-sans text-base leading-none font-medium">Delete Case</Dialog.Title>
          <Dialog.Description class="text-sm text-muted-foreground">
            Deleting removes the case "{row.case.label}" and its
            {rows.filter((candidate) => candidate.case.caseId === row.case.caseId && candidate.execution !== null).length}
            executions from this cycle. This cannot be undone.
          </Dialog.Description>
        </div>
        <div class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end">
          <Button onclick={() => (deleteOpen = false)} variant="outline">Cancel</Button>
          <Button onclick={() => deleteCase(row.case.caseId)} variant="danger">Delete Case</Button>
        </div>
      </DialogContent>
    </Dialog.Root>
    {#if execution}
      <SettingsDialog
        open={settingsOpen}
        onOpenChange={(open) => (settingsOpen = open)}
        title="Execution Settings"
        description="The execution's own settings. Its title is its case's label; a sealed attempt keeps the label its report froze."
        values={{ title: row.case.label }}
        readOnly={executionSealed ? { title: "Sealed: the report froze this label." } : {}}
        onSubmit={async ({ title }) => {
          if (title !== row.case.label) {
            await commands.renameExecution.mutateAsync({ executionId: execution.test_execution_id, label: title });
          }
          toast.success("Execution Settings Saved");
        }}
      />
    {/if}
    <ExecutionSections
      {context}
      {commands}
      {cycleId}
      {focus}
      {plan}
      {prompts}
      {row}
      {snapshot}
      menu={executionMenu(row)}
      label={row.case.label}
      titlePrefix={`Execution ${row.ordinal} · `}
      onSelectExecution={selectExecution}
      onRename={execution && !executionSealed
        ? (label) => commands.renameExecution.mutateAsync({ executionId: execution.test_execution_id, label })
        : null}
    />
  </div>
{:else}
  {@const cycle = plan.test_cycle}
  {@const description = typeof cycle.metadata.description === "string" ? cycle.metadata.description : ""}
  <div class="grid min-h-0">
    <CycleEvidencePane {context} {cycleId} {plan} prompts={cyclePrompts} {snapshot}>
      {#snippet pageActions()}
        <NodeActionCluster
          {commands}
          {focus}
          menu={cycleMenu}
          primary={sealActions}
          subject={{ executionId: null, label: cycle.title, sealed: phase === "sealed" }}
          reportSealed={cycleReportSealed(status, executions)}
        />
      {/snippet}
      {#snippet lead()}
        <NodeSummary
          {description}
          live
          onRename={cycle.sealed || cycle.archived ? null : (label) => commands.renameCycle.mutateAsync(label)}
          progress={`${sealed} of ${cases.length} Cases Sealed`}
          rerun={rerunOf
            ? {
                href: `/cycles/${encodeURIComponent(rerunOf)}`,
                label: rerunLabel.data ?? rerunOf,
                open: () => {
                  if (rerunOf) {
                    openCycle(rerunOf);
                  }
                },
              }
            : undefined}
          status={cycleStatus}
          subject="Cycle"
          title={cycle.title}
        />
        <ControlCard
          actions={sealActions}
          label="Cycle Controls"
          progress={runningRow && progress ? { completed: progress.completed, phase: progress.phase, total: progress.total } : null}
          state={control.line}
        >
          {#if phase === "seal_failed" && cycle.seal_failure}
            <ErrorCard
              failure={failureOf(cycle.seal_failure, "Cycle Finalization Failed", [cycleId, "seal", cycle.seal_failure].join(":"))}
            />
          {/if}
          {#if rerunNotices}
            {#each rerunNoticeLines(rerunNotices) as line (line)}
              <p class="text-sm text-muted-foreground" data-rerun-notice>{line}</p>
            {/each}
          {/if}
          {#if waiting}
            <p class="grid auto-cols-max grid-flow-col items-baseline gap-x-2 text-sm" data-input-needed>
              <span class="text-warning">{waiting}</span>
              {#each waitingRows as row (row.key)}
                {#if row.execution}
                  {@const id = row.execution.test_execution_id}
                  <Button class="h-auto px-0" onclick={() => selectExecution(id)} variant="link">Answer On #{row.ordinal}</Button>
                {/if}
              {/each}
            </p>
          {/if}
        </ControlCard>
      {/snippet}
    </CycleEvidencePane>

    <SettingsDialog
      open={settingsOpen}
      onOpenChange={(open) => (settingsOpen = open)}
      title="Cycle Settings"
      description="The cycle's own settings. Continuation starts the next unstarted execution after one completes, whatever its verdict."
      values={{ continueAutomatically: cycle.continue_automatically, description, title: cycle.title }}
      readOnly={cycleSettingsReadOnly(phase, Boolean(cycle.archived))}
      onSubmit={async ({ title, continueAutomatically }) => {
        if (title !== cycle.title) {
          await commands.renameCycle.mutateAsync(title);
        }
        if (continueAutomatically !== undefined && continueAutomatically !== cycle.continue_automatically) {
          await commands.updatePolicy.mutateAsync({ continueAutomatically });
        }
        toast.success("Cycle Settings Saved");
      }}
    />
    <Dialog.Root
      open={confirming !== null}
      onOpenChange={(next) => {
        if (!next && !confirmPending) {
          confirm = null;
        }
      }}
    >
      <DialogContent>
        <div class="flex flex-col gap-2 pr-8" data-slot="dialog-header">
          <Dialog.Title class="font-sans text-base leading-none font-medium">{confirming?.title}</Dialog.Title>
          <Dialog.Description class="text-sm text-muted-foreground">{confirming?.description}</Dialog.Description>
        </div>
        <div class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end">
          <Button disabled={confirmPending} onclick={() => (confirm = null)} variant="outline">Cancel</Button>
          <Button disabled={confirmPending} onclick={confirmCycleAction} variant="danger">
            {confirmPending ? confirming?.pending : confirming?.action}
          </Button>
        </div>
      </DialogContent>
    </Dialog.Root>
  </div>
{/if}
