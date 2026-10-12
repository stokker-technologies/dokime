/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The hub's own log, as console rows.
 *
 * `Backend.WatchLogs(min_level, replay_lines)` replays a bounded tail and then
 * follows it. The level the operator picked is the stream's `min_level`, so a
 * threshold is a narrower subscription rather than a client-side filter over
 * everything the backend could have sent; changing it re-opens the stream.
 * `dropped` is cumulative and counts the matching lines the client fell behind
 * on, which the drawer states rather than hides.
 *
 * `trace` has no `Severity` of its own — the projections the GUI draws start
 * at `debug` — so a trace frame arrives as the debug row it reads as, while
 * the subscription itself still asks for trace and below.
 */

/* === Local Imports === */

import { NativeApiUnsupportedError } from "@/lib/api/native-values";
import * as rpc from "@/lib/rpc/generated/client";
import { LogLevel } from "@/lib/rpc/generated/dokime/v1/backend_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { HubRecord, Severity } from "@/lib/types";

/* === Types === */

/** The levels the console offers, which is one more than a record carries. */
export type InstanceLogLevel = "trace" | Severity;

export type InstanceLogFrame = {
  /** RFC3339. */
  time: string;
  level: Severity;
  /** The emitting module, which reads as the row's source. */
  target: string;
  message: string;
  fields: Record<string, string>;
  /** Cumulative count of matching lines the client fell behind on. */
  dropped: number;
};

/* === Constants === */

const MIN_LEVEL: Record<InstanceLogLevel, LogLevel> = {
  debug: LogLevel.DEBUG,
  error: LogLevel.ERROR,
  info: LogLevel.INFO,
  trace: LogLevel.TRACE,
  warning: LogLevel.WARN,
};

/* === Public Functions === */

/** Whether this client can subscribe to the backend's own log at all. */
export function instanceLogsSupported(): boolean {
  return typeof rpc.backendWatchLogs === "function";
}

export async function openInstanceLogStream(
  session: SessionIdentity,
  request: { minLevel: InstanceLogLevel; replayLines?: number },
): Promise<AsyncIterable<InstanceLogFrame> & { cancel: () => Promise<void> }> {
  if (!instanceLogsSupported()) {
    throw new NativeApiUnsupportedError(
      "Watch instance logs",
      "Backend.WatchLogs (not available on this backend)",
    );
  }
  const reader = await rpc.backendWatchLogs(session, {
    minLevel: MIN_LEVEL[request.minLevel],
    ...(request.replayLines === undefined ? {} : { replayLines: request.replayLines }),
  });
  return {
    async *[Symbol.asyncIterator]() {
      // One frame is consumed before the next is pulled: the stream reader
      // acknowledges as it iterates, so there is never more than one
      // outstanding pull.
      for await (const frame of reader) {
        yield {
          dropped: Number(frame.dropped),
          fields: { ...frame.fields },
          level: logSeverity(frame.level),
          message: frame.message,
          target: frame.target,
          time: frame.time ? timestampIso(frame.time) : "",
        };
      }
    },
    cancel: () => reader.cancel(),
  };
}

/** One log frame as the console row the drawer already draws. */
export function instanceLogRecord(frame: InstanceLogFrame, sequence: number): HubRecord {
  return {
    details: frame.fields,
    display: { icon: "activity", label: frame.level, tone: "neutral" },
    level: frame.level,
    message: frame.message,
    record_id: `instance-log-${sequence}`,
    record_type: "log",
    source: frame.target,
    timestamp: frame.time,
  } as HubRecord;
}

/* === Private Functions === */

/** Trace has no projection of its own; it reads as the debug row it is. */
function logSeverity(level: LogLevel): Severity {
  switch (level) {
    case LogLevel.ERROR:
      return "error";
    case LogLevel.WARN:
      return "warning";
    case LogLevel.DEBUG:
    case LogLevel.TRACE:
      return "debug";
    default:
      return "info";
  }
}

function timestampIso(time: { seconds: bigint; nanos: number }): string {
  return new Date(Number(time.seconds) * 1000 + Math.floor(time.nanos / 1_000_000)).toISOString();
}
