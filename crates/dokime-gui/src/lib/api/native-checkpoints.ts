// Dokime — SPDX-License-Identifier: Apache-2.0
// History Projects Final Execution Resources and Their Retained Cuts.

/* === External Imports === */

import { create, fromJson, toJson } from "@bufbuild/protobuf";
import { ReadCutSchema, type ReadCut } from "@host/harmos/v1/types_pb.js";
import type { ExecutionDetail } from "@host/dokime/client/v1/executions_pb.js";
import type { CapturedRecord } from "@host/dokime/client/v1/evidence/records_pb.js";
import * as protocolV1Execution from "@host/dokime/v1/execution_pb.js";
import * as protocolV1Evidence from "@host/dokime/v1/evidence_pb.js";
import * as protocolClientV1ContentExecutionFiles from "@host/dokime/client/v1/content/execution_files_pb.js";
import { Severity as HostSeverity } from "@host/dokime/v1/evidence_pb.js";
import { CyclePhase, type CycleSummary } from "@host/dokime/v1/cycles_pb.js";

/* === Local Imports === */

import * as rpc from "@/lib/host/client";
import { executionSchemas } from "@/lib/host/schemas";
import { required, cutKey, type Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  ExecutionBrowseCycle,
  ExecutionBrowseEntry,
  ExecutionBrowsePage,
  ExecutionCheckpoint,
  SavedCycleSummary,
} from "@/lib/types/view-contracts";
import { mapCyclePhase, mapStepStatus, projectExecutionStatus, listNativeCycles } from "./native-planning";
import { decodeFrozenRecords } from "./native-evidence";
import { fetchNativeDocument } from "./native-documents";
import { readFileReference } from "./native-content";
import { NATIVE_PAGE_SIZE, NATIVE_MAX_PAGES, safeNumber, timestampIso } from "./native-values";

/* === Constants === */

export const NATIVE_CHECKPOINT_WINDOW_BYTES = 65_536;
export const NATIVE_CHECKPOINT_ARTIFACT_BYTES = 33_554_432;

/* === Types === */

type ExecutionStatus = protocolV1Execution.ExecutionState;
type ResultStatus = protocolV1Execution.ResultStatus;
type Severity = HostSeverity;
type ExecutionFileMetadata = Wire<protocolClientV1ContentExecutionFiles.ExecutionFileMetadata>;
type ExecutionFileKind = protocolClientV1ContentExecutionFiles.ExecutionFileKind;
type ExecutionFileState = protocolClientV1ContentExecutionFiles.ExecutionFileState;
type ArtifactState = protocolV1Evidence.ArtifactState;
export interface NativeCheckpointSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}
export type CheckpointReference = ExecutionCheckpoint;
export type NativeCycleContinuation = { at: Wire<ReadCut>; pageToken: string };
export type NativeCyclePage = { entries: SavedCycleSummary[]; next: NativeCycleContinuation | null };
export type NativeExecutionSelection = {
  filter?: Parameters<typeof rpc.executionsList>[1]["filter"];
  order?: Parameters<typeof rpc.executionsList>[1]["order"];
};

/* === Public Functions === */

export function checkpointKey(checkpoint: CheckpointReference): string {
  if (
    !checkpoint.executionIdentifier ||
    !checkpoint.version.length ||
    !checkpoint.position.history.store ||
    !checkpoint.position.history.epoch ||
    checkpoint.position.sequence < 0n ||
    cutKey(checkpoint.position.history) !== cutKey(checkpoint.evidenceThrough.history) ||
    checkpoint.evidenceThrough.sequence > checkpoint.position.sequence
  )
    throw new Error("Read Checkpoint: Invalid Identity, Seal or Position");
  return cutKey(checkpoint);
}

function assertActive(session: SessionIdentity, signal: AbortSignal | undefined, action: string): void {
  if (!session.isCurrent() || signal?.aborted) throw new Error(`${action}: Owning Connection Changed or Read Cancelled`);
}

