/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* What the cycle workspace shows about a cycle's progress and controls.
 *
 * Only derivation lives here: the execution rows the list shows, the finished count
 * the header states, the status wording, and which run control a lifecycle
 * allows. The projections in `@/lib/types` stay the only data contract; nothing
 * here introduces a second one.
 */

/* === Local Imports === */

import type {
  CaseActions,
  ExecutionQueueItem,
  TestCase,
  TestCycle,
  TestCyclePlan,
  TestExecution,
} from "@/lib/types";

/* === Types === */

export type CycleLifecycle = TestCycle["status"];

export type CaseState = ExecutionQueueItem["status"];

/** A case of the cycle, as the plan's case projection states it. */
export type CycleCase = {
  caseId: string;
  templateId: string;
  label: string;
  testCase: TestCase;
  /** The case's own state, as `CaseSummary` projects it. */
  status: CaseState;
  /** Every live execution this case holds, superseded ones included. */
  attemptCount: number;
  /** A recorded outcome with a fresh undispatched execution on top of it. */
  rerunScheduled: boolean;
  /** Null where the projection reports none: no menu item is offered then. */
  actions: CaseActions | null;
  finished: boolean;
};

/**
 * One row of the cycle list: one execution, in cycle order.
 *
 * A re-run is a new execution appended to the cycle, so it is a new row at the
 * end rather than a choice inside an existing one. A case the runtime has not
 * materialized an execution for still needs a row to show its parameters and
 * offer Run Case; that row has no execution and is keyed by its case.
 */
export type CycleExecutionRow = {
  /** Unique within the cycle: the execution id, or `case:<caseId>` without one. */
  key: string;
  case: CycleCase;
  execution: TestExecution | null;
  /** One-based position in the cycle list. */
  ordinal: number;
  /** The ordinal of the earlier execution of the same case this one re-runs. */
  rerunOf: number | null;
  /** The newest row of its case, which is the one carrying the case's actions. */
  newest: boolean;
};

/** What the URL selects: an execution, or a case that has none yet. */
export type CycleSelection = { caseId?: string; executionId?: string };

export type RunControlAvailability = {
  run: boolean;
  pause: boolean;
  resume: boolean;
  stop: boolean;
};

/**
 * Which run controls the header shows at all.
 *
 * A wall of unavailable buttons says nothing about a cycle; the controls the
 * lifecycle cannot reach are not dimmed, they are absent, and the overflow
 * menu keeps everything that is not a run control. Pause and Resume are one
 * control that swaps, never two.
 */
export type RunControlVisibility = {
  run: boolean;
  pauseResume: "pause" | "resume" | null;
  stop: boolean;
};

/* === Constants === */

/**
 * One pill, everywhere, sized by what it says.
 *
 * Height, horizontal air, text size and line height are density tokens in
 * `styles.css`, so a status chip in the workspace header, a row in Cycles and
 * a case in the sidebar are literally the same pill. This is the default:
 * reserving the width of the longest status this app can print costs every
 * short one the same space, and in the sidebar that space came out of the case
 * names, which are what the operator is actually reading.
 */
export const STATUS_PILL_CLASS = "justify-center rounded-4xl";

/**
 * The same pill on a fixed column, for the tables that were the reason for one.
 *
 * `--pill-width` is measured from the longest status word this app can print.
 * It belongs where many pills stack vertically and have to align down a wide
 * column — the Cycles table — and nowhere else: a list that is narrow, or
 * short, pays for the column without gaining the alignment.
 */
export const LIST_STATUS_PILL_CLASS = `w-[var(--pill-width)] ${STATUS_PILL_CLASS}`;

/**
 * The case states that end a case.
 *
 * These are the `CaseStatus` members the concept names — PASSED, FAILED,
 * BLOCKED, CANCELLED, INTERRUPTED, SKIPPED — through their `@/lib/types`
 * projection. INVALIDATED is deliberately absent: a superseded case is
 * awaiting its rerun, which is the opposite of finished. So are PLANNED and
 * QUEUED, which are work the cycle still owes, and `unknown`, which is a
 * captured grouping stating nothing about the case at all.
 */
const FINISHED_CASE_STATES: ReadonlySet<CaseState> = new Set([
  "passed",
  "failed",
  "blocked",
  "stopped",
  "interrupted",
  "skipped",
]);

/** The two pre-dispatch states: no attempt at all, and one not yet dispatched. */
const QUEUED_CASE_STATES: ReadonlySet<CaseState> = new Set(["planned", "queued"]);

