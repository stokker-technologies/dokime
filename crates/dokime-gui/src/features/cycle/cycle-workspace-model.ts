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

import type { CaseActions, ExecutionQueueItem, TestCase, TestCycle, TestCyclePlan, TestExecution } from "@/lib/types";

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
/** Characters no saved report's name carries, whichever desktop saves it. */
const UNSAFE_FILENAME_CHARACTERS = '\\/:*?"<>|';

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
const FINISHED_CASE_STATES: ReadonlySet<CaseState> = new Set(["passed", "failed", "blocked", "stopped", "interrupted", "skipped"]);

/** The two pre-dispatch states: no attempt at all, and one not yet dispatched. */
const QUEUED_CASE_STATES: ReadonlySet<CaseState> = new Set(["planned", "queued"]);

const ACTIVE_EXECUTION_STATUSES: ReadonlySet<TestExecution["status"]> = new Set(["running", "waiting_for_prompt", "finalizing"]);

/** The execution states an attempt ends in, and so the ones Run Again follows. */
const TERMINAL_EXECUTION_STATUSES: ReadonlySet<TestExecution["status"]> = new Set([
  "passed",
  "failed",
  "skipped",
  "stopped",
  "interrupted",
  "disconnected",
]);

/** The cycle states a whole run ends in, and so the ones Run Again follows. */
const TERMINAL_CYCLE_STATUSES: ReadonlySet<CycleLifecycle> = new Set(["completed", "failed", "stopped", "interrupted"]);

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

/**
 * What the URL selects: the cycle itself, an execution, or a case that has
 * none yet. No field at all is the landing state, which follows the active
 * execution; `view: "cycle"` is the operator's explicit choice of the cycle.
 */
export type CycleSelection = { caseId?: string; executionId?: string; view?: "cycle" };

/** The input `followSelection` decides from. */
export type FollowState = {
  selection: CycleSelection;
  /** The execution the runtime names as running now. */
  active: string | null;
  /** The execution it named on the previous render. */
  previousActive: string | null;
  /** The selected execution status is absent until its data arrives. */
  selectedStatus: TestExecution["status"] | undefined;
  /** The previous observation belongs to this exact selected execution. */
  previousSelected: { executionId?: string; status: TestExecution["status"] | undefined } | null;
};

/** One entry of a node's "…" menu. */
export type NodeMenuItem = "Copy Cycle" | "Add Cases" | "Settings" | "Follow Active" | "Open Report" | "Archive" | "Unarchive" | "Re-Run";

/** The page bar's Download Report button as it stands. */
export type ReportDownloadState = {
  disabled: boolean;
  label: "Download Report";
  title: string;
};

/* === Public Functions === */

export function isFinishedCase(status: CaseState): boolean {
  return FINISHED_CASE_STATES.has(status);
}

/** Whether this attempt is a placeholder for a case the cycle has not run. */
export function isPlannedExecution(status: TestExecution["status"]): boolean {
  return status === "planned";
}

/** An attempt with nothing captured yet: planned, or queued and undispatched. */
export function isUnstartedExecution(status: TestExecution["status"]): boolean {
  return status === "planned" || status === "created";
}

/** Whether the selection names the cycle itself rather than a member. */
export function isCycleSelection(selection: CycleSelection): boolean {
  return selection.view === "cycle";
}

/**
 * Where Follow Active moves the selection, or `null` to leave it.
 *
 * An explicit choice always wins: once the operator selects the cycle, a case
 * or an execution other than the active one, following stops. It resumes when
 * they select the active execution again, or when the execution they were on
 * ends. Landing with nothing selected follows the active execution, and so
 * does a selection that was following when the runtime moved on.
 */
export function followSelection({ selection, active, previousActive, selectedStatus, previousSelected }: FollowState): string | null {
  if (!active || selection.executionId === active) return null;
  if (isCycleSelection(selection) || selection.caseId) return null;
  if (!selection.executionId) return active;
  const selectedEnded =
    previousSelected !== null &&
    previousSelected.executionId === selection.executionId &&
    previousSelected.status !== undefined &&
    selectedStatus !== undefined &&
    !isTerminalExecutionStatus(previousSelected.status) &&
    isTerminalExecutionStatus(selectedStatus);
  if (selection.executionId === previousActive || selectedEnded) return active;
  return null;
}

/** Whether the selection is following `active`: landing, or on it already. */
export function isFollowing(selection: CycleSelection, active: string | null): boolean {
  if (!active || isCycleSelection(selection) || selection.caseId) return false;
  return !selection.executionId || selection.executionId === active;
}

export function isActiveExecutionStatus(status: TestExecution["status"]): boolean {
  return ACTIVE_EXECUTION_STATUSES.has(status);
}

export function isTerminalExecutionStatus(status: TestExecution["status"]): boolean {
  return TERMINAL_EXECUTION_STATUSES.has(status);
}

export function isCycleTerminal(status: CycleLifecycle): boolean {
  return TERMINAL_CYCLE_STATUSES.has(status);
}

