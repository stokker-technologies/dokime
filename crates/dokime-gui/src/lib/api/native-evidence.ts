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

import { create, createFileRegistry, fromBinary, ScalarType, type DescMessage } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema, FeatureSet_FieldPresence } from "@bufbuild/protobuf/wkt";
import { reflect, isReflectMessage, isReflectList, isReflectMap, type ReflectMessage } from "@bufbuild/protobuf/reflect";
import { RecordContractsSchema } from "@host/dokime/v1/catalog_pb.js";
import { Severity, WriteLogSchema } from "@host/dokime/v1/execution_pb.js";

/* === Local Imports === */

import { nativeEvidenceScope, recordScopeFromNative } from "@/lib/host/values";
import { fallbackRecordDisplay } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";

type Cursor = NonNullable<RecordPage["scanned"]>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, RecordScope, JsonObject } from "@/lib/types";
import { mapBounded, NativeIntegerValue, NativeNumberValue, NativeApiLimitError, safeNumber, timestampIso } from "./native-values";

/* === Canonical Types === */

type RecordPage = Wire<Awaited<ReturnType<typeof rpc.evidenceListRecords>>>;
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
  const reply: RecordPage = await rpc.evidenceListRecords(session, {
    selection: { scope: nativeEvidenceScope(scope), recordTypes: [], sources: [], severities: [] },
    latest: RECORD_TAIL,
  });
  if (!reply.replayComplete) throw new NativeApiLimitError("Evidence.ListRecords", "the record tail did not complete in one page");
  const records = await decodeRecords(session, reply.records, new Map());
  return { cursor: reply.through ?? null, records };
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
 * A scope's live tail: `Evidence.WatchRecords` for one scope from `after`,
 * already projected. The frames arrive as pages, and each page is handed on
 * whole, with the cursor it was read through, so a holder appends once per
 * page rather than per record.
 */
export async function openNativeRecordStream(
  session: NativeEvidenceSession,
  scope: RecordScope | null,
  after?: Cursor | null,
): Promise<NativeStream<{ records: HubRecord[]; cursor: Cursor | null }>> {
  const catalogs = new Map<string, Promise<FrozenRecords>>();
  const reader = await rpc.evidenceWatchRecords(session, {
    selection: { scope: nativeEvidenceScope(scope), recordTypes: [], sources: [], severities: [] },
    after: after ?? undefined,
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
        const records = await decodeRecords(session, page.records, catalogs);
        yield { cursor: page.scanned ?? null, records };
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

type FrozenRecords = { messages: Map<string, DescMessage> };

async function decodeRecords(
  session: SessionIdentity,
  records: CapturedRecord[],
  catalogs: Map<string, Promise<FrozenRecords>>,
): Promise<HubRecord[]> {
  return mapBounded(records, async (record) => {
    if (!session.isCurrent()) throw new Error("Read Records: Owning Connection Changed");
    if (record.recordType === "dokime.log") {
      const log = fromBinary(WriteLogSchema, record.payload);
      return mapNativeRecord(record, { message: log.message, severity: log.severity });
    }
    const executionId = record.scope?.executionId;
    if (!executionId) throw new Error("Read Record: Missing Frozen Execution Identity");
    let pending = catalogs.get(executionId);
    if (!pending) {
      if (catalogs.size >= RECORD_TAIL) {
        const oldest = catalogs.keys().next().value;
        if (oldest !== undefined) catalogs.delete(oldest);
      }
      pending = (async () => {
        const execution = await rpc.executionGetExecution(session, { executionId });
        const catalog = execution.schemas?.catalog;
        if (!catalog?.types) throw new Error("Read Record: Missing Frozen Descriptors; Restore This Execution's Catalog");
        const registry = createFileRegistry(create(FileDescriptorSetSchema, catalog.types));
        const messages = new Map<string, DescMessage>();
        for (const section of catalog.sections) {
          if (section.type !== "dokime.v1.RecordContracts") continue;
          for (const contract of fromBinary(RecordContractsSchema, section.payload).records) {
            const identifier = contract.metadata?.identifier;
            const message = registry.getMessage(contract.message);
            if (!identifier || !message || messages.has(identifier)) throw new Error("Read Record: Invalid Frozen Contract Identity");
            messages.set(identifier, message);
          }
        }
        return { messages };
      })();
      catalogs.set(executionId, pending);
    }
    const catalog = await pending;
    if (!session.isCurrent()) throw new Error("Read Records: Owning Connection Changed");
    const message = catalog.messages.get(record.recordType);
    if (!message) throw new Error(`Read Record ${record.recordType}: Frozen Contract Is Missing`);
    const decoded = fromBinary(message, record.payload);
    return mapNativeRecord(record, recordFields(reflect(message, decoded), 0));
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