function evidenceCut(checkpoint: CheckpointReference): Wire<ReadCut> {
  checkpointKey(checkpoint);
  return { positions: [checkpoint.evidenceThrough], revisions: [] };
}

async function executionAt(session: SessionIdentity, checkpoint: CheckpointReference, signal?: AbortSignal) {
  assertActive(session, signal, "Read Execution Checkpoint");
  const reply = await rpc.executionsGet(session, { name: checkpoint.executionIdentifier, at: checkpoint.position });
  const execution = required(reply.execution, "Read Execution Checkpoint");
  if (cutKey(execution.seal) !== cutKey(checkpoint.version) || execution.summary?.executionId !== checkpoint.executionIdentifier)
    throw new Error("Read Execution Checkpoint: Seal or Identity Changed");
  assertActive(session, signal, "Read Execution Checkpoint");
  return execution;
}

export async function fetchNativeCyclePage(
  session: SessionIdentity,
  after: NativeCycleContinuation | null,
  signal: AbortSignal | undefined,
  filter: NonNullable<Parameters<typeof rpc.cyclesList>[1]["filter"]>,
): Promise<NativeCyclePage> {
  assertActive(session, signal, "List Saved Cycles");
  const reply = await rpc.cyclesList(session, {
    filter,
    selection: { phases: [CyclePhase.SEALED] },
    at: after?.at,
    page: { pageSize: NATIVE_PAGE_SIZE, pageToken: after?.pageToken ?? "" },
  });
  const at = required(reply.resolvedAt, "List Saved Cycles Cut");
  if (after && cutKey(after.at) !== cutKey(at)) throw new Error("List Saved Cycles: Retained Cut Changed");
  if (reply.nextPageToken && reply.nextPageToken === after?.pageToken) throw new Error("List Saved Cycles: Continuation Did Not Advance");
  return {
    entries: reply.cycles.map((cycle) => ({
      cycle_id: cycle.cycleId,
      label: cycle.metadata?.label || cycle.cycleId,
      phase: "sealed",
      created_at: required(timestampIso(cycle.createdAt), "Read Saved Cycle Creation"),
      updated_at: required(timestampIso(cycle.updatedAt), "Read Saved Cycle Update"),
      execution_count: safeNumber(cycle.executionCount, "Saved Cycle Executions"),
      archived: cycle.archived,
    })),
    next: reply.nextPageToken ? { at, pageToken: reply.nextPageToken } : null,
  };
}

export async function fetchNativeExecutionPage(
  session: SessionIdentity,
  after: string | null,
  signal?: AbortSignal,
  selection: NativeExecutionSelection = {},
): Promise<ExecutionBrowsePage> {
  assertActive(session, signal, "List Executions");
  const continuation = after ? (JSON.parse(after) as { at: Parameters<typeof fromJson>[1]; token: string }) : undefined;
  const at = continuation ? fromJson(ReadCutSchema, continuation.at) : undefined;
  const reply = await rpc.executionsList(session, {
    ...selection,
    at,
    page: { pageSize: NATIVE_PAGE_SIZE, pageToken: continuation?.token ?? "" },
  });
  const cut = required(reply.resolvedAt, "List Executions Cut");
  if (at && cutKey(at) !== cutKey(cut)) throw new Error("List Executions: Retained Cut Changed");
  if (reply.nextPageToken && reply.nextPageToken === continuation?.token) throw new Error("List Executions: Continuation Did Not Advance");
  const cycles = await listNativeCycles(session);
  assertActive(session, signal, "List Executions");
  return {
    entries: reply.executions.map(mapBrowseEntry),
    next: reply.nextPageToken
      ? JSON.stringify({ at: toJson(ReadCutSchema, create(ReadCutSchema, cut)), token: reply.nextPageToken })
      : null,
    discovery_complete: true,
    conflicts: [],
    omitted_conflicts: 0,
    issues: [],
    omitted_issues: 0,
    cycles: cycles.map(mapBrowseCycle),
  };
}

