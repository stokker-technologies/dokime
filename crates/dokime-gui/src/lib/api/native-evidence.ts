/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native captured-record adapters for the original routed UI. */

/* === External Imports === */

import { type createFileRegistry, fromBinary, ScalarType } from "@bufbuild/protobuf";
import { FeatureSet_FieldPresence } from "@bufbuild/protobuf/wkt";
import { reflect, isReflectMessage, isReflectList, isReflectMap, type ReflectMessage } from "@bufbuild/protobuf/reflect";
import { executionSchemas, recordMessage } from "@/lib/host/schemas";
import { required, cutKey } from "@/lib/rpc/canonical";
import { Severity } from "@host/dokime/v1/evidence_pb.js";
import { LogsAppendRequestSchema } from "@host/dokime/plugin/host/v1/evidence/logs_pb.js";

/* === Local Imports === */

import { nativeEvidenceScope, recordScopeFromNative } from "@/lib/host/values";
import { fallbackRecordDisplay } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";

type Cursor = NonNullable<RecordPage["resolvedAt"]>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, RecordScope, JsonObject } from "@/lib/types";
import { mapBounded, NativeIntegerValue, NativeNumberValue, NativeApiLimitError, safeNumber, timestampIso } from "./native-values";

/* === Canonical Types === */

type RecordPage = Wire<Awaited<ReturnType<typeof rpc.evidenceRecordsList>>>;
type CapturedRecord = RecordPage["records"][number];

/* === Constants === */

const GUI_RECORD_TAIL_ITEMS = 1_000;

/**
 * How many of a scope's latest records the GUI holds: the server's page
 * bound, so the tail arrives in one call and later records append from the
 * record stream instead of a fresh replay.
 */
export const RECORD_TAIL = GUI_RECORD_TAIL_ITEMS;

/* === Public API === */

/** A scope's latest records and the cursor its live tail continues from. */
export type RecordTail = { records: HubRecord[]; cursor: Cursor | null };

/**
 * The latest `RECORD_TAIL` records of one scope, oldest first, in one call.
 *
 * The server scans the replay itself and returns the tail with the frontier
 * it read through; `openNativeRecordStream` continues from that cursor.
 */
