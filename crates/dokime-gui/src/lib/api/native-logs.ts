/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Library Trace Snapshot and Watch Adapter. */
import type { Event } from "@host/harmos/v1/status/logs_pb.js";
import type { Position } from "@host/harmos/v1/types_pb.js";
import * as rpc from "@/lib/host/client";
import type { Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, Severity } from "@/lib/types";
import { NATIVE_MAX_PAGES, timestampIso } from "./native-values";

const GUI_LOG_BYTES = 262_144;
const GUI_LOG_EVENTS = 256;
const LEVELS = { trace: 0, debug: 1, info: 2, warning: 3, error: 4 };
export type TraceLevel = "trace" | Severity;
export type TraceSelection = { cycleId: string | null; executionId: string | null };
export type TracePage = { epoch: string; records: HubRecord[]; dropped: bigint; nextSequence: bigint };
export class NativeTraceHistoryExpired extends Error {}

export async function openTraceStream(
  session: SessionIdentity,
  request: TraceSelection & { minLevel: TraceLevel; afterSequence: bigint; epoch?: string },
): Promise<AsyncIterable<TracePage> & { cancel: () => Promise<void> }> {
  let cancelled = false;
  let reader: Awaited<ReturnType<typeof rpc.statusTraceEventsWatch>> | undefined;
  return {
    async *[Symbol.asyncIterator]() {
      let at: Wire<Position> | undefined;
      let token = "";
      let dropped = 0n;
      // Resolve One Retained Snapshot, Then Watch From Its Exact Issued Boundary.
      for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
        if (cancelled || !session.isCurrent()) return;
        const reply = await rpc
          .statusTraceEventsList(session, { at, page: { pageSize: GUI_LOG_EVENTS, maxBytes: BigInt(GUI_LOG_BYTES), pageToken: token } })
          .catch(traceFailure);
        if (cancelled || !session.isCurrent()) return;
        const position = reply.resolvedAt;
        if (!position?.history?.store || !position.history.epoch) throw new Error("Read Trace: Missing History Position");
        if (
          at &&
          (at.sequence !== position.sequence || at.history?.store !== position.history.store || at.history.epoch !== position.history.epoch)
        )
          throw new Error("Read Trace: Snapshot Changed Between Pages");
        at = position;
        if (page === 0 && reply.events.length) dropped = reply.events[0].sequence - 1n;
        yield projectTracePage(reply.events, position, dropped, request);
        if (!reply.nextPageToken) break;
        if (reply.nextPageToken === token || page + 1 === NATIVE_MAX_PAGES)
          throw new Error("Read Trace: Snapshot Pagination Did Not Finish");
        token = reply.nextPageToken;
      }
      if (cancelled || !session.isCurrent()) return;
      reader = await rpc
        .statusTraceEventsWatch(session, { at, maxBytes: BigInt(GUI_LOG_BYTES), maxEvents: GUI_LOG_EVENTS })
        .catch(traceFailure);
      if (cancelled || !session.isCurrent()) {
        await reader.cancel();
        return;
      }
      try {
        for await (const batch of reader) {
          if (cancelled || !session.isCurrent()) return;
          const position = batch.resolvedAt;
          if (!position?.history?.store || !position.history.epoch) throw new Error("Watch Trace: Missing History Position");
          yield projectTracePage(batch.events, position, dropped, request);
        }
      } catch (error) {
        traceFailure(error);
      } finally {
        await reader.cancel();
      }
    },
    async cancel() {
      cancelled = true;
      await reader?.cancel();
    },
  };
}

function projectTracePage(
  events: Wire<Event>[],
  at: Wire<Position>,
  dropped: bigint,
  selection: TraceSelection & { minLevel: TraceLevel },
): TracePage {
  return {
    epoch: JSON.stringify(at.history),
    nextSequence: at.sequence,
    dropped,
    records: events
      .filter((event) => {
        const level = logLevel(event.level);
        return (
          LEVELS[level] >= LEVELS[selection.minLevel] &&
          (!selection.executionId || event.fields.execution_id === selection.executionId) &&
          (!selection.cycleId || event.fields.cycle_id === selection.cycleId)
        );
      })
      .map((event) => ({
        record_id: `trace-${event.sequence}`,
        record_type: "log",
        source: event.target,
        level: logSeverity(event.level),
        message: event.message,
        timestamp: timestampIso(event.observedAt) ?? "",
        details: { ...event.fields, trace_level: event.level },
        display: { icon: "clock", tone: "neutral" },
      })),
  };
}
function logLevel(level: string): TraceLevel {
  switch (level.toLowerCase()) {
    case "trace":
      return "trace";
    case "debug":
      return "debug";
    case "warn":
    case "warning":
      return "warning";
    case "error":
      return "error";
    default:
      return "info";
  }
}
function logSeverity(level: string): Severity {
  const value = logLevel(level);
  return value === "trace" ? "debug" : value;
}

function traceFailure(error: unknown): never {
  const text = error instanceof Error ? error.message : JSON.stringify(error);
  if (/history gap|position is foreign, expired or was not issued|page token is expired or mismatched/.test(text))
    throw new NativeTraceHistoryExpired("Trace History Expired; Replay Retained Events");
  throw error;
}