const ACTIVE_EXECUTION_STATUSES: ReadonlySet<TestExecution["status"]> = new Set([
  "running",
  "waiting_for_prompt",
  "finalizing",
]);

/* === Public Functions === */

export function isFinishedCase(status: CaseState): boolean {
  return FINISHED_CASE_STATES.has(status);
}

/** Whether this attempt is a placeholder for a case the cycle has not run. */
export function isPlannedExecution(status: TestExecution["status"]): boolean {
  return status === "planned";
}

export function isActiveExecutionStatus(status: TestExecution["status"]): boolean {
  return ACTIVE_EXECUTION_STATUSES.has(status);
}

/**
 * The cycle's cases in plan order, each with what the runtime says about it.
 *
 * State, attempt count, rerun and action eligibility come from the plan's case
 * projection — one RPC, the runtime's own statement — and are never rederived
 * from the executions.
 */
export function buildCycleCases(plan: TestCyclePlan | null): CycleCase[] {
  if (!plan) return [];
  const projection = new Map(plan.queue.map((item) => [item.case_id, item]));
  return plan.cases.map((testCase) => {
    const item = projection.get(testCase.case_id);
    const status: CaseState = item?.status ?? "unknown";
    return {
      caseId: testCase.case_id,
      templateId: testCase.template_id,
      label: testCase.label || testCase.case_id,
      testCase,
      status,
      attemptCount: item?.attempt_count ?? 0,
      rerunScheduled: item?.rerun_scheduled ?? false,
      actions: item?.actions ?? null,
      finished: isFinishedCase(status),
    };
  });
}

/**
 * The cycle list: every execution in cycle order, then the cases with none.
 *
 * `attempt_index` is the execution's position in the cycle, live and frozen
 * alike, so ordering by it puts a re-run after everything already created. An
 * execution re-runs the previous execution of its case; the runtime states no
 * other linkage. Executions of a case the plan does not carry are not listed.
 */
export function buildCycleExecutionRows(
  cases: readonly CycleCase[],
  executions: readonly TestExecution[],
): CycleExecutionRow[] {
  const byId = new Map(cases.map((item) => [item.caseId, item]));
  const ordered = executions
    .filter((execution) => byId.has(execution.case_id))
    .map((execution, position) => ({ execution, position }))
    .sort(
      (left, right) =>
        left.execution.attempt_index - right.execution.attempt_index ||
        left.position - right.position,
    );

  const rows: CycleExecutionRow[] = [];
  const previous = new Map<string, CycleExecutionRow>();
  for (const { execution } of ordered) {
    const owner = byId.get(execution.case_id);
    if (!owner) continue;
    const earlier = previous.get(owner.caseId);
    if (earlier) earlier.newest = false;
    const row: CycleExecutionRow = {
      case: owner,
      execution,
      key: execution.test_execution_id,
      newest: true,
      ordinal: rows.length + 1,
      rerunOf: earlier?.ordinal ?? null,
    };
    rows.push(row);
    previous.set(owner.caseId, row);
  }
  for (const owner of cases) {
    if (previous.has(owner.caseId)) continue;
    rows.push({
      case: owner,
      execution: null,
      key: `case:${owner.caseId}`,
      newest: true,
      ordinal: rows.length + 1,
      rerunOf: null,
    });
  }
  return rows;
}

/** The row a selection names, or null for the overview or a stale selection. */
export function selectedCycleRow(
  rows: readonly CycleExecutionRow[],
  selection: CycleSelection,
): CycleExecutionRow | null {
  if (selection.executionId) {
    return rows.find((row) => row.execution?.test_execution_id === selection.executionId) ?? null;
  }
  if (selection.caseId) {
    return (
      rows.find((row) => row.execution === null && row.case.caseId === selection.caseId) ?? null
    );
  }
  return null;
}

/** The selection that names `row`: its execution, or its case while it has none. */
export function cycleRowSelection(row: CycleExecutionRow): CycleSelection {
  return row.execution
    ? { executionId: row.execution.test_execution_id }
    : { caseId: row.case.caseId };
}

/**
 * The execution a re-run of `caseId` created: the first of that case that was
 * not known before the re-run was requested.
 */
export function createdExecutionId(
  executions: readonly TestExecution[],
  caseId: string,
  known: ReadonlySet<string>,
): string | null {
  return (
    executions.find(
      (execution) => execution.case_id === caseId && !known.has(execution.test_execution_id),
    )?.test_execution_id ?? null
  );
}