export async function fetchNativeRecordTail(session: SessionIdentity, scope: RecordScope | null): Promise<RecordTail> {
  let pageToken = "";
  let at: Cursor | undefined;
  const records: CapturedRecord[] = [];
  const tokens = new Set<string>();
  for (let index = 0; index < 4096; index++) {
    const reply = await rpc.evidenceRecordsList(session, {
      selection: { scope: nativeEvidenceScope(scope), recordTypes: [], sources: [], severities: [] },
      at,
      page: { pageSize: RECORD_TAIL, pageToken },
    });
    const cut = required(reply.resolvedAt, "Read Records Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("Read Records: Retained Cut Changed");
    at = cut;
    records.push(...reply.records);
    if (records.length > RECORD_TAIL) records.splice(0, records.length - RECORD_TAIL);
    if (!reply.nextPageToken) return { cursor: at, records: await decodeRecords(session, records) };
    if (tokens.has(reply.nextPageToken)) throw new Error("Read Records: Repeated Page Token");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("Evidence.Records.List", "Page Budget Reached");
}

/**
 * The tail after `appended`, keeping only the latest `RECORD_TAIL` records. A
 * record the tail already holds is not appended twice, so a stream that
 * overlaps a re-read tail never duplicates a row.
 */
export function appendRecordTail(tail: RecordTail, appended: readonly HubRecord[], cursor: Cursor | null): RecordTail {
  const held = new Set(tail.records.map((record) => record.record_id));
  const fresh = appended.filter((record) => !held.has(record.record_id));
  const records = fresh.length ? [...tail.records, ...fresh].slice(-RECORD_TAIL) : tail.records;
  return { cursor: cursor ?? tail.cursor, records };
}

/* === Streams === */

/** A record subscription owned by the connection that opened it. */
export interface NativeEvidenceSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

/**
 * A scope's live tail: `Evidence.Records.Watch` for one scope from `after`,
 * already projected. The frames arrive as pages, and each page is handed on
 * whole, with the cursor it was read through, so a holder appends once per
 * page rather than per record.
 */
export async function openNativeRecordStream(
  session: NativeEvidenceSession,
  scope: RecordScope | null,
  after?: Cursor | null,
): Promise<NativeStream<{ records: HubRecord[]; cursor: Cursor | null }>> {
  const reader = await rpc.evidenceRecordsWatch(session, {
    selection: { scope: nativeEvidenceScope(scope), recordTypes: [], sources: [], severities: [] },
    at: after ?? undefined,
  });
  let settled = false;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = (): Promise<void> => {
    cancelPromise ??= reader.cancel();
    return cancelPromise;
  };
  const release = session.own(() => {
    if (!settled) void cancelReader().catch(() => undefined);
  });
  let released = false;
  const releaseOnce = () => {
    if (!released) {
      released = true;
      release();
    }
  };
  const forward = (async function* () {
    let exhausted = false;
    try {
      for await (const page of reader) {
        const records = await decodeRecords(session, page.records);
        if (page.resyncRequired) throw new Error(`Watch Records: ${page.resyncRequired.message}; Refresh the Record Tail`);
        yield { cursor: page.resolvedAt ?? null, records };
      }
      exhausted = true;
    } finally {
      if (!exhausted) await cancelReader().catch(() => undefined);
      settled = true;
      releaseOnce();
    }
  })();
  return {
    cancel: async () => {
      if (settled) return;
      await cancelReader();
      settled = true;
      releaseOnce();
    },
    [Symbol.asyncIterator]: () => forward,
  };
}

/* === Mapping === */

function mapNativeRecord(record: CapturedRecord, details: JsonObject): HubRecord {
  const timestamp = timestampIso(record.timestamp);
  if (!timestamp) throw new Error(`Native record ${record.recordId} has no valid timestamp.`);
  const level =
    record.severity === Severity.ERROR
      ? "error"
      : record.severity === Severity.WARNING
        ? "warning"
        : record.severity === Severity.DEBUG
          ? "debug"
          : "info";
  if (record.error) {
    details.error = {
      message: record.error.message,
      code: record.error.code ?? null,
      source: record.error.source ?? null,
      cause_chain: record.error.causeChain.map((cause) => ({
        message: cause.message,
        code: cause.code ?? null,
        source: cause.source ?? null,
      })),
      stacktrace_attachment: record.error.stacktraceAttachment ?? null,
      trace_id: record.error.traceId ?? null,
      correlation_id: record.error.correlationId ?? null,
    };
  }
  return {
    record_id: record.recordId,
    record_type: record.recordType,
    template_id: record.scope?.templateId ?? null,
    source: record.source,
    timestamp,
    session_id: record.scope?.sessionId ?? null,
    sequence: safeNumber(record.sequence, "Evidence.CapturedRecord sequence"),
    level,
    display: fallbackRecordDisplay(level),
    message: record.message ?? record.error?.message ?? null,
    details,
    related_sample_id: record.relatedSampleId ?? null,
    related_stream: record.relatedStreamId ?? null,
    related_artifact_id: record.relatedArtifactId ?? null,
    scope: recordScopeFromNative(record.scope),
  };
}

async function decodeRecords(session: SessionIdentity, records: CapturedRecord[]): Promise<HubRecord[]> {
  const details = new Map<string, Promise<Awaited<ReturnType<typeof rpc.executionsGet>>>>();
  return mapBounded(records, async (record) => {
    if (!session.isCurrent()) throw new Error("Read Records: Owning Connection Changed");
    if (record.recordType === "dokime.log") {
      const log = fromBinary(LogsAppendRequestSchema, record.payload);
      return mapNativeRecord(record, { message: log.message, severity: log.severity });
    }
    const name = required(record.scope?.executionId, "Read Record Execution Identity");
    let pending = details.get(name);
    if (!pending) {
      pending = rpc.executionsGet(session, { name });
      details.set(name, pending);
    }
    const detail = required((await pending).execution, "Read Record Execution");
    const registry = await executionSchemas(session).get(session, detail);
    const message = recordMessage(registry, record.recordType, detail.schemas?.records);
    if (!message) throw new Error(`Read Record ${record.recordType}: Frozen Contract Is Missing`);
    return mapNativeRecord(record, recordFields(reflect(message, fromBinary(message, record.payload)), 0));
  });
}

function recordFields(message: ReflectMessage, depth: number): JsonObject {
  if (depth > 64) throw new Error("Read Record: Payload Nesting Exceeds the View Budget");
  const result: JsonObject = {};
  for (const field of message.fields) {
    if (field.presence !== FeatureSet_FieldPresence.IMPLICIT && !message.isSet(field)) continue;
    result[field.name] = recordValue(message.get(field), field.scalar, depth + 1);
  }
  return result;
}

function recordValue(value: unknown, scalar: ScalarType | undefined, depth: number): unknown {
  if (depth > 64) throw new Error("Read Record: Payload Nesting Exceeds the View Budget");
  if (isReflectMessage(value)) {
    if (value.desc.typeName === "google.protobuf.Timestamp") {
      const fields = recordFields(value, depth);
      const seconds = fields.seconds instanceof NativeIntegerValue ? BigInt(fields.seconds.decimal) : BigInt(String(fields.seconds ?? 0));
      const timestamp = timestampIso({ seconds, nanos: Number(fields.nanos ?? 0) });
      if (!timestamp) throw new Error("Read Record: Invalid Payload Timestamp");
      return { $datetime: timestamp };
    }
    return recordFields(value, depth);
  }
  if (isReflectList(value)) return Array.from(value, (item) => recordValue(item, value.field().scalar, depth + 1));
  if (isReflectMap(value))
    return Object.fromEntries(Array.from(value, ([key, item]) => [String(key), recordValue(item, value.field().scalar, depth + 1)]));
  if (typeof value === "bigint") {
    const unsigned = scalar === ScalarType.UINT64 || scalar === ScalarType.FIXED64;
    const number = Number(value);
    return unsigned || !Number.isSafeInteger(number) ? new NativeIntegerValue(value, unsigned) : number;
  }
  if (typeof value === "number" && (scalar === ScalarType.DOUBLE || scalar === ScalarType.FLOAT)) return new NativeNumberValue(value);
  return value;
}

/** Decode Immutable Evidence With Its Verified Frozen Registry. */
export function decodeFrozenRecords(
  records: CapturedRecord[],
  registry?: ReturnType<typeof createFileRegistry>,
  recordsBindings?: readonly { recordType: string; messageName: string }[],
): HubRecord[] {
  return records.map((record) => {
    if (record.recordType === "dokime.log") {
      const log = fromBinary(LogsAppendRequestSchema, record.payload);
      return mapNativeRecord(record, { message: log.message, severity: log.severity });
    }
    const message = registry ? recordMessage(registry, record.recordType, recordsBindings) : undefined;
    if (!message) throw new Error(`Read Record ${record.recordType}: Frozen Contract Is Missing`);
    return mapNativeRecord(record, recordFields(reflect(message, fromBinary(message, record.payload)), 0));
  });
}
