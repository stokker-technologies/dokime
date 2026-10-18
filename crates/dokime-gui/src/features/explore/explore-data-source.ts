/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import {
  fetchNativeCheckpointContext,
  fetchNativeCheckpointDocument,
  fetchNativeCheckpointEvidence,
  fetchNativeCheckpointRecords,
  fetchNativeCheckpointTelemetry,
  fetchNativeExecutionPage,
  type NativeExecutionSelection,
  readNativeCheckpointOutput,
} from "@/lib/api/native-checkpoints";

type CheckpointEvidence = Awaited<ReturnType<typeof fetchNativeCheckpointEvidence>>;
type CheckpointRecordPage = Awaited<ReturnType<typeof fetchNativeCheckpointRecords>>;
type CheckpointTelemetryPage = Awaited<ReturnType<typeof fetchNativeCheckpointTelemetry>>;
type CheckpointContext = Awaited<ReturnType<typeof fetchNativeCheckpointContext>>;
type ExecutionFileWindow = Awaited<ReturnType<typeof readNativeCheckpointOutput>>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import type { ExecutionBrowseCycle, ExecutionBrowseEntry, ExecutionBrowsePage, ExecutionCheckpoint } from "@/lib/types/view-contracts";
import { type ExploreEntry, type ExploreTarget, exploreEntryKey, isUnrunExecution } from "./explore-contracts";

export function fetchExecutionPage(
  session: SessionIdentity,
  after: string | null,
  signal?: AbortSignal,
  selection?: NativeExecutionSelection,
): Promise<ExecutionBrowsePage> {
  return fetchNativeExecutionPage(session, after, signal, selection);
}
export function fetchCheckpoint(session: SessionIdentity, checkpoint: ExecutionCheckpoint, signal?: AbortSignal) {
  return fetchNativeCheckpointDocument(session, checkpoint, signal);
}
export function fetchCheckpointEvidence(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  signal?: AbortSignal,
): Promise<CheckpointEvidence> {
  return fetchNativeCheckpointEvidence(session, checkpoint, signal);
}
export function fetchCheckpointRecords(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  after: string | null,
  signal?: AbortSignal,
): Promise<CheckpointRecordPage> {
  return fetchNativeCheckpointRecords(session, checkpoint, { after }, signal);
}
export function fetchCheckpointTelemetry(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  table: string,
  after: string | null,
  signal?: AbortSignal,
): Promise<CheckpointTelemetryPage> {
  return fetchNativeCheckpointTelemetry(session, checkpoint, table, { after }, signal);
}
export function fetchCheckpointContext(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  signal?: AbortSignal,
): Promise<CheckpointContext> {
  return fetchNativeCheckpointContext(session, checkpoint, signal);
}
export function readCheckpointOutput(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  relativePath: string,
  offset: bigint,
  signal?: AbortSignal,
): Promise<ExecutionFileWindow> {
  return readNativeCheckpointOutput(session, checkpoint, relativePath, { offset, maxBytes: 64 * 1024 }, signal);
}
/** The one suite label every entry of this cycle agrees on, or its identifier. */
function suiteTitle(entries: ExecutionBrowseEntry[], cycleId: string): string {
  const labels = new Set(entries.flatMap((entry) => (entry.metadata.suite_label ? [entry.metadata.suite_label] : [])));
  const [only] = [...labels];
  return labels.size === 1 && only ? only : `Cycle ${cycleId}`;
}

/**
 * Only associated executions form cycle groups; unassociated ones stay
 * independent. Every count, lifecycle, result and time span is the backend's
 * fold over the whole selection; grouping only places the loaded entries.
 */
export function groupExecutions(pages: ExecutionBrowsePage[]): ExploreEntry[] {
  const executions = new Map<string, ExecutionBrowseEntry>();
  const cycles = new Map<string, ExecutionBrowseCycle>();
  for (const page of pages) {
    for (const entry of page.entries) executions.set(entry.metadata.execution_id, entry);
    for (const cycle of page.cycles) cycles.set(cycle.cycle_id, cycle);
  }
  const groups = new Map<string, ExecutionBrowseEntry[]>();
  for (const entry of executions.values()) {
    const target: ExploreTarget =
      entry.metadata.cycle_id === null
        ? { kind: "execution", id: entry.metadata.execution_id }
        : { kind: "cycle", id: entry.metadata.cycle_id };
    const key = exploreEntryKey(target);
    const group = groups.get(key) ?? [];
    group.push(entry);
    groups.set(key, group);
  }
  return [...groups]
    .map(([key, entries]): ExploreEntry => {
      const cycleId = entries[0].metadata.cycle_id;
      const totals = cycleId === null ? undefined : cycles.get(cycleId);
      if (!totals) throw new Error(`History.BrowseExecutions returned no totals for ${key}.`);
      const target: ExploreTarget = { kind: "cycle", id: totals.cycle_id };
      const started = totals.started_at;
      const finished = totals.finished_at;
      return {
        target,
        /* A group is titled by its own cycle. Only a cycle whose header is
           no longer readable falls back to a suite label every entry agrees
           on, and then to its identifier. */
        title: totals.label ?? suiteTitle(entries, target.id),
        session_id: null,
        phase: totals.phase,
        lifecycle: totals.lifecycle,
        result_status: totals.result_status,
        case_count: totals.case_count,
        execution_count: totals.execution_count,
        queued_count: totals.queued_count,
        running_count: totals.running_count,
        passed_count: totals.passed_count,
        failed_count: totals.failed_count,
        blocked_count: totals.blocked_count,
        created_at: started ?? "",
        started_at: started,
        finished_at: finished,
        updated_at: finished ?? started ?? "",
        duration_s: started && finished ? Math.max(0, (Date.parse(finished) - Date.parse(started)) / 1000) : null,
        source: { id: "storage", label: "Storage", kind: "storage" },
        /* Only what ran is history. The planned cases that never started are
           held aside: they carry no acquisition, so listing them would state
           cases this cycle never performed as outcomes it recorded. */
        executions: entries.filter((entry) => !isUnrunExecution(entry)),
        unrun: entries.filter(isUnrunExecution),
        unrun_count: totals.unrun_count,
      };
    })
    .sort(
      (left, right) =>
        right.updated_at.localeCompare(left.updated_at) || exploreEntryKey(left.target).localeCompare(exploreEntryKey(right.target)),
    );
}