/**
 * Whether `row` offers Run Again: the newest attempt of its case, ended. An
 * earlier attempt is history, and the case's actions live on its newest row.
 */
export function rowOffersRunAgain(row: CycleExecutionRow): boolean {
  return row.newest && row.execution !== null && isTerminalExecutionStatus(row.execution.status);
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
export function buildCycleExecutionRows(cases: readonly CycleCase[], executions: readonly TestExecution[]): CycleExecutionRow[] {
  const byId = new Map(cases.map((item) => [item.caseId, item]));
  const ordered = executions
    .filter((execution) => byId.has(execution.case_id))
    .map((execution, position) => ({ execution, position }))
    .sort((left, right) => left.execution.attempt_index - right.execution.attempt_index || left.position - right.position);

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
export function selectedCycleRow(rows: readonly CycleExecutionRow[], selection: CycleSelection): CycleExecutionRow | null {
  if (selection.executionId) {
    return rows.find((row) => row.execution?.test_execution_id === selection.executionId) ?? null;
  }
  if (selection.caseId) {
    return rows.find((row) => row.execution === null && row.case.caseId === selection.caseId) ?? null;
  }
  return null;
}

/** The selection that names `row`: its execution, or its case while it has none. */
export function cycleRowSelection(row: CycleExecutionRow): CycleSelection {
  return row.execution ? { executionId: row.execution.test_execution_id } : { caseId: row.case.caseId };
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
 * A node's "…" menu, as groups a separator divides. Open Report is always
 * there and always enabled: the runtime answers with the sealed report, the
 * one regenerated from sealed evidence, or one rendered from the run so far.
 * The last group archives the node, or unarchives one already archived.
 */
export function nodeMenuGroups(kind: "cycle" | "execution", archived: boolean): NodeMenuItem[][] {
  const archive: NodeMenuItem = archived ? "Unarchive" : "Archive";
  return kind === "cycle"
    ? [["Copy Cycle", "Add Cases", "Follow Active"], ["Open Report", "Settings"], [archive]]
    : [["Re-Run"], ["Open Report", "Settings"], [archive]];
}

/**
 * Why a node cannot be archived yet, or `undefined` once it can: a cycle
 * once it is sealed, an execution once its attempt has ended.
 */
export function archiveRefusal(allowed: boolean, subject: "cycle" | "execution"): string | undefined {
  if (allowed) return undefined;
  return subject === "cycle"
    ? "Archiving becomes available once the cycle is sealed."
    : "Archiving becomes available once the run has ended.";
}

/** Download Report waits only for a save already in progress. */
export function reportDownloadState(saving: boolean): ReportDownloadState {
  return {
    disabled: saving,
    label: "Download Report",
    title: saving ? "Saving the report…" : "Download Report",
  };
}

/** The file a saved report is offered as, named after its cycle or execution. */
export function reportFilename(label: string): string {
  const name = Array.from(label, (char) => (char.charCodeAt(0) < 32 || UNSAFE_FILENAME_CHARACTERS.includes(char) ? "-" : char))
    .join("")
    .replace(/\s+/g, " ")
    .replace(/^[\s.-]+|[\s.-]+$/g, "");
  return `${name || "report"}.html`;
}

/** Whether the cycle is still producing evidence: running, paused or stopping. */
export function isCycleInFlight(status: CycleLifecycle): boolean {
  return status === "running" || status === "paused" || status === "stopping";
}

/** Every label `cycleState` gives, longest first; the Status column fits the first. */
export const CYCLE_STATE_LABELS = ["Active · Waiting", "Active · Running", "Active · Idle", "Sealing", "Sealed"] as const;

export type CycleState = {
  label: (typeof CYCLE_STATE_LABELS)[number];
  variant: "default" | "secondary" | "warning" | "outline";
};

/**
 * The cycle's own state, as its pill and label say it.
 *
 * An Active cycle is Idle, Running or Waiting for an operator; Sealing and
 * Sealed close it. An execution's outcome never reaches this label: a stopped
 * or finished run leaves its Active cycle Idle.
 */
export function cycleState(phase: TestCycle["phase"], lifecycle: CycleLifecycle, waiting: boolean): CycleState {
  if (phase === "sealing") return { label: "Sealing", variant: "warning" };
  if (phase === "sealed") return { label: "Sealed", variant: "secondary" };
  if (waiting) return { label: "Active · Waiting", variant: "warning" };
  if (isCycleInFlight(lifecycle)) return { label: "Active · Running", variant: "default" };
  return { label: "Active · Idle", variant: "outline" };
}

/** Whether the cycle has closed: prompts end with it, and only a sealed cycle archives. */
export function isCycleSealed(phase: TestCycle["phase"]): boolean {
  return phase === "sealed";
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

export function cycleStatusVariant(status: CycleLifecycle): "default" | "secondary" | "success" | "warning" | "destructive" | "outline" {
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

export function caseStatusVariant(status: CaseState): "default" | "secondary" | "success" | "warning" | "destructive" | "outline" {
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
