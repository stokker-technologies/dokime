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

import { executionSchemas } from "@/lib/host/schemas";
import { cutKey, required } from "@dokime/plots/lib/rpc/canonical";
import type { EvidenceScope } from "@host/model";
import type { CapturedRecord, RecordPage } from "@dokime/document/lib/api/native-record-projection";

/* === Local Imports === */

import { decodeFrozenRecords } from "@dokime/document/lib/api/native-record-projection";

import * as rpc from "@/lib/host/client";
import { nativeRecordScope } from "@dokime/plots/lib/host/values";
import type { Wire } from "@dokime/plots/lib/rpc/canonical";

type Cursor = NonNullable<RecordPage["resolvedAt"]>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord } from "@dokime/plots/lib/types";
import { NativeApiLimitError, mapBounded } from "@dokime/plots/lib/api/native-values";

/* === Canonical Types === */

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
export async function fetchNativeRecordTail(session: SessionIdentity, scope: Wire<EvidenceScope> | null): Promise<RecordTail> {
  let pageToken = "";
  let at: Cursor | undefined;
  const records: CapturedRecord[] = [];
  const tokens = new Set<string>();
  for (let index = 0; index < 4096; index++) {
    const reply = await rpc.evidenceRecordsList(session, {
      scope: nativeRecordScope(scope),
      selection: { scope: scope ?? undefined, recordTypes: [], sources: [], severities: [] },
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
  scope: Wire<EvidenceScope> | null,
  after?: Cursor | null,
): Promise<NativeStream<{ records: HubRecord[]; cursor: Cursor | null }>> {
  const reader = await rpc.evidenceRecordsWatch(session, {
    scope: nativeRecordScope(scope),
    selection: { scope: scope ?? undefined, recordTypes: [], sources: [], severities: [] },
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

async function decodeRecords(session: SessionIdentity, records: CapturedRecord[]): Promise<HubRecord[]> {
  const details = new Map<string, Promise<Awaited<ReturnType<typeof rpc.executionsGet>>>>();
  return mapBounded(records, async (record) => {
    if (!session.isCurrent()) throw new Error("Read Records: Owning Connection Changed");
    if (record.recordType === "dokime.log") {
      return decodeFrozenRecords([record])[0];
    }
    const name = required(record.scope?.executionId, "Read Record Execution Identity");
    let pending = details.get(name);
    if (!pending) {
      pending = rpc.executionsGet(session, { name });
      details.set(name, pending);
    }
    const detail = required((await pending).execution, "Read Record Execution");
    const registry = await executionSchemas(session).get(session, detail);
    return decodeFrozenRecords([record], registry, detail.schemas?.records)[0];
  });
}
