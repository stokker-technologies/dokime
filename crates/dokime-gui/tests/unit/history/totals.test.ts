/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";

// Local Imports
import { groupExecutions } from "@/features/explore/explore-data-source";
import { parseTimestampMs } from "@/lib/time-format";
import type { ExecutionBrowseCycle, ExecutionBrowseEntry, ExecutionBrowsePage, SavedCycleSummary } from "@/lib/types/view-contracts";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function entry(executionId: string, status: string, startedAt: string | null): ExecutionBrowseEntry {
  return {
    metadata: {
      execution_id: executionId,
      cycle_id: "cycle-a",
      case_id: `case-${executionId}`,
      suite_label: "Charger Suite",
      title: executionId,
      status,
      started_at: startedAt,
      finished_at: null,
      result: null,
    },
    checkpoint: null,
    availability: "live",
    archived: false,
  } as ExecutionBrowseEntry;
}

function totals(overrides: Partial<ExecutionBrowseCycle> = {}): ExecutionBrowseCycle {
  return {
    cycle_id: "cycle-a",
    label: null,
    phase: "active",
    lifecycle: "running",
    result_status: null,
    case_count: 6,
    execution_count: 6,
    unrun_count: 2,
    queued_count: 4,
    running_count: 1,
    passed_count: 1,
    failed_count: 0,
    blocked_count: 0,
    started_at: "2026-10-16T08:00:00Z",
    finished_at: null,
    ...overrides,
  };
}

function page(entries: ExecutionBrowseEntry[], cycles: ExecutionBrowseCycle[]): ExecutionBrowsePage {
  return {
    entries,
    next: null,
    discovery_complete: true,
    conflicts: [],
    omitted_conflicts: 0,
    issues: [],
    omitted_issues: 0,
    cycles,
  };
}

/* === Entry Point === */

test("Historical Timestamps Remain Valid", () => {
  assert.equal(parseTimestampMs(undefined), null);
  assert.equal(parseTimestampMs(null), null);
  assert.equal(parseTimestampMs(""), null);
  assert.equal(parseTimestampMs("invalid"), null);
  assert.equal(parseTimestampMs("1970-01-01T00:00:00Z"), 0);
  assert.equal(parseTimestampMs("1969-12-31T23:59:59Z"), -1_000);
  assert.equal(parseTimestampMs("1999-12-31T23:59:59Z"), 946_684_799_000);
  assert.equal(parseTimestampMs("2000-01-01T00:00:00Z"), 946_684_800_000);
});

test("Backend Cycle Totals", () => {
  // Totals Cover The Whole Selection, Not The Loaded Entries
  const loaded = [entry("run-1", "running", "2026-10-16T08:00:00Z"), entry("run-2", "pending", null), entry("plan-1", "skipped", null)];
  const [cycle] = groupExecutions([page(loaded, [totals()])]);
  assert.equal(cycle.target.id, "cycle-a");
  assert.equal(cycle.title, "Charger Suite");
  assert.equal(cycle.lifecycle, "running");
  assert.equal(cycle.phase, "active");
  assert.equal(cycle.result_status, null);
  assert.equal(cycle.execution_count, 6);
  assert.equal(cycle.running_count, 1);
  assert.equal(cycle.queued_count, 4);
  assert.equal(cycle.unrun_count, 2);
  assert.equal(cycle.started_at, "2026-10-16T08:00:00Z");
  assert.deepEqual(
    cycle.executions.map((execution) => execution.metadata.execution_id),
    ["run-1", "run-2"],
  );
  assert.deepEqual(
    cycle.unrun.map((execution) => execution.metadata.execution_id),
    ["plan-1"],
  );

  // A Later Page Carries The Latest Totals
  const settled = totals({
    lifecycle: "completed",
    result_status: "passed",
    queued_count: 0,
    running_count: 0,
    passed_count: 6,
    finished_at: "2026-10-16T09:00:00Z",
  });
  const [later] = groupExecutions([page(loaded, [totals()]), page([], [settled])]);
  assert.equal(later.lifecycle, "completed");
  assert.equal(later.phase, "active", "finished executions do not seal their container");
  assert.equal(later.result_status, "passed");
  assert.equal(later.passed_count, 6);
  assert.equal(later.duration_s, 3600);

  const [sealed] = groupExecutions([page(loaded, [totals({ phase: "sealed", lifecycle: "completed" })])]);
  assert.equal(sealed.phase, "sealed");

  // A Readable Cycle Is Titled By Its Own Label
  const [labelled] = groupExecutions([page(loaded, [totals({ label: "Charger Bench" })])]);
  assert.equal(labelled.title, "Charger Bench");

  // Entries Without Backend Totals Are Refused
  assert.throws(() => groupExecutions([page(loaded, [])]), /no totals/);
});

test("Empty Sealed Cycles Remain Discoverable Without Invented Outcomes", () => {
  const saved: SavedCycleSummary = {
    cycle_id: "empty-cycle",
    label: "Saved Empty Cycle",
    phase: "sealed",
    created_at: "2026-10-16T08:00:00Z",
    updated_at: "2026-10-16T09:00:00Z",
    execution_count: 0,
    archived: false,
  };
  const [empty] = groupExecutions([], [saved, saved]);
  assert.equal(empty.title, saved.label);
  assert.equal(empty.phase, "sealed");
  assert.equal(empty.updated_at, saved.updated_at);
  assert.equal(empty.result_status, null);
  assert.equal(empty.execution_count, 0);
  assert.deepEqual(empty.executions, []);
  assert.deepEqual(empty.unrun, []);
  assert.equal(groupExecutions([], [saved, saved]).length, 1);

  const loaded = [entry("execution-a", "passed", "2026-10-16T08:00:00Z")];
  const [nonempty] = groupExecutions(
    [page(loaded, [totals({ label: "Authoritative Fold" })])],
    [{ ...saved, cycle_id: "cycle-a", execution_count: 1 }],
  );
  assert.equal(nonempty.title, "Authoritative Fold");
  assert.equal(nonempty.execution_count, 6);
  assert.equal(groupExecutions([], [{ ...saved, execution_count: 1 }]).length, 0);
});
