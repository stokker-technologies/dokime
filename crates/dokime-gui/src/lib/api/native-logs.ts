/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, Severity } from "@/lib/types";

const GUI_LOG_BYTES = 262_144;
const GUI_LOG_EVENTS = 256;

const MIN_LEVEL: Record<TraceLevel, LogLevel> = {
  debug: { HarmosVariant: { case: "Debug", value: {} } },
  error: { HarmosVariant: { case: "Error", value: {} } },
  info: { HarmosVariant: { case: "Info", value: {} } },
  trace: { HarmosVariant: { case: "Trace", value: {} } },
  warning: { HarmosVariant: { case: "Warn", value: {} } },
};

type Request = Parameters<typeof rpc.backendWatchTrace>[1];
type LogLevel = Request["minLevel"];
export type TraceLevel = "trace" | Severity;
export type TraceSelection = { cycleId: string | null; executionId: string | null };
export type TracePage = {
  epoch: string;
  records: HubRecord[];
  dropped: bigint;
  nextSequence: bigint;
};

export async function openTraceStream(
  session: SessionIdentity,
  request: TraceSelection & { minLevel: TraceLevel; afterSequence: bigint; epoch?: string },
): Promise<AsyncIterable<TracePage> & { cancel: () => Promise<void> }> {
  const scope: Request["scope"] = request.executionId
    ? { HarmosVariant: { case: "Execution", value: { field0: request.executionId } } }
    : request.cycleId
      ? { HarmosVariant: { case: "Cycle", value: { field0: request.cycleId } } }
      : { HarmosVariant: { case: "Instance", value: {} } };
  const reader = await rpc.backendWatchTrace(session, {
    scope,
    epoch: request.epoch,
    minLevel: MIN_LEVEL[request.minLevel],
    afterSequence: request.afterSequence,
    maxEvents: GUI_LOG_EVENTS,
    maxBytes: GUI_LOG_BYTES,
    follow: true,
  });
  return {
    async *[Symbol.asyncIterator]() {
      // The Transport Checks Final Status And Pulls One Bounded Batch At A Time.
      for await (const batch of reader) {
        yield {
          epoch: batch.epoch,
          dropped: batch.dropped,
          nextSequence: batch.nextSequence,
          records: batch.events.map((event) => {
            const level = logSeverity(event.level);
            return {
              details: {
                ...Object.fromEntries(event.fields.map(({ key, value }) => [key, value])),
                ...(event.pluginId ? { plugin: event.pluginId } : {}),
                ...(event.cycleId ? { cycle: event.cycleId } : {}),
                ...(event.executionId ? { execution: event.executionId } : {}),
                trace_level: event.level?.HarmosVariant?.case ?? "Info",
              },
              display: { icon: "clock", tone: "neutral" },
              level,
              message: event.message,
              record_id: `trace-${event.sequence}`,
              record_type: "log",
              source: event.target,
              timestamp: event.time ? timestampIso(event.time) : "",
            } as HubRecord;
          }),
        };
      }
    },
    cancel: () => reader.cancel(),
  };
}

function logSeverity(level: LogLevel): Severity {
  switch (level?.HarmosVariant?.case) {
    case "Error":
      return "error";
    case "Warn":
      return "warning";
    case "Debug":
    case "Trace":
      return "debug";
    default:
      return "info";
  }
}

function timestampIso(time: { seconds: bigint; nanos: number }): string {
  return new Date(Number(time.seconds) * 1000 + Math.floor(time.nanos / 1_000_000)).toISOString();
}
