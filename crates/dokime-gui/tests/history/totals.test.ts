/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "vitest";
import { groupExecutions } from "@/features/explore/explore-data-source";
import type {
  ExecutionBrowseCycle,
  ExecutionBrowseEntry,
  ExecutionBrowsePage,
} from "@/lib/types/view-contracts";

/* === Fixture === */

function entry(
  executionId: string,
  status: string,
  startedAt: string | null,
): ExecutionBrowseEntry {
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

function page(
  entries: ExecutionBrowseEntry[],
  cycles: ExecutionBrowseCycle[],
): ExecutionBrowsePage {
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

/* === Tests === */

test("Backend Cycle Totals", () => {
  // Totals Cover The Whole Selection, Not The Loaded Entries
  const loaded = [
    entry("run-1", "running", "2026-10-16T08:00:00Z"),
    entry("run-2", "pending", null),
    entry("plan-1", "skipped", null),
  ];
  const [cycle] = groupExecutions([page(loaded, [totals()])]);
  assert.equal(cycle.target.id, "cycle-a");
  assert.equal(cycle.title, "Charger Suite");
  assert.equal(cycle.lifecycle, "running");
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
  assert.equal(later.result_status, "passed");
  assert.equal(later.passed_count, 6);
  assert.equal(later.duration_s, 3600);

  // Entries Without Backend Totals Are Refused
  assert.throws(() => groupExecutions([page(loaded, [])]), /no totals/);
});
