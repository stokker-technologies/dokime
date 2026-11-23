/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The instance console's trace, held for the session rather than for a route.
 *
 * Each level keeps its own bounded ring here, so returning to a level shows
 * what it already had instead of starting empty. A new native session starts
 * over. Cycle and execution scopes read the shared records query instead.
 */

/* === External Imports === */

import type { NotificationFailure } from "@/lib/notification-details";
import { createStore } from "zustand/vanilla";

/* === Local Imports === */

import type { TracePage } from "@/lib/api/native-logs";
import type { HubRecord } from "@/lib/types";

/* === Types === */

/** One level's ring and the trace history it was read from. */
export type ConsoleBuffer = {
  dropped: bigint;
  epoch: string | undefined;
  error: string | null;
  failure?: NotificationFailure;
  records: HubRecord[];
};

/** The Rows the Drawer Shows, From the Trace or the Records Query. */
export type ConsoleRecords = {
  clear: () => void;
  dropped: bigint;
  records: HubRecord[];
  error: string | null;
  failure?: NotificationFailure;
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
  /** A replay from the same epoch merges into the rows, one from another epoch replaces them. */
  receive: (key: string, page: TracePage) => void;
  setError: (key: string, error: string | null, failure?: NotificationFailure) => void;
  setFilter: (filter: string) => void;
};

export type ConsoleRecordsStore = ConsoleRecordsState & ConsoleRecordsActions;

/* === Constants === */

const GUI_CONSOLE_BUFFERS = 4;
const GUI_CONSOLE_RECORDS = 5_000;
const GUI_CONSOLE_RETENTION_BYTES = 2_097_152;

export const RETAINED_RECORDS = GUI_CONSOLE_RECORDS;
export const RETAINED_BYTES = GUI_CONSOLE_RETENTION_BYTES;

/** Levels kept at once; the least recently read one goes first. */
export const RETAINED_BUFFERS = GUI_CONSOLE_BUFFERS;

export const EMPTY_CONSOLE_BUFFER: ConsoleBuffer = {
  dropped: 0n,
  epoch: undefined,
  error: null,
  records: [],
};

/* === Public Functions === */

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
      receive: (key, page) =>
        update(key, (buffer) => {
          const held = buffer.epoch !== undefined && buffer.epoch !== page.epoch ? [] : buffer.records;
          return { dropped: page.dropped, epoch: page.epoch, error: null, records: mergeConsoleRecords(held, page.records) };
        }),
      setError: (key, error, failure) => update(key, (buffer) => ({ ...buffer, error, failure })),
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

/** One Per Window: The Console Rows Outlive Every Route. */
export const consoleRecordsStore = createConsoleRecordsStore();
