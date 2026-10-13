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
import type {
  ExecutionBrowseEntry,
  ExecutionBrowsePage,
  ExecutionCheckpoint,
} from "@/lib/types/view-contracts";
import {
  type ExploreEntry,
  type ExploreTarget,
  exploreEntryKey,
  isUnrunExecution,
} from "./explore-contracts";

export function fetchExecutionPage(
  session: SessionIdentity,
  after: string | null,
  signal?: AbortSignal,
  selection?: NativeExecutionSelection,
): Promise<ExecutionBrowsePage> {
  return fetchNativeExecutionPage(session, after, signal, selection);
}
export function fetchCheckpoint(
  session: SessionIdentity,
  checkpoint: ExecutionCheckpoint,
  signal?: AbortSignal,
) {
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
  return readNativeCheckpointOutput(
    session,
    checkpoint,
    relativePath,
    { offset, maxBytes: 64 * 1024 },
    signal,
  );
}
/** The one suite label every entry of this cycle agrees on, or its identifier. */
function suiteTitle(entries: ExecutionBrowseEntry[], cycleId: string): string {
  const labels = new Set(
    entries.flatMap((entry) => (entry.metadata.suite_label ? [entry.metadata.suite_label] : [])),
  );
  const [only] = [...labels];
  return labels.size === 1 && only ? only : `Cycle ${cycleId}`;
}

/** Only associated executions form cycle groups; unassociated ones stay independent. */
export function groupExecutions(pages: ExecutionBrowsePage[]): ExploreEntry[] {
  const complete =
    pages.length > 0 && !pages.at(-1)?.next && pages.every((page) => page.discovery_complete);
  const executions = new Map<string, ExecutionBrowseEntry>();
  for (const page of pages)
    for (const entry of page.entries) executions.set(entry.metadata.execution_id, entry);
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
    .map(([, entries]): ExploreEntry => {
      const cycleId = entries[0].metadata.cycle_id;
      const target: ExploreTarget =
        cycleId === null
          ? { kind: "execution", id: entries[0].metadata.execution_id }
          : { kind: "cycle", id: cycleId };
      /* Only what ran is history. The planned cases that never started are
         held aside: they carry no acquisition, so counting them would state
         cases this cycle never performed as outcomes it recorded. */
      const ran = entries.filter((entry) => !isUnrunExecution(entry));
      const unrun = entries.filter(isUnrunExecution);
      const starts = ran
        .flatMap((entry) => (entry.metadata.started_at ? [entry.metadata.started_at] : []))
        .sort();
      const finishes = ran
        .flatMap((entry) => (entry.metadata.finished_at ? [entry.metadata.finished_at] : []))
        .sort();
      const started = starts[0] ?? null;
      const finished = finishes.at(-1) ?? null;
      const passed = ran.filter((entry) => entry.metadata.status === "passed").length;
      const failed = ran.filter((entry) => entry.metadata.status === "failed").length;
      const running = ran.filter(
        (entry) => entry.availability === "live" && entry.metadata.status === "running",
      ).length;
      const pending = ran.some((entry) => entry.metadata.status === "pending");
      const interrupted = ran.some((entry) => entry.availability === "interrupted");
      const uncheckpointed = ran.some((entry) => entry.availability === "uncheckpointed");
      return {
        target,
        /* A cycle's own label is not in the browse projection, but the
           admitted suite label is, and it is what the entries were selected
           on. Only when every entry that resolves a suite names the same one
           does it describe the group: a cycle spanning two suites has no
           single suite, and naming it after whichever entry paged first would
           be worse than the identifier. */
        title: target.kind === "cycle" ? suiteTitle(entries, target.id) : entries[0].metadata.title,
        session_id: null,
        suite_id: null,
        suite_name: null,
        lifecycle: running
          ? "running"
          : interrupted
            ? "interrupted"
            : pending || ran.length === 0
              ? "pending"
              : uncheckpointed
                ? "uncheckpointed"
                : !complete
                  ? "partial"
                  : "completed",
        result_status:
          !complete || running || interrupted || pending || uncheckpointed || ran.length === 0
            ? null
            : failed
              ? "failed"
              : passed === ran.length
                ? "passed"
                : null,
        case_count: new Set(ran.map((entry) => entry.metadata.case_id)).size,
        execution_count: ran.length,
        queued_count: ran.filter((entry) => entry.metadata.status === "pending").length,
        running_count: running,
        passed_count: passed,
        failed_count: failed,
        blocked_count: ran.filter((entry) => entry.metadata.status === "blocked").length,
        created_at: started ?? "",
        started_at: started,
        finished_at: finished,
        updated_at: finished ?? started ?? "",
        duration_s:
          started && finished
            ? Math.max(0, (Date.parse(finished) - Date.parse(started)) / 1000)
            : null,
        source: { id: "storage", label: "Storage", kind: "storage" },
        executions: ran,
        unrun,
      };
    })
    .sort(
      (left, right) =>
        right.updated_at.localeCompare(left.updated_at) ||
        exploreEntryKey(left.target).localeCompare(exploreEntryKey(right.target)),
    );
}
