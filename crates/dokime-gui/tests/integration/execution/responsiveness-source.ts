/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Bounded Record Pages For The Browser Responsiveness Workload. */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { RecordTail } from "@/lib/api/native-evidence";
import type { HubRecord } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

const pending: RecordTail[] = [];
let wake: (() => void) | undefined;
let sequence = 0;
export let detailReads = 0;

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function record(): HubRecord {
  sequence += 1;
  return {
    record_id: `record-${sequence}`,
    record_type: "measurement",
    source: "fixture",
    level: "info",
    timestamp: new Date(1_800_000_000_000 + sequence).toISOString(),
    sequence,
    scope: { test_cycle_id: "cycle" },
    display: { icon: "clock", tone: "info" },
    message: `Sample ${sequence}`,
    details: {
      sample: sequence,
      payload: "payload ".repeat(256),
      toJSON() {
        detailReads += 1;
        return { payload: this.payload };
      },
    },
  } as HubRecord;
}

export async function fetchNativeRecordTail(): Promise<RecordTail> {
  return { records: Array.from({ length: 1_000 }, record), cursor: null };
}

export async function openNativeRecordStream() {
  let cancelled = false;
  return {
    async cancel() {
      cancelled = true;
      wake?.();
    },
    async *[Symbol.asyncIterator]() {
      while (!cancelled) {
        if (!pending.length)
          await new Promise<void>((resolve) => {
            wake = resolve;
          });
        if (cancelled) return;
        const page = pending.shift();
        if (page) yield page;
      }
    },
  };
}

export function appendPage() {
  const next = record();
  pending.push({
    records: [next],
    cursor: { positions: [{ history: { store: "fixture", epoch: "fixture" }, sequence: BigInt(sequence) }], revisions: [] },
  });
  wake?.();
  return next.record_id;
}

/* === Private Functions === */

/* === Entry Point === */