/** The pill a row shows: its execution's own status, or its case's without one. */
export function cycleRowStatus(row: CycleExecutionRow): {
  label: string;
  variant: ReturnType<typeof caseStatusVariant>;
} {
  return row.execution
    ? {
        label: executionStatusLabel(row.execution.status),
        variant: executionStatusVariant(row.execution.status),
      }
    : { label: caseStatusLabel(row.case.status), variant: caseStatusVariant(row.case.status) };
}

export function finishedCaseCount(cases: readonly CycleCase[]): number {
  return cases.reduce((total, item) => total + (item.finished ? 1 : 0), 0);
}

/**
 * Why the bundle and the report are unavailable while a cycle is in flight.
 *
 * Both are cut from the sealed cycle, and a running, paused or stopping cycle
 * has no seal yet. The backend refuses both with `FAILED_PRECONDITION` while
 * the cycle is in flight, so the CLI and the Python client get the answer the
 * disabled menu items state.
 */
export const SEALED_EVIDENCE_PENDING =
  "The bundle and the report become available when the cycle is terminal.";

/** Whether the cycle is still producing evidence: running, paused or stopping. */
export function isCycleInFlight(status: CycleLifecycle): boolean {
  return status === "running" || status === "paused" || status === "stopping";
}

export function cycleStatusLabel(status: CycleLifecycle): string {
  switch (status) {
    case "planned":
      return "Ready";
    case "running":
      return "Running";
    case "paused":
      return "Paused";
    case "stopping":
      return "Stopping";
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "stopped":
      return "Stopped";
    default:
      return "Interrupted";
  }
}

export function cycleStatusVariant(
  status: CycleLifecycle,
): "default" | "secondary" | "success" | "warning" | "destructive" | "outline" {
  switch (status) {
    case "running":
      return "default";
    case "paused":
    case "stopping":
      return "warning";
    case "completed":
      return "success";
    case "failed":
      return "destructive";
    case "stopped":
    case "interrupted":
      return "secondary";
    default:
      return "outline";
  }
}

export function executionStatusLabel(status: TestExecution["status"]): string {
  switch (status) {
    case "planned":
      return "Planned";
    case "created":
      return "Queued";
    case "running":
      return "Running";
    case "waiting_for_prompt":
      return "Waiting";
    case "finalizing":
      return "Finalizing";
    case "passed":
      return "Passed";
    case "failed":
      return "Failed";
    case "skipped":
      return "Skipped";
    case "stopped":
      return "Stopped";
    case "disconnected":
      return "Disconnected";
    case "invalidated":
      return "Invalidated";
    default:
      return "Interrupted";
  }
}

export function executionStatusVariant(
  status: TestExecution["status"],
): "default" | "secondary" | "success" | "warning" | "destructive" | "outline" {
  switch (status) {
    case "passed":
      return "success";
    case "failed":
      return "destructive";
    case "running":
    case "finalizing":
      return "default";
    case "waiting_for_prompt":
      return "warning";
    case "disconnected":
    case "invalidated":
      return "warning";
    case "stopped":
    case "interrupted":
    case "skipped":
      return "secondary";
    case "planned":
      return "outline";
    default:
      return "outline";
  }
}

/**
 * Which run control the cycle's lifecycle allows.
 *
 * Run Queued needs a case the runtime would admit, so a finished cycle whose
 * cases all ended — or one whose cases the runtime refuses — offers nothing to
 * start. Pause, Resume and Stop stay the lifecycle's own: they act on the
 * cycle, not on a case, and no case projection speaks for them.
 */
export function runControlAvailability(
  status: CycleLifecycle,
  startableCases: number,
): RunControlAvailability {
  switch (status) {
    case "running":
      return { run: false, pause: true, resume: false, stop: true };
    case "paused":
      return { run: false, pause: false, resume: true, stop: true };
    case "stopping":
      return { run: false, pause: false, resume: false, stop: false };
    default:
      return { run: startableCases > 0, pause: false, resume: false, stop: false };
  }
}

/**
 * Which run controls the cycle's lifecycle shows.
 *
 * Run Queued appears exactly when it would work, or when the runtime has a
 * reason it would not. A cycle with no queued case needs cases, not a dead
 * Run, so the control is absent rather than dimmed; a cycle with queued cases
 * the runtime refuses keeps it, disabled, carrying that refusal — which is the
 * explanation a hidden button cannot give.
 */
