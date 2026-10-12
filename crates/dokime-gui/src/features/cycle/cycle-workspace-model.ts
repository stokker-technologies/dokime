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
 * Only derivation lives here: the case rows the pane lists, the finished count
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

export type CycleCaseRow = {
  caseId: string;
  templateId: string;
  label: string;
  testCase: TestCase | null;
  /** Every attempt of this case, oldest first. */
  attempts: TestExecution[];
  /**
   * The attempts there is something to read, oldest first.
   *
   * A planned attempt is a placeholder for work the cycle has not done: no
   * steps, no records, no document. It is not an attempt to browse, so the
   * evidence pane and the sidebar's attempt count both count from here.
   */
  browsable: TestExecution[];
  latest: TestExecution | null;
  /** The case's own state, as `CaseSummary` projects it. */
  status: CaseState;
  /** Every live attempt this case holds, superseded ones included. */
  attemptCount: number;
  /** A recorded outcome with a fresh undispatched attempt on top of it. */
  rerunScheduled: boolean;
  /** Null where the projection reports none: no menu item is offered then. */
  actions: CaseActions | null;
  finished: boolean;
};

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
export type CycleTreeSelection = { kind: "overview" } | { kind: "case"; caseId: string };

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
 * from the attempt vector. The attempts are still joined, because the evidence
 * pane and the attempt selector list them; nothing about a case's *state* is
 * read off them.
 */
export function buildCycleCaseRows(
  plan: TestCyclePlan | null,
  executions: readonly TestExecution[],
): CycleCaseRow[] {
  if (!plan) return [];
  const byCase = new Map<string, TestExecution[]>();
  for (const execution of executions) {
    const attempts = byCase.get(execution.case_id);
    if (attempts) attempts.push(execution);
    else byCase.set(execution.case_id, [execution]);
  }
  const projection = new Map(plan.queue.map((item) => [item.case_id, item]));

  return plan.cases.map((testCase) => {
    const attempts = [...(byCase.get(testCase.case_id) ?? [])].sort(
      (left, right) => left.attempt_index - right.attempt_index,
    );
    const item = projection.get(testCase.case_id);
    const status: CaseState = item?.status ?? "unknown";
    return {
      caseId: testCase.case_id,
      templateId: testCase.template_id,
      label: testCase.label || testCase.case_id,
      testCase,
      attempts,
      browsable: attempts.filter((attempt) => !isPlannedExecution(attempt.status)),
      latest: attempts.at(-1) ?? null,
      status,
      attemptCount: item?.attempt_count ?? attempts.length,
      rerunScheduled: item?.rerun_scheduled ?? false,
      actions: item?.actions ?? null,
      finished: isFinishedCase(status),
    };
  });
}

export function finishedCaseCount(rows: readonly CycleCaseRow[]): number {
  return rows.reduce((total, row) => total + (row.finished ? 1 : 0), 0);
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
 * Which row of the sidebar tree the URL has selected.
 *
 * The cycle overview and a case are one selection, not two: an absent — or
 * stale — `caseId` selects the overview, so exactly one row can ever carry
 * the selected treatment.
 */
export function cycleTreeSelection(
  caseId: string | undefined,
  rows: readonly CycleCaseRow[],
): CycleTreeSelection {
  if (!caseId || !rows.some((row) => row.caseId === caseId)) {
    return { kind: "overview" };
  }
  return { caseId, kind: "case" };
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
  const lifecycleOffersRun = !["paused", "running", "stopping"].includes(status);
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
export function queuedCaseCount(rows: readonly CycleCaseRow[]): number {
  return rows.reduce((total, row) => total + (isQueuedCase(row) ? 1 : 0), 0);
}

export function isQueuedCase(row: CycleCaseRow): boolean {
  return QUEUED_CASE_STATES.has(row.status);
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
export function startableCaseCount(rows: readonly CycleCaseRow[]): number {
  if (!rows.some((row) => row.actions !== null)) return queuedCaseCount(rows);
  return rows.reduce((total, row) => total + (row.actions?.run ? 1 : 0), 0);
}

/**
 * Why the runtime would refuse this case's mutations, as one tooltip.
 *
 * `unavailable_reasons` is diagnostic copy the runtime deduplicated for the
 * whole case, not a per-operation code, so every disabled item on a row shows
 * the same sentence rather than inventing a reason of its own.
 */
export function caseActionTooltip(row: CycleCaseRow): string | undefined {
  const reasons = row.actions?.unavailable_reasons ?? [];
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
export function cycleRunRefusal(rows: readonly CycleCaseRow[]): string | null {
  if (startableCaseCount(rows) > 0) return null;
  const reasons = [
    ...new Set(
      rows
        .filter((row) => isQueuedCase(row))
        .flatMap((row) => row.actions?.unavailable_reasons ?? []),
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
 * The attempt the workspace shows, as a zero-based index into `attempts`.
 *
 * An unknown or absent execution resolves to the newest attempt; the caller
 * decides whether that resolution may move the selection.
 */
export function attemptIndexOf(
  attempts: readonly TestExecution[],
  executionId: string | undefined,
): number {
  if (attempts.length === 0) return -1;
  const found = attempts.findIndex((attempt) => attempt.test_execution_id === executionId);
  return found >= 0 ? found : attempts.length - 1;
}

export function attemptLabel(index: number, total: number): string {
  return `Attempt ${index + 1} Of ${total}`;
}

/** The case a running cycle is on, as the case projection states it. */
export function activeCaseLabel(rows: readonly CycleCaseRow[]): string | null {
  return rows.find((row) => row.status === "running")?.label ?? null;
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
