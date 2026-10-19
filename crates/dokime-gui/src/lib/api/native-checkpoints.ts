/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native immutable history and checkpoint adapters for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import type {
  Row14 as GeneratedExecutionAvailability,
  Row15 as GeneratedExecutionBrowseLifecycle,
  Row8 as GeneratedExecutionStatus,
  Row12 as GeneratedHistoricalExecutionMetadata,
  Row6 as GeneratedHistoryIssue,
  Row2 as GeneratedNativeExecutionBrowsePage,
  Row13 as GeneratedNativeExecutionCheckpoint,
} from "@generated/dokime/v1/HistoryMessages/BrowseExecutions/messages_pb";
import type {
  Row3 as GeneratedArtifactFrame,
  Row11 as GeneratedArtifactMetadata,
  Row15 as GeneratedArtifactState,
} from "@generated/dokime/v1/HistoryMessages/DownloadCheckpointArtifact/messages_pb";
import type { Row2 as GeneratedCheckpointContext } from "@generated/dokime/v1/HistoryMessages/GetCheckpointContext/messages_pb";
import type { Row2 as GeneratedCheckpointDocument } from "@generated/dokime/v1/HistoryMessages/GetCheckpointDocument/messages_pb";
import type {
  Row2 as GeneratedCheckpointEvidence,
  Row15 as GeneratedExecutionFileKind,
  Row5 as GeneratedExecutionFileMetadata,
  Row16 as GeneratedExecutionFileState,
} from "@generated/dokime/v1/HistoryMessages/GetCheckpointEvidence/messages_pb";
import type {
  Row7 as GeneratedCapturedRecord,
  Row2 as GeneratedCheckpointRecordPage,
  Row10 as GeneratedSeverity,
} from "@generated/dokime/v1/HistoryMessages/ListCheckpointRecords/messages_pb";
import type { Row2 as GeneratedCheckpointTelemetryPage } from "@generated/dokime/v1/HistoryMessages/ListCheckpointTelemetry/messages_pb";
import type { Row4 as GeneratedExecutionFileWindow } from "@generated/dokime/v1/HistoryMessages/ReadCheckpointOutput/messages_pb";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type {
  CheckpointCursor,
  ExecutionBrowseCycle,
  ExecutionBrowseEntry,
  ExecutionBrowsePage,
  ExecutionCheckpoint,
} from "@/lib/types/view-contracts";
import { mapCyclePhase } from "./native-planning";
import { NATIVE_PAGE_SIZE, NativeApiLimitError, nativeValueObjectToJson, safeNumber, timestampIso } from "./native-values";

type NativeExecutionCheckpoint = Wire<GeneratedNativeExecutionCheckpoint>;
type NativeStorageCursor = NonNullable<NativeExecutionCheckpoint["position"]>;
type Severity = Wire<GeneratedSeverity>;
type ArtifactFrame = Wire<GeneratedArtifactFrame>;
type ArtifactMetadata = Wire<GeneratedArtifactMetadata>;
type ArtifactState = Wire<GeneratedArtifactState>;
type ExecutionFileKind = Wire<GeneratedExecutionFileKind>;
type ExecutionFileState = Wire<GeneratedExecutionFileState>;
type ExecutionFileMetadata = Wire<GeneratedExecutionFileMetadata>;
type ExecutionFileWindow = Wire<GeneratedExecutionFileWindow>;
type CapturedRecord = Wire<GeneratedCapturedRecord>;
type ExecutionStatus = Wire<GeneratedExecutionStatus>;
type ResultStatus = NonNullable<Wire<Awaited<ReturnType<typeof rpc.executionGetCycle>>>["result"]>["status"];
type CheckpointContext = Wire<GeneratedCheckpointContext>;
type CheckpointDocument = Wire<GeneratedCheckpointDocument>;
type CheckpointEvidence = Wire<GeneratedCheckpointEvidence>;
type CheckpointRecordPage = Wire<GeneratedCheckpointRecordPage>;
type CheckpointTelemetryPage = Wire<GeneratedCheckpointTelemetryPage>;
type ExecutionAvailability = Wire<GeneratedExecutionAvailability>;
type ExecutionBrowseLifecycle = Wire<GeneratedExecutionBrowseLifecycle>;
type HistoricalExecutionMetadata = Wire<GeneratedHistoricalExecutionMetadata>;
type HistoryIssue = Wire<GeneratedHistoryIssue>;
type NativeExecutionBrowsePage = Wire<GeneratedNativeExecutionBrowsePage>;

