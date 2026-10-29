/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native captured-record adapters for the original routed UI. */

import { fallbackRecordDisplay } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

type Cursor = NonNullable<RecordPage["scanned"]>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, RecordScope } from "@/lib/types";
import {
  mapNativeSeverity,
  NativeApiLimitError,
  nativeEvidenceScope,
  nativeValueObjectToJson,
  recordScopeFromNative,
  safeNumber,
  timestampIso,
} from "./native-values";

type RecordPage = Wire<Awaited<ReturnType<typeof rpc.evidenceListRecords>>>;
type CapturedRecord = RecordPage["records"][number];

const GUI_RECORD_TAIL_ITEMS = 1_000;

/**
 * How many of a scope's latest records the GUI holds: the server's page
 * bound, so the tail arrives in one call and later records append from the
 * record stream instead of a fresh replay.
 */
export const RECORD_TAIL = GUI_RECORD_TAIL_ITEMS;

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
  return { cursor: reply.through ?? null, records: reply.records.map(mapNativeRecord) };
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
      for await (const page of reader) yield { cursor: page.scanned ?? null, records: page.records.map(mapNativeRecord) };
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

export function mapNativeRecord(record: CapturedRecord): HubRecord {
  const timestamp = timestampIso(record.timestamp);
  if (!timestamp) throw new Error(`Native record ${record.recordId} has no valid timestamp.`);
  const level = mapNativeSeverity(record.severity);
  const details = nativeValueObjectToJson(record.details);
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
