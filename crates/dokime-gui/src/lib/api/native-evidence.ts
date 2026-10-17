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

/* === Local Imports === */

import { fallbackRecordDisplay } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

type Cursor = NonNullable<RecordPage["scanned"]>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, RecordScope } from "@/lib/types";
import {
  mapNativeSeverity,
  NATIVE_MAX_PAGES,
  NativeApiLimitError,
  nativeEvidenceScope,
  nativeValueObjectToJson,
  recordScopeFromNative,
  safeNumber,
  timestampIso,
} from "./native-values";

/* === Canonical Types === */

type RecordPage = Wire<Awaited<ReturnType<typeof rpc.evidenceListRecords>>>;
type CapturedRecord = RecordPage["records"][number];

/* === Public API === */

export async function fetchNativeRecords(
  session: SessionIdentity,
  options: { limit?: number; scope?: RecordScope | null } = {},
): Promise<HubRecord[]> {
  const records: CapturedRecord[] = [];
  const selection = {
    scope: nativeEvidenceScope(options.scope),
    recordTypes: [],
    sources: [],
    severities: [],
  };
  let after: Cursor | undefined;
  let through: Cursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply: RecordPage = await rpc.evidenceListRecords(session, {
      selection,
      after,
      through,
    });
    records.push(...reply.records);
    if (reply.replayComplete) break;
    if (!reply.scanned || !reply.through) {
      throw new NativeApiLimitError("Evidence.ListRecords", "replay page did not include continuation cursors");
    }
    if (sameCursor(reply.scanned, after) && sameCursor(reply.through, through)) {
      throw new NativeApiLimitError("Evidence.ListRecords", "replay cursors did not advance");
    }
    after = reply.scanned;
    through = reply.through;
    if (page === NATIVE_MAX_PAGES - 1) {
      throw new NativeApiLimitError("Evidence.ListRecords", `more than ${NATIVE_MAX_PAGES} pages`);
    }
  }
  const limit = options.limit ?? records.length;
  if (limit < 0 || !Number.isSafeInteger(limit)) {
    throw new NativeApiLimitError("Evidence.ListRecords", `invalid result limit ${limit}`);
  }
  return (limit === 0 ? [] : records.slice(-limit)).map(mapNativeRecord);
}

export async function fetchNativeCycleRecords(
  session: SessionIdentity,
  cycleId: string,
  options: { executionId?: string | null; limit?: number } = {},
): Promise<HubRecord[]> {
  return fetchNativeRecords(session, {
    limit: options.limit,
    scope: {
      test_cycle_id: cycleId,
      test_execution_id: options.executionId ?? null,
    },
  });
}

/* === Streams === */

/** A record subscription owned by the connection that opened it. */
export interface NativeEvidenceSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

/**
 * The console's live tail: `Evidence.WatchRecords` for one scope, already
 * projected. The frames arrive as pages, and each page is handed on whole so
 * the drawer appends once rather than per record.
 */
export async function openNativeRecordStream(
  session: NativeEvidenceSession,
  scope: RecordScope | null,
): Promise<NativeStream<HubRecord[]>> {
  const reader = await rpc.evidenceWatchRecords(session, {
    selection: { scope: nativeEvidenceScope(scope), recordTypes: [], sources: [], severities: [] },
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
      for await (const page of reader) yield page.records.map(mapNativeRecord);
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

function sameCursor(left?: Cursor, right?: Cursor): boolean {
  return left?.epoch === right?.epoch && left?.stream === right?.stream && left?.position === right?.position;
}