/* === Limits and session ownership === */

export const NATIVE_CHECKPOINT_WINDOW_BYTES = 64 * 1024;
export const NATIVE_CHECKPOINT_ARTIFACT_BYTES = 32 * 1024 * 1024;

export interface NativeCheckpointSession extends SessionIdentity {
  /** Keep an artifact stream owned by the selected connection generation. */
  own(dispose: () => void): () => void;
}

export type CheckpointReference = ExecutionCheckpoint;

/** Identity preserves the opaque revision and both original native cursors. */
export function checkpointKey(checkpoint: CheckpointReference | NativeExecutionCheckpoint): string {
  if (
    !checkpoint.executionIdentifier ||
    checkpoint.executionIdentifier.length > 4096 ||
    !(checkpoint.version instanceof Uint8Array) ||
    checkpoint.version.length === 0 ||
    checkpoint.version.length > 4096
  ) {
    throw new Error("Invalid checkpoint identity or revision token.");
  }
  const positionCursor = checkpointCursor(checkpoint.position);
  const evidenceCursor = checkpointCursor(checkpoint.evidenceThrough);
  const position = cursorKey(positionCursor);
  const evidence = cursorKey(evidenceCursor);
  if (position[0] !== evidence[0] || position[1] !== evidence[1] || evidenceCursor.sequence > positionCursor.sequence) {
    throw new Error("Checkpoint evidence differs from inspected history.");
  }
  const version = Array.from(checkpoint.version, (byte) => byte.toString(16).padStart(2, "0")).join("");
  return JSON.stringify([checkpoint.executionIdentifier, position, version, evidence]);
}

/** The cursor narrowed to its validated fields; throws when it is absent or out of bounds. */
function checkpointCursor(cursor: CheckpointReference["position"] | NativeStorageCursor | undefined): CheckpointCursor {
  if (
    !cursor?.history?.store ||
    !cursor.history.epoch ||
    cursor.history.store.length > 4096 ||
    cursor.history.epoch.length > 4096 ||
    typeof cursor.sequence !== "bigint" ||
    cursor.sequence < 0n ||
    cursor.sequence > 0xffffffffffffffffn
  ) {
    throw new Error("Invalid checkpoint storage cursor.");
  }
  return {
    history: { store: cursor.history.store, epoch: cursor.history.epoch },
    sequence: cursor.sequence,
  };
}

function cursorKey(cursor: CheckpointCursor) {
  return [cursor.history.store, cursor.history.epoch, cursor.sequence.toString()] as const;
}

function nativeCheckpoint(checkpoint: CheckpointReference): NativeExecutionCheckpoint {
  checkpointKey(checkpoint);
  return checkpoint;
}