export function runControlVisibility(
  status: CycleLifecycle,
  startableCases: number,
  refused = false,
): RunControlVisibility {
  const lifecycleOffersRun = !isCycleInFlight(status);
  return {
    pauseResume: status === "running" ? "pause" : status === "paused" ? "resume" : null,
    run: runControlAvailability(status, startableCases).run || (lifecycleOffersRun && refused),
    stop: status === "running" || status === "paused",
  };
}

/**
 * The cases the cycle still owes, as the runtime states them.
 *
 * PLANNED is a case with no attempt at all and QUEUED one whose attempt is
 * materialized but undispatched; those two, and only those two, are work
 * waiting to be dispatched. A RUNNING case is in flight, a finished one needs
 * an explicit rerun, an INVALIDATED one needs its rerun scheduled first, and a
 * captured grouping states nothing to count.
 */
export function queuedCaseCount(cases: readonly CycleCase[]): number {
  return cases.reduce((total, item) => total + (isQueuedCase(item) ? 1 : 0), 0);
}

export function isQueuedCase(item: CycleCase): boolean {
  return QUEUED_CASE_STATES.has(item.status);
}

/**
 * The cases Run Queued would actually start.
 *
 * Eligibility is the runtime's own: `CaseActions.run` is computed from the
 * same checks RunCase enforces, so a case whose provider has gone away does
 * not keep the button alive. A projection that reports no eligibility at all —
 * a captured cycle grouping — has only its case states to go on, and there the
 * queued projection is the whole statement.
 */
export function startableCaseCount(cases: readonly CycleCase[]): number {
  if (!cases.some((item) => item.actions !== null)) return queuedCaseCount(cases);
  return cases.reduce((total, item) => total + (item.actions?.run ? 1 : 0), 0);
}

/**
 * Why the runtime would refuse this case's mutations, as one tooltip.
 *
 * `unavailable_reasons` is diagnostic copy the runtime deduplicated for the
 * whole case, not a per-operation code, so every disabled item on a row shows
 * the same sentence rather than inventing a reason of its own.
 */
export function caseActionTooltip(item: CycleCase): string | undefined {
  const reasons = item.actions?.unavailable_reasons ?? [];
  return reasons.length ? reasons.join(" ") : undefined;
}

/**
 * Why Run Queued would do nothing, when the cycle still owes cases.
 *
 * A cycle with queued work whose every case the runtime refuses is the one
 * place the header has something to explain: hiding the control there would
 * drop the explanation with it, so the button stays and says why. A cycle with
 * nothing queued owes nothing and needs no reason.
 */
export function cycleRunRefusal(cases: readonly CycleCase[]): string | null {
  if (startableCaseCount(cases) > 0) return null;
  const reasons = [
    ...new Set(
      cases
        .filter((item) => isQueuedCase(item))
        .flatMap((item) => item.actions?.unavailable_reasons ?? []),
    ),
  ];
  return reasons.length ? reasons.join(" ") : null;
}

/** What a case's state is called. A captured grouping states nothing. */
export function caseStatusLabel(status: CaseState): string {
  switch (status) {
    case "planned":
      return "Planned";
    case "queued":
      return "Queued";
    case "running":
      return "Running";
    case "waiting_for_prompt":
      return "Waiting";
    case "finalizing":
      return "Finalizing";
    case "passed":
      return "Passed";
    case "failed":
      return "Failed";
    case "blocked":
      return "Blocked";
    case "skipped":
      return "Skipped";
    case "stopped":
      return "Stopped";
    case "interrupted":
      return "Interrupted";
    case "invalidated":
      return "Invalidated";
    default:
      return "Not Run";
  }
}

export function caseStatusVariant(
  status: CaseState,
): "default" | "secondary" | "success" | "warning" | "destructive" | "outline" {
  switch (status) {
    case "passed":
      return "success";
    case "failed":
      return "destructive";
    case "running":
    case "finalizing":
      return "default";
    case "waiting_for_prompt":
    case "blocked":
    case "invalidated":
      return "warning";
    case "stopped":
    case "interrupted":
    case "skipped":
      return "secondary";
    default:
      return "outline";
  }
}

/**
 * The step an operator reads in the header.
 *
 * Step identifiers are frequently opaque (a UUID or a hash), and an opaque
 * token says less than the phase the runtime is in, so fall back to the phase.
 */
export function progressStepLabel(stepId: string, phase: string): string {
  const trimmed = stepId.trim();
  if (!trimmed || /^[0-9a-f]{8,}$/i.test(trimmed) || /^[0-9a-f-]{32,}$/i.test(trimmed)) {
    return phase;
  }
  return trimmed.replaceAll(/[._-]+/g, " ");
}
