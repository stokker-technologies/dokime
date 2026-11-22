// Dokime — SPDX-License-Identifier: Apache-2.0
import type { Origin, ExecutionOutcome } from "@generated/host/model";
import type { Wire } from "../rpc/canonical";

export type CycleHistorySummary = {
  test_cycle_id: string;
  session_id: string | null;
  title: string;
  phase: "active" | "sealing" | "seal_failed" | "sealed" | null;
  lifecycle: string;
  result_status: string | null;
  case_count: number;
  execution_count: number;
  queued_count: number;
  running_count: number;
  passed_count: number;
  failed_count: number;
  blocked_count: number;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  updated_at: string;
  duration_s: number | null;
};

export type ExecutionAvailability = "live" | "stored" | "interrupted" | "uncheckpointed";

export type ExecutionBrowseEntry = {
  metadata: ExecutionMetadata;
  checkpoint: ExecutionCheckpoint | null;
  availability: ExecutionAvailability;
  /** The execution, or its whole cycle, is archived. */
  archived: boolean;
};

/** One cycle's totals over the whole browse selection, as the backend folded them. */

export type ExecutionBrowseCycle = {
  cycle_id: string;
  /** The cycle's own label; null once its header is no longer readable. */
  label: string | null;
  phase: "active" | "sealing" | "seal_failed" | "sealed" | null;
  lifecycle: "running" | "interrupted" | "pending" | "uncheckpointed" | "completed";
  result_status: "pending" | "passed" | "failed" | "blocked" | "cancelled" | null;
  case_count: number;
  execution_count: number;
  /** Planned executions that never started; every other count excludes them. */
  unrun_count: number;
  queued_count: number;
  running_count: number;
  passed_count: number;
  failed_count: number;
  blocked_count: number;
  started_at: string | null;
  finished_at: string | null;
};

export type ExecutionBrowseIssue = {
  source: string | null;
  dataset: string | null;
  message: string;
};

export type ExecutionBrowsePage = {
  entries: ExecutionBrowseEntry[];
  next: string | null;
  discovery_complete: boolean;
  conflicts: string[];
  omitted_conflicts: number;
  issues: ExecutionBrowseIssue[];
  omitted_issues: number;
  /** Totals of every cycle the entries belong to, over the whole selection. */
  cycles: ExecutionBrowseCycle[];
};

export type SavedCycleSummary = {
  cycle_id: string;
  label: string;
  phase: "sealed";
  created_at: string;
  updated_at: string;
  execution_count: number;
  archived: boolean;
};

export type ExecutionCheckpoint = {
  executionIdentifier: string;
  version: Uint8Array;
  position: CheckpointCursor;
  evidenceThrough: CheckpointCursor;
};

export type CheckpointCursor = {
  sequence: bigint;
  history: { store: string; epoch: string };
};

export type ExecutionMetadata = {
  origin?: Wire<Origin>;
  outcome?: Wire<ExecutionOutcome>;
  execution_id: string;
  cycle_id: string | null;
  case_id: string;
  /** Null when the stored row no longer resolves its case revision. */
  template_id?: string | null;
  /** Null when the stored row no longer resolves its suite revision. */
  suite_label?: string | null;
  title: string;
  status: ExecutionStatus;
  started_at: string | null;
  finished_at: string | null;
  result: ResultSummary | null;
};

export type ExecutionStatus =
  | "not_run"
  | "skipped"
  | "invalidated"
  | "pending"
  | "running"
  | "passed"
  | "failed"
  | "blocked"
  | "cancelled"
  | "interrupted";

export type ResultStatus = "pending" | "passed" | "failed" | "blocked" | "cancelled";

export type ResultSummary = {
  status: ResultStatus;
  passed: number;
  failed: number;
  blocked: number;
  skipped: number;
};