function assertActive(session: SessionIdentity, signal: AbortSignal | undefined, operation: string): void {
  if (signal?.aborted) throw new Error(`${operation} was cancelled.`);
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

function assertCheckpoint(actual: NativeExecutionCheckpoint | undefined, expected: CheckpointReference, operation: string): void {
  if (!actual || checkpointKey(actual) !== checkpointKey(expected)) {
    throw new Error(`${operation} returned a different checkpoint.`);
  }
}

function pageLimit(limit: number | undefined, operation: string): number {
  const value = limit ?? NATIVE_PAGE_SIZE;
  if (!Number.isSafeInteger(value) || value < 1 || value > NATIVE_PAGE_SIZE) {
    throw new NativeApiLimitError(operation, `limit must be between 1 and ${NATIVE_PAGE_SIZE}`);
  }
  return value;
}

function mapIssue(issue: HistoryIssue): {
  source: string | null;
  dataset: string | null;
  message: string;
} {
  return {
    source: issue.source ?? null,
    dataset: issue.dataset ?? null,
    message: issue.message,
  };
}

/* === History discovery === */

/** Read one bounded immutable execution page and retain the old grouping view model. */
/**
 * Server-side selection and ordering for the merged execution browse.
 *
 * `text` matches an entry's title, its execution identifier and its suite
 * label; an empty string selects everything. Repeated fields are disjunctive
 * within themselves and an empty one is no constraint.
 */
export type NativeExecutionSelection = {
  filter?: NonNullable<Parameters<typeof rpc.historyBrowseExecutions>[1]["filter"]>;
  order?: NonNullable<Parameters<typeof rpc.historyBrowseExecutions>[1]["order"]>;
};

/**
 * One page of History.BrowseExecutions under one unchanged selection.
 *
 * `filter` and `order` go out verbatim on every continuation: the contract
 * rejects a changed selection, a changed order, or an `after_execution_id` the
 * filter excludes rather than silently restarting the read, so the caller owns
 * one selection object and this forwards exactly it.
 */
export async function fetchNativeExecutionPage(
  session: SessionIdentity,
  after: string | null,
  signal?: AbortSignal,
  selection: NativeExecutionSelection = {},
): Promise<ExecutionBrowsePage> {
  assertActive(session, signal, "History.BrowseExecutions");
  const page = await rpc.historyBrowseExecutions(session, {
    limit: NATIVE_PAGE_SIZE,
    afterExecutionId: after ?? "",
    filter: selection.filter,
    order: selection.order,
  });
  assertActive(session, signal, "History.BrowseExecutions");
  const next = page.nextAfterExecutionId ?? null;
  if (next !== null && next === (after ?? "")) {
    throw new NativeApiLimitError("History.BrowseExecutions", "pagination did not advance");
  }
  return {
    entries: page.entries.map(mapBrowseEntry),
    next,
    discovery_complete: page.discoveryComplete,
    conflicts: [...page.conflicts],
    omitted_conflicts: page.omittedConflicts,
    issues: page.issues.map(mapIssue),
    omitted_issues: page.omittedIssues,
    cycles: page.cycles.map(mapBrowseCycle),
  };
}

/** Permanently delete one sealed execution using a caller-stable retry identity. */
export async function deleteNativeExecution(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  requestIdentifier: string,
): Promise<{
  request_identifier: string;
  execution_identifier: string;
  local_complete: boolean;
}> {
  if (!requestIdentifier || new TextEncoder().encode(requestIdentifier).byteLength > 256) {
    throw new Error("History.DeleteExecution requires a request identifier of 1 to 256 UTF-8 bytes.");
  }

  assertActive(session, undefined, "History.DeleteExecution");
  const reply = await rpc.historyDeleteExecution(session, {
    checkpoint: nativeCheckpoint(checkpoint),
    requestIdentifier,
  });
  assertActive(session, undefined, "History.DeleteExecution");

  if (reply.requestIdentifier !== requestIdentifier) {
    throw new Error("History.DeleteExecution returned a different request identifier.");
  }
  if (reply.executionIdentifier !== checkpoint.executionIdentifier) {
    throw new Error("History.DeleteExecution returned a different execution identifier.");
  }

  return {
    request_identifier: reply.requestIdentifier,
    execution_identifier: reply.executionIdentifier,
    local_complete: reply.localComplete,
  };
}

function mapBrowseEntry(entry: NativeExecutionBrowsePage["entries"][number]): ExecutionBrowseEntry {
  const metadata = entry.metadata;
  if (!metadata) throw new Error("History.BrowseExecutions returned an entry without metadata.");
  return {
    metadata: mapExecutionMetadata(metadata),
    checkpoint: entry.checkpoint ? mapCheckpoint(entry.checkpoint) : null,
    availability: mapAvailability(entry.availability),
    archived: entry.archived,
  };
}

function mapBrowseCycle(cycle: NativeExecutionBrowsePage["cycles"][number]): ExecutionBrowseCycle {
  const count = (value: bigint, label: string) => safeNumber(value, `History.BrowseExecutions cycle ${label} count`);
  return {
    cycle_id: cycle.cycleId,
    label: cycle.label ?? null,
    phase: cycle.phase ? mapCyclePhase(cycle.phase) : null,
    lifecycle: mapBrowseLifecycle(cycle.lifecycle),
    result_status: cycle.resultStatus ? mapResultStatus(cycle.resultStatus) : null,
    case_count: count(cycle.caseCount, "case"),
    execution_count: count(cycle.executionCount, "execution"),
    unrun_count: count(cycle.unrunCount, "unrun"),
    queued_count: count(cycle.queuedCount, "queued"),
    running_count: count(cycle.runningCount, "running"),
    passed_count: count(cycle.passedCount, "passed"),
    failed_count: count(cycle.failedCount, "failed"),
    blocked_count: count(cycle.blockedCount, "blocked"),
    started_at: timestampIso(cycle.startedAt),
    finished_at: timestampIso(cycle.finishedAt),
  };
}

function mapBrowseLifecycle(value: ExecutionBrowseLifecycle | undefined): ExecutionBrowseCycle["lifecycle"] {
  switch (value?.HarmosVariant.case) {
    case "Running":
      return "running";
    case "Interrupted":
      return "interrupted";
    case "Uncheckpointed":
      return "uncheckpointed";
    case "Completed":
      return "completed";
    default:
      return "pending";
  }
}

function mapCheckpoint(checkpoint: NativeExecutionCheckpoint): ExecutionCheckpoint {
  checkpointKey(checkpoint);
  return {
    executionIdentifier: checkpoint.executionIdentifier,
    position: checkpointCursor(checkpoint.position),
    version: checkpoint.version.slice(),
    evidenceThrough: checkpointCursor(checkpoint.evidenceThrough),
  };
}

function mapExecutionMetadata(metadata: HistoricalExecutionMetadata) {
  return {
    execution_id: metadata.executionId,
    cycle_id: metadata.cycleId ?? null,
    case_id: metadata.caseId,
    /* The frozen case template and admitted suite label this entry belongs
       to: what `ExecutionFilter` selects on, so a client can read back what it
       asked for. Absent when the stored row no longer resolves its case or
       suite revision. */
    template_id: metadata.templateId ?? null,
    suite_label: metadata.suiteLabel ?? null,
    title: metadata.title || metadata.executionId,
    status: executionStatusLabel(metadata.status),
    started_at: timestampIso(metadata.startedAt),
    finished_at: timestampIso(metadata.finishedAt),
    result: metadata.result
      ? {
          status: mapResultStatus(metadata.result.status),
          passed: metadata.result.passed,
          failed: metadata.result.failed,
          blocked: metadata.result.blocked,
          skipped: metadata.result.skipped,
        }
      : null,
  };
}

export function executionStatusLabel(
  status: ExecutionStatus | undefined,
): "skipped" | "invalidated" | "pending" | "running" | "passed" | "failed" | "blocked" | "cancelled" | "interrupted" {
  switch (status?.HarmosVariant.case) {
    case "Skipped":
      return "skipped";
    case "Running":
      return "running";
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "blocked";
    case "Cancelled":
      return "cancelled";
    case "Interrupted":
      return "interrupted";
    case "Invalidated":
      return "invalidated";
    default:
      return "pending";
  }
}

function mapResultStatus(status: ResultStatus | undefined): "pending" | "passed" | "failed" | "blocked" | "cancelled" {
  switch (status?.HarmosVariant.case) {
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "blocked";
    case "Cancelled":
      return "cancelled";
    default:
      return "pending";
  }
}

function mapAvailability(value: ExecutionAvailability | undefined): ExecutionBrowseEntry["availability"] {
  switch (value?.HarmosVariant.case) {
    case "Live":
      return "live";
    case "Stored":
      return "stored";
    case "Interrupted":
      return "interrupted";
    default:
      return "uncheckpointed";
  }
}

/* === Checkpoint reads === */

export async function fetchNativeCheckpointDocument(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  signal?: AbortSignal,
): Promise<Omit<CheckpointDocument, "checkpoint"> & { checkpoint: ExecutionCheckpoint }> {
  assertActive(session, signal, "History.GetCheckpointDocument");
  const reply = await rpc.historyGetCheckpointDocument(session, {
    checkpoint: nativeCheckpoint(checkpoint),
  });
  assertActive(session, signal, "History.GetCheckpointDocument");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.GetCheckpointDocument");
  if (!reply.document?.execution) {
    throw new Error("History.GetCheckpointDocument returned no execution document.");
  }
  if (!reply.checkpoint) throw new Error("History.GetCheckpointDocument returned no checkpoint.");
  return { ...reply, checkpoint: mapCheckpoint(reply.checkpoint) };
}

export async function fetchNativeCheckpointEvidence(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  signal?: AbortSignal,
): Promise<CheckpointEvidence> {
  assertActive(session, signal, "History.GetCheckpointEvidence");
  const reply = await rpc.historyGetCheckpointEvidence(session, {
    checkpoint: nativeCheckpoint(checkpoint),
    sections: [
      { HarmosVariant: { case: "Outputs", value: {} } },
      { HarmosVariant: { case: "Artifacts", value: {} } },
      { HarmosVariant: { case: "RecordSchemas", value: {} } },
      { HarmosVariant: { case: "TelemetryTables", value: {} } },
    ],
  });
  assertActive(session, signal, "History.GetCheckpointEvidence");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.GetCheckpointEvidence");
  return reply;
}

export async function fetchNativeCheckpointRecords(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  options: { after?: string | null; limit?: number } = {},
  signal?: AbortSignal,
): Promise<CheckpointRecordPage> {
  const limit = pageLimit(options.limit, "History.ListCheckpointRecords");
  const after = options.after ?? "";
  assertActive(session, signal, "History.ListCheckpointRecords");
  const reply = await rpc.historyListCheckpointRecords(session, {
    checkpoint: nativeCheckpoint(checkpoint),
    limit,
    after,
  });
  assertActive(session, signal, "History.ListCheckpointRecords");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.ListCheckpointRecords");
  if (reply.nextAfter !== undefined && reply.nextAfter === after) {
    throw new NativeApiLimitError("History.ListCheckpointRecords", "pagination did not advance");
  }
  if (reply.records.some((row) => !row.epoch || !row.record)) {
    throw new NativeApiLimitError("History.ListCheckpointRecords", "record source identity is missing");
  }
  return reply;
}

export async function fetchNativeCheckpointTelemetry(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  table: string,
  options: { after?: string | null; limit?: number } = {},
  signal?: AbortSignal,
): Promise<CheckpointTelemetryPage> {
  if (!table.trim()) throw new Error("History.ListCheckpointTelemetry requires a table.");
  const limit = pageLimit(options.limit, "History.ListCheckpointTelemetry");
  const after = options.after ?? "";
  assertActive(session, signal, "History.ListCheckpointTelemetry");
  const reply = await rpc.historyListCheckpointTelemetry(session, {
    checkpoint: nativeCheckpoint(checkpoint),
    table,
    limit,
    after,
  });
  assertActive(session, signal, "History.ListCheckpointTelemetry");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.ListCheckpointTelemetry");
  if (reply.table !== table) throw new Error("History.ListCheckpointTelemetry returned another table.");
  if (reply.rows.some((row) => !row.epoch || !row.sample || row.sample.stream !== table)) {
    throw new NativeApiLimitError("History.ListCheckpointTelemetry", "sample source identity differs");
  }
  if (reply.nextAfter !== undefined && reply.nextAfter === after) {
    throw new NativeApiLimitError("History.ListCheckpointTelemetry", "pagination did not advance");
  }
  return reply;
}

export async function fetchNativeCheckpointContext(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  signal?: AbortSignal,
): Promise<CheckpointContext> {
  assertActive(session, signal, "History.GetCheckpointContext");
  const reply = await rpc.historyGetCheckpointContext(session, {
    checkpoint: nativeCheckpoint(checkpoint),
  });
  assertActive(session, signal, "History.GetCheckpointContext");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.GetCheckpointContext");
  return reply;
}

export async function readNativeCheckpointOutput(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  relativePath: string,
  options: { offset?: bigint; maxBytes?: number } = {},
  signal?: AbortSignal,
): Promise<ExecutionFileWindow> {
  const offset = options.offset ?? 0n;
  const maxBytes = options.maxBytes ?? NATIVE_CHECKPOINT_WINDOW_BYTES;
  if (offset < 0n) throw new NativeApiLimitError("History.ReadCheckpointOutput", "offset is negative");
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 0 || maxBytes > NATIVE_CHECKPOINT_WINDOW_BYTES) {
    throw new NativeApiLimitError("History.ReadCheckpointOutput", `max_bytes must be between 0 and ${NATIVE_CHECKPOINT_WINDOW_BYTES}`);
  }
  if (!relativePath || relativePath.includes("\0")) {
    throw new Error("History.ReadCheckpointOutput requires a relative path.");
  }
  assertActive(session, signal, "History.ReadCheckpointOutput");
  const reply = await rpc.historyReadCheckpointOutput(session, {
    checkpoint: nativeCheckpoint(checkpoint),
    relativePath,
    offset,
    maxBytes,
  });
  assertActive(session, signal, "History.ReadCheckpointOutput");
  assertCheckpoint(reply.checkpoint, checkpoint, "History.ReadCheckpointOutput");
  const content = reply.content;
  if (!content) throw new Error("History.ReadCheckpointOutput returned no content window.");
  if (content.relativePath !== relativePath || content.offset !== offset) {
    throw new Error("History.ReadCheckpointOutput returned a different window.");
  }
  if (content.data.byteLength > maxBytes || content.observedSizeBytes < content.offset + BigInt(content.data.byteLength)) {
    throw new NativeApiLimitError("History.ReadCheckpointOutput", "invalid response window");
  }
  return content;
}