function mapBrowseEntry(execution: Wire<ExecutionDetail>): ExecutionBrowseEntry {
  const summary = required(execution.summary, "Read Execution Summary");
  const position = execution.resolvedAt;
  const checkpoint =
    summary.state === protocolV1Execution.ExecutionState.SEALED && execution.seal.length && position?.history
      ? {
          executionIdentifier: summary.executionId,
          version: execution.seal,
          position: { history: position.history, sequence: position.sequence },
          evidenceThrough: { history: position.history, sequence: position.sequence },
        }
      : null;
  return {
    metadata: {
      execution_id: summary.executionId,
      cycle_id: execution.cycleId || null,
      case_id: summary.caseId,
      template_id: execution.admittedTemplate?.identifier ?? null,
      suite_label: execution.admittedSuite?.label ?? null,
      title: execution.admittedTemplate?.label || summary.executionId,
      status: executionStatusLabel(summary.state, execution.outcome, !!execution.startedAt),
      started_at: timestampIso(execution.startedAt),
      finished_at: timestampIso(execution.finishedAt),
      result: null,
      origin: execution.origin,
      outcome: execution.outcome,
    },
    checkpoint,
    availability:
      summary.state === protocolV1Execution.ExecutionState.SEALED ? "stored" : execution.controls?.requestId ? "live" : "uncheckpointed",
    archived: summary.archived,
  };
}

function mapBrowseCycle(cycle: Wire<CycleSummary>): ExecutionBrowseCycle {
  const result = cycle.result;
  return {
    cycle_id: cycle.cycleId,
    label: cycle.metadata?.label || null,
    phase: mapCyclePhase(cycle.phase),
    lifecycle: cycle.runningCount ? "running" : cycle.sealed ? "completed" : "pending",
    result_status: result ? mapResultStatus(result.status) : null,
    case_count: safeNumber(cycle.caseCount, "Cycle Cases"),
    execution_count: safeNumber(cycle.executionCount, "Cycle Executions"),
    unrun_count: safeNumber(cycle.queuedCount, "Cycle Unrun Executions"),
    queued_count: safeNumber(cycle.queuedCount, "Cycle Queued Executions"),
    running_count: safeNumber(cycle.runningCount, "Cycle Running Executions"),
    passed_count: result?.passed ?? 0,
    failed_count: result?.failed ?? 0,
    blocked_count: result?.blocked ?? 0,
    started_at: timestampIso(cycle.startedAt),
    finished_at: cycle.sealed ? timestampIso(cycle.updatedAt) : null,
  };
}

export async function deleteNativeExecution(session: SessionIdentity, checkpoint: CheckpointReference, requestIdentifier: string) {
  await executionAt(session, checkpoint);
  const reply = await rpc.executionsDelete(session, { name: checkpoint.executionIdentifier, requestId: requestIdentifier });
  if (reply.receipt?.requestId !== requestIdentifier) throw new Error("Delete Execution: Mutation Receipt Changed");
  return {
    request_identifier: requestIdentifier,
    execution_identifier: checkpoint.executionIdentifier,
    local_complete: reply.receipt.completed,
  };
}

export async function fetchNativeCheckpointDocument(session: SessionIdentity, checkpoint: CheckpointReference, signal?: AbortSignal) {
  const execution = await executionAt(session, checkpoint, signal);
  const [steps, cycle] = await Promise.all([
    rpc.executionsSteps(session, { name: checkpoint.executionIdentifier, at: checkpoint.position }),
    rpc.cyclesGet(session, { name: execution.cycleId }),
  ]);
  return {
    checkpoint,
    acquisitionFinished: !!execution.finishedAt,
    execution,
    steps: steps.steps,
    cycle: required(cycle.cycle, "Read Checkpoint Cycle"),
  };
}

