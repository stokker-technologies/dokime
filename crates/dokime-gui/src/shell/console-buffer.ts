/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The console's records, held for the session rather than for a route.
 *
 * The drawer lives in the root shell, but the stream it reads is keyed by the
 * scope the route resolves to: leaving a cycle page for an execution page
 * changes the key. Each key keeps its own bounded ring and resume cursor here,
 * so returning to a scope shows what it already had and resumes from where it
 * stopped instead of starting empty. A new native session starts over.
 *
 * Pure: no React, no path aliases, so the Node test runner imports it as is.
 */

/* === External Imports === */

import { createStore } from "zustand/vanilla";

/* === Local Imports === */

import type { HubRecord } from "@/lib/types";

/* === Types === */

export type ConsoleStreamKey = {
  cycleId: string | null;
  executionId: string | null;
  level: string;
};

/** One scope's ring and the cursor its stream resumes from. */
export type ConsoleBuffer = {
  afterSequence: bigint;
  dropped: bigint;
  epoch: string | undefined;
  error: string | null;
  records: HubRecord[];
};

export type ConsolePage = {
  dropped: bigint;
  epoch: string;
  nextSequence: bigint;
  records: HubRecord[];
};

type ConsoleRecordsState = {
  /** Least recently used first. */
  buffers: Map<string, ConsoleBuffer>;
  /** The free-text filter; a way of reading, kept while the drawer closes. */
  filter: string;
  /** The native session generation the buffers belong to. */
  session: string | null;
};

type ConsoleRecordsActions = {
  /** Start over when the native session changes; keep everything otherwise. */
  bindSession: (session: string | null) => void;
  /** Empty one scope's rows; its cursor stays, so nothing is replayed. */
  clear: (key: string) => void;
  /**
   * Replay from the start after the backend expired the cursor. The rows and
   * their epoch stay: a replay from the same epoch merges into them, one from
   * another epoch replaces them.
   */
  rewind: (key: string) => void;
  receive: (key: string, page: ConsolePage, dropped: bigint) => void;
  setError: (key: string, error: string | null) => void;
  setFilter: (filter: string) => void;
};

export type ConsoleRecordsStore = ConsoleRecordsState & ConsoleRecordsActions;

/* === Constants === */

const GUI_CONSOLE_BUFFERS = 4;
const GUI_CONSOLE_RECORDS = 5_000;
const GUI_CONSOLE_RETENTION_BYTES = 2_097_152;

export const RETAINED_RECORDS = GUI_CONSOLE_RECORDS;
export const RETAINED_BYTES = GUI_CONSOLE_RETENTION_BYTES;

/** Scopes kept at once; the least recently read one goes first. */
export const RETAINED_BUFFERS = GUI_CONSOLE_BUFFERS;

export const EMPTY_CONSOLE_BUFFER: ConsoleBuffer = {
  afterSequence: 0n,
  dropped: 0n,
  epoch: undefined,
  error: null,
  records: [],
};

/* === Public Functions === */

export function consoleBufferKey({ cycleId, executionId, level }: ConsoleStreamKey): string {
  return `${cycleId ?? "*"}/${executionId ?? "*"}/${level}`;
}

/**
 * `incoming` folded into `current`, oldest first, once per sequence, then
 * trimmed to the ring. A reconnect may replay rows already held, so the merge
 * deduplicates rather than appends.
 */
export function mergeConsoleRecords(current: HubRecord[], incoming: HubRecord[]): HubRecord[] {
  if (incoming.length === 0) return current;
  const last = current.at(-1);
  const ordered =
    !last || recordSequence(incoming[0]) > recordSequence(last)
      ? [...current, ...incoming]
      : [...new Map([...current, ...incoming].map((record) => [record.record_id, record])).values()]
          .map((record) => ({ record, sequence: recordSequence(record) }))
          .sort((left, right) => compareSequence(left.sequence, right.sequence))
          .map(({ record }) => record);
  return retainedConsoleRecords(ordered);
}

/** The newest records that fit both the count and the byte budget. */
export function retainedConsoleRecords(records: HubRecord[]): HubRecord[] {
  let bytes = 0;
  let start = records.length;
  while (start > 0 && records.length - start < RETAINED_RECORDS) {
    const record = records[start - 1];
    const size = 256 + 2 * (record.source.length + (record.message?.length ?? 0) + JSON.stringify(record.details).length);
    if (bytes + size > RETAINED_BYTES) break;
    bytes += size;
    start -= 1;
  }
  return start === 0 ? records : records.slice(start);
}

/** A fresh store; the shell holds one for the window's lifetime. */
export function createConsoleRecordsStore() {
  return createStore<ConsoleRecordsStore>()((set) => {
    const update = (key: string, change: (buffer: ConsoleBuffer) => ConsoleBuffer) =>
      set((state) => ({ buffers: touched(state.buffers, key, change) }));

    return {
      buffers: new Map(),
      filter: "",
      session: null,
      bindSession: (session) => set((state) => (state.session === session ? state : { buffers: new Map(), session })),
      clear: (key) => update(key, (buffer) => ({ ...buffer, records: [] })),
      receive: (key, page, dropped) =>
        update(key, (buffer) => {
          const held = buffer.epoch !== undefined && buffer.epoch !== page.epoch ? [] : buffer.records;
          return {
            afterSequence: page.nextSequence,
            dropped,
            epoch: page.epoch,
            error: null,
            records: mergeConsoleRecords(held, page.records),
          };
        }),
      rewind: (key) => update(key, (buffer) => ({ ...buffer, afterSequence: 0n, dropped: 0n })),
      setError: (key, error) => update(key, (buffer) => ({ ...buffer, error })),
      setFilter: (filter) => set({ filter }),
    };
  });
}

/* === Private Functions === */

function recordSequence(record: HubRecord): bigint {
  const digits = record.record_id.slice(record.record_id.lastIndexOf("-") + 1);
  return /^\d+$/.test(digits) ? BigInt(digits) : -1n;
}

function compareSequence(left: bigint, right: bigint): number {
  if (left === right) return 0;
  return left < right ? -1 : 1;
}

/** The map with `key` changed and moved to the most recent end, then trimmed. */
function touched(
  buffers: Map<string, ConsoleBuffer>,
  key: string,
  change: (buffer: ConsoleBuffer) => ConsoleBuffer,
): Map<string, ConsoleBuffer> {
  const next = new Map(buffers);
  const current = next.get(key) ?? EMPTY_CONSOLE_BUFFER;
  next.delete(key);
  next.set(key, change(current));
  while (next.size > RETAINED_BUFFERS) {
    const oldest = next.keys().next().value;
    if (oldest === undefined) break;
    next.delete(oldest);
  }
  return next;
}

/** Rows Returned By The Live Source To The Shared Console Surface. */
export type ConsoleRecords = {
  clear: () => void;
  dropped: bigint;
  records: HubRecord[];
  error: string | null;
};

/** One Per Window: The Console Rows Outlive Every Route. */
export const consoleRecordsStore = createConsoleRecordsStore();