/* === Verified checkpoint artifact transfer === */

export interface NativeCheckpointArtifact {
  metadata: ArtifactMetadata;
  bytes: Uint8Array;
}

export async function downloadNativeCheckpointArtifact(
  session: NativeCheckpointSession,
  checkpoint: CheckpointReference,
  artifactId: string,
  signal?: AbortSignal,
): Promise<NativeCheckpointArtifact> {
  if (!artifactId.trim()) throw new Error("History.DownloadCheckpointArtifact requires an artifact id.");
  assertActive(session, signal, "History.DownloadCheckpointArtifact");
  let reader: NativeStream<ArtifactFrame> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = async (): Promise<void> => {
    if (!reader) return;
    cancelPromise ??= reader.cancel();
    await cancelPromise;
  };
  const abort = () => void cancelReader().catch(() => undefined);
  signal?.addEventListener("abort", abort, { once: true });
  try {
    reader = await rpc.historyDownloadCheckpointArtifact(session, {
      checkpoint: nativeCheckpoint(checkpoint),
      artifactId,
    });
    assertActive(session, signal, "History.DownloadCheckpointArtifact");
    release = session.own(() => void cancelReader().catch(() => undefined));
    return await assembleCheckpointArtifact(reader, artifactId, () => !signal?.aborted && session.isCurrent());
  } finally {
    signal?.removeEventListener("abort", abort);
    await cancelReader().catch(() => undefined);
    release?.();
  }
}