export async function fetchNativeCheckpointEvidence(session: SessionIdentity, checkpoint: CheckpointReference, signal?: AbortSignal) {
  const execution = await executionAt(session, checkpoint, signal);
  const at = evidenceCut(checkpoint);
  const outputs: Wire<Awaited<ReturnType<typeof rpc.contentExecutionFilesList>>>["files"] = [];
  const artifacts: Wire<Awaited<ReturnType<typeof rpc.contentArtifactsList>>>["artifacts"] = [];
  let pageToken = "";
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.contentExecutionFilesList(session, {
      parent: checkpoint.executionIdentifier,
      at,
      page: { pageSize: NATIVE_PAGE_SIZE, pageToken },
    });
    if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Files: Retained Cut Changed");
    outputs.push(...reply.files);
    if (!reply.nextPageToken) break;
    if (reply.nextPageToken === pageToken || index + 1 === NATIVE_MAX_PAGES)
      throw new Error("Read Checkpoint Files: Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
  }
  pageToken = "";
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.contentArtifactsList(session, {
      scope: { executionId: checkpoint.executionIdentifier },
      at,
      page: { pageSize: NATIVE_PAGE_SIZE, pageToken },
    });
    if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Artifacts: Retained Cut Changed");
    artifacts.push(...reply.artifacts);
    if (!reply.nextPageToken) break;
    if (reply.nextPageToken === pageToken || index + 1 === NATIVE_MAX_PAGES)
      throw new Error("Read Checkpoint Artifacts: Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
  }
  assertActive(session, signal, "Read Checkpoint Evidence");
  return { outputs, artifacts, telemetryTables: execution.schemas?.streams.map((schema) => ({ table: schema.stream, schema })) ?? [] };
}

export async function fetchNativeCheckpointRecords(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  options: { after?: string | null; limit?: number } = {},
  signal?: AbortSignal,
) {
  const execution = await executionAt(session, checkpoint, signal);
  const at = evidenceCut(checkpoint);
  const reply = await rpc.evidenceRecordsList(session, {
    selection: { scope: { executionId: checkpoint.executionIdentifier } },
    at,
    page: { pageSize: options.limit ?? NATIVE_PAGE_SIZE, pageToken: options.after ?? "" },
  });
  if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Records: Retained Cut Changed");
  const registry = await executionSchemas(session).get(session, execution);
  const decoded = decodeFrozenRecords(reply.records, registry);
  return {
    records: reply.records.map((record, index) => ({ record, epoch: checkpoint.position.history.epoch, decoded: decoded[index] })),
    nextAfter: reply.nextPageToken || null,
  };
}

export async function fetchNativeCheckpointTelemetry(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  table: string,
  options: { after?: string | null; limit?: number } = {},
  signal?: AbortSignal,
) {
  await executionAt(session, checkpoint, signal);
  const at = evidenceCut(checkpoint);
  const reply = await rpc.evidenceSamplesList(session, {
    scope: { executionId: checkpoint.executionIdentifier },
    streams: [table],
    at,
    page: { pageSize: options.limit ?? NATIVE_PAGE_SIZE, pageToken: options.after ?? "" },
  });
  if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Samples: Retained Cut Changed");
  return { samples: reply.samples, nextAfter: reply.nextPageToken || null };
}