async function assembleCheckpointArtifact(
  reader: NativeStream<ArtifactFrame>,
  artifactId: string,
  active: () => boolean,
): Promise<NativeCheckpointArtifact> {
  let metadata: ArtifactMetadata | undefined;
  let expectedOffset = 0n;
  let completionSize: bigint | undefined;
  let completionSha: string | undefined;
  let complete = false;
  const chunks: Uint8Array[] = [];
  let total = 0n;
  for await (const frame of reader) {
    if (!active()) throw new Error("History.DownloadCheckpointArtifact was cancelled.");
    if (complete) throw new Error("History.DownloadCheckpointArtifact continued after completion.");
    switch (frame.payload?.HarmosVariant.case) {
      case "Metadata": {
        if (metadata || !frame.payload.HarmosVariant.value.field0 || frame.payload.HarmosVariant.value.field0.artifactId !== artifactId) {
          throw new Error("History.DownloadCheckpointArtifact metadata identity is invalid.");
        }
        metadata = frame.payload.HarmosVariant.value.field0;
        validateArtifactMetadata(metadata);
        break;
      }
      case "Chunk": {
        if (!metadata) throw new Error("History.DownloadCheckpointArtifact sent data before metadata.");
        const chunk = frame.payload.HarmosVariant.value.field0;
        if (!chunk) throw new Error("Artifact chunk payload is missing.");
        if (chunk.offset !== expectedOffset) {
          throw new Error("History.DownloadCheckpointArtifact chunks are not contiguous.");
        }
        total += BigInt(chunk.data.byteLength);
        if (total > BigInt(NATIVE_CHECKPOINT_ARTIFACT_BYTES)) {
          throw new NativeApiLimitError("History.DownloadCheckpointArtifact", "artifact is too large");
        }
        chunks.push(chunk.data);
        expectedOffset = total;
        break;
      }
      case "Complete": {
        if (!metadata) throw new Error("History.DownloadCheckpointArtifact completed before metadata.");
        const completion = frame.payload.HarmosVariant.value.field0;
        if (!completion) throw new Error("Artifact completion payload is missing.");
        completionSize = completion.sizeBytes;
        completionSha = normalizedSha(completion.sha256, "artifact completion");
        complete = true;
        break;
      }
      default:
        throw new Error("History.DownloadCheckpointArtifact contained an unknown frame.");
    }
  }
  if (!metadata || !complete || completionSize === undefined || !completionSha) {
    throw new Error("History.DownloadCheckpointArtifact ended before metadata and completion.");
  }
  if (metadata.sizeBytes !== total || completionSize !== total) {
    throw new Error("History.DownloadCheckpointArtifact size does not match its metadata.");
  }
  if (normalizedSha(metadata.sha256, "artifact metadata") !== completionSha) {
    throw new Error("History.DownloadCheckpointArtifact digest metadata disagrees.");
  }
  const bytes = new Uint8Array(Number(total));
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  if (!active()) throw new Error("History.DownloadCheckpointArtifact was cancelled.");
  const actualSha = hex(new Uint8Array(digest));
  if (actualSha !== completionSha) {
    throw new Error("History.DownloadCheckpointArtifact digest verification failed.");
  }
  return { metadata, bytes };
}