export async function fetchNativeCheckpointContext(session: SessionIdentity, checkpoint: CheckpointReference, signal?: AbortSignal) {
  const execution = await executionAt(session, checkpoint, signal);
  const at = evidenceCut(checkpoint);
  const selection = {
    owner: {
      owner: { kind: { case: "execution" as const, value: { cycleId: execution.cycleId, executionId: checkpoint.executionIdentifier } } },
    },
    includeContext: true,
  };
  const notes = [] as Awaited<ReturnType<typeof rpc.evidenceNotesList>>["notes"];
  const health = [] as Awaited<ReturnType<typeof rpc.evidenceHealthEventsList>>["events"];
  let pageToken = "";
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.evidenceNotesList(session, { selection, at, page: { pageSize: NATIVE_PAGE_SIZE, pageToken } });
    if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Notes: Retained Cut Changed");
    notes.push(...reply.notes);
    if (!reply.nextPageToken) break;
    if (reply.nextPageToken === pageToken || index + 1 === NATIVE_MAX_PAGES)
      throw new Error("Read Checkpoint Notes: Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
  }
  pageToken = "";
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.evidenceHealthEventsList(session, { selection, at, page: { pageSize: NATIVE_PAGE_SIZE, pageToken } });
    if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Health: Retained Cut Changed");
    health.push(...reply.events);
    if (!reply.nextPageToken) break;
    if (reply.nextPageToken === pageToken || index + 1 === NATIVE_MAX_PAGES)
      throw new Error("Read Checkpoint Health: Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
  }
  const prompts = [] as Awaited<ReturnType<typeof rpc.promptsList>>["prompts"];
  pageToken = "";
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.promptsList(session, {
      executionId: checkpoint.executionIdentifier,
      at,
      page: { pageSize: NATIVE_PAGE_SIZE, pageToken },
    });
    if (cutKey(reply.resolvedAt) !== cutKey(at)) throw new Error("Read Checkpoint Prompts: Retained Cut Changed");
    prompts.push(...reply.prompts);
    if (!reply.nextPageToken) break;
    if (reply.nextPageToken === pageToken || index + 1 === NATIVE_MAX_PAGES)
      throw new Error("Read Checkpoint Prompts: Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
  }
  const cycle = await rpc.cyclesGet(session, { name: execution.cycleId });
  const overview = await rpc.backendOverviewGet(session, {});
  const document = await fetchNativeDocument(
    { ...session, own: () => () => undefined },
    {
      cycleId: execution.cycleId,
      executionId: checkpoint.executionIdentifier,
      documentId: "",
      epoch: required(overview.overview, "Read Checkpoint Host Epoch").sessionId,
      at,
      structureOnly: true,
    },
    signal,
  );
  return { cycle: cycle.cycle, notes, health, prompts, document };
}

export async function readNativeCheckpointOutput(
  session: SessionIdentity,
  checkpoint: CheckpointReference,
  relativePath: string,
  options: { offset?: bigint; maxBytes?: number } = {},
  signal?: AbortSignal,
) {
  await executionAt(session, checkpoint, signal);
  const item = await rpc.contentExecutionFilesGet(session, {
    parent: checkpoint.executionIdentifier,
    name: relativePath,
    at: evidenceCut(checkpoint),
  });
  const file = required(item.file, "Read Checkpoint File");
  const offset = options.offset ?? 0n;
  const maxBytes = options.maxBytes ?? NATIVE_CHECKPOINT_WINDOW_BYTES;
  if (offset < 0n || maxBytes < 1 || maxBytes > NATIVE_CHECKPOINT_WINDOW_BYTES) throw new Error("Read Checkpoint File: Invalid Window");
  const reply = await rpc.storageGetFileBytes(session, { file, offset, maxBytes: BigInt(maxBytes), at: item.resolvedAt });
  if (reply.data.length > maxBytes || reply.totalLength !== file.length || cutKey(reply.sha256) !== cutKey(file.sha256))
    throw new Error("Read Checkpoint File: Invalid Range or Digest");
  return { ...required(item.metadata, "Read Checkpoint File Metadata"), offset, data: reply.data, observedSizeBytes: reply.totalLength };
}