function validateArtifactMetadata(metadata: ArtifactMetadata): void {
  if (metadata.state?.HarmosVariant.case !== "Complete") {
    throw new Error("History.DownloadCheckpointArtifact metadata is not complete.");
  }
  if (metadata.sizeBytes > BigInt(NATIVE_CHECKPOINT_ARTIFACT_BYTES)) {
    throw new NativeApiLimitError("History.DownloadCheckpointArtifact", "artifact is too large");
  }
  normalizedSha(metadata.sha256, "artifact metadata");
}

function normalizedSha(value: string, label: string): string {
  const normalized = value.toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(normalized)) throw new Error(`Invalid ${label} SHA-256.`);
  return normalized;
}

function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

/* === Small display helpers consumed by the original evidence panels === */

export function checkpointFileLabel(file: ExecutionFileMetadata): string {
  return `${file.relativePath} · ${file.sizeBytes.toString()} bytes · ${fileStateLabel(file.state)}`;
}

export function checkpointFileKind(kind: ExecutionFileKind | undefined): "text" | "binary" | "media" | "unknown" {
  switch (kind?.HarmosVariant.case) {
    case "Text":
      return "text";
    case "Binary":
      return "binary";
    case "Media":
      return "media";
    default:
      return "unknown";
  }
}

export function checkpointFileState(state: ExecutionFileState | undefined): string {
  return fileStateLabel(state);
}

function fileStateLabel(state: ExecutionFileState | undefined): string {
  switch (state?.HarmosVariant.case) {
    case "Streaming":
      return "streaming";
    case "Finalized":
      return "finalized";
    case "Interrupted":
      return "interrupted";
    default:
      return "unspecified";
  }
}

export function artifactStateLabel(state: ArtifactState | undefined): string {
  switch (state?.HarmosVariant.case) {
    case "Complete":
      return "complete";
    case "Cancelled":
      return "cancelled";
    case "Rejected":
      return "rejected";
    default:
      return "unspecified";
  }
}

export function nativeRecordText(record: CapturedRecord): {
  timestamp: string;
  severity: string;
  details: Record<string, unknown>;
} {
  return {
    timestamp: timestampIso(record.timestamp) ?? "Unknown time",
    severity: severityLabel(record.severity),
    details: nativeValueObjectToJson(record.details),
  };
}

function severityLabel(value: Severity | undefined): string {
  return value?.HarmosVariant.case?.toLowerCase() ?? "unspecified";
}