export async function downloadNativeCheckpointArtifact(
  session: NativeCheckpointSession,
  checkpoint: CheckpointReference,
  artifactId: string,
  signal?: AbortSignal,
) {
  await executionAt(session, checkpoint, signal);
  const item = await rpc.contentArtifactsGet(session, { name: artifactId, at: evidenceCut(checkpoint) });
  const metadata = required(item.metadata, "Read Checkpoint Artifact Metadata");
  if (metadata.artifactId !== artifactId || metadata.scope?.executionId !== checkpoint.executionIdentifier)
    throw new Error("Read Checkpoint Artifact: Identity Changed");
  const bytes = await readFileReference(
    session,
    required(item.file, "Read Checkpoint Artifact File"),
    signal,
    item.resolvedAt,
    NATIVE_CHECKPOINT_ARTIFACT_BYTES,
  );
  return { metadata, bytes };
}

export function executionStatusLabel(
  state: ExecutionStatus | undefined,
  outcome?: Wire<protocolV1Execution.ExecutionOutcome>,
  started = false,
): "pending" | "running" | "passed" | "failed" | "blocked" | "cancelled" | "interrupted" | "skipped" {
  if (state === protocolV1Execution.ExecutionState.OPEN) return started ? "running" : "pending";
  const attribution = outcome?.attribution;
  if (
    (attribution?.case === "evaluated" || attribution?.case === "declared") &&
    attribution.value.status === protocolV1Execution.ResultStatus.BLOCKED
  )
    return "blocked";
  const status = projectExecutionStatus(state, outcome);
  if (status === "stopped") return "cancelled";
  if (status === "planned" || status === "finalizing") return "pending";
  if (status === "invalidated") return "skipped";
  if (status === "created" || status === "disconnected") return "pending";
  if (status === "waiting_for_prompt") return "running";
  return status;
}

function mapResultStatus(status: ResultStatus | undefined): "pending" | "passed" | "failed" | "blocked" | "cancelled" {
  switch (status) {
    case protocolV1Execution.ResultStatus.PASSED:
      return "passed";
    case protocolV1Execution.ResultStatus.FAILED:
      return "failed";
    case protocolV1Execution.ResultStatus.BLOCKED:
      return "blocked";
    case protocolV1Execution.ResultStatus.CANCELLED:
      return "cancelled";
    default:
      return "pending";
  }
}

export function checkpointFileLabel(file: ExecutionFileMetadata): string {
  return `${file.relativePath} · ${file.sizeBytes.toString()} bytes · ${fileStateLabel(file.state)}`;
}

export function checkpointFileKind(kind: ExecutionFileKind | undefined): "text" | "binary" | "media" | "unknown" {
  switch (kind) {
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.TEXT:
      return "text";
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.BINARY:
      return "binary";
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.MEDIA:
      return "media";
    default:
      return "unknown";
  }
}

export function checkpointFileState(state: ExecutionFileState | undefined): string {
  return fileStateLabel(state);
}

function fileStateLabel(state: ExecutionFileState | undefined): string {
  switch (state) {
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.STREAMING:
      return "streaming";
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.FINALIZED:
      return "finalized";
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.INTERRUPTED:
      return "interrupted";
    default:
      return "unspecified";
  }
}

export function artifactStateLabel(state: ArtifactState | undefined): string {
  switch (state) {
    case protocolV1Evidence.ArtifactState.COMPLETE:
      return "complete";
    case protocolV1Evidence.ArtifactState.CANCELLED:
      return "cancelled";
    case protocolV1Evidence.ArtifactState.REJECTED:
      return "rejected";
    case protocolV1Evidence.ArtifactState.FAILED:
      return "failed";
    default:
      return "unspecified";
  }
}

export function nativeRecordText(
  record: Wire<CapturedRecord>,
  decoded?: import("@/lib/types").HubRecord,
): {
  timestamp: string;
  severity: string;
  details: Record<string, unknown>;
} {
  return {
    timestamp: timestampIso(record.timestamp) ?? "Unknown Time",
    severity: severityLabel(record.severity),
    details: decoded?.details ?? {},
  };
}

function severityLabel(value: Severity | undefined): string {
  return value === undefined ? "unspecified" : (HostSeverity[value]?.toLowerCase() ?? "unspecified");
}

export const checkpointStepStatus = mapStepStatus;
