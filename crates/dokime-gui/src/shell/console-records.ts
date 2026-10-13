/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */
import { useEffect, useState } from "react";

/* === Local Imports === */
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { openTraceStream, type TraceLevel } from "@/lib/api/native-logs";
import type { HubRecord } from "@/lib/types";

/* === Constants === */
const RETAINED_RECORDS = 5_000;
const RETAINED_BYTES = 2 * 1024 * 1024;
const NO_RECORDS: HubRecord[] = [];

/* === Hooks === */
export function useConsoleRecords({
  cycleId,
  enabled,
  executionId,
  level,
}: {
  cycleId: string | null;
  enabled: boolean;
  executionId: string | null;
  level: TraceLevel;
}): { dropped: bigint; records: HubRecord[]; error: string | null } {
  const session = useNativeSessionOptional();
  const [records, setRecords] = useState<HubRecord[]>(NO_RECORDS);
  const [dropped, setDropped] = useState(0n);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setRecords(NO_RECORDS);
    setDropped(0n);
    setError(null);
    if (!enabled || !session) return undefined;
    let active = true;
    let retry: number | undefined;
    let stream: Awaited<ReturnType<typeof openTraceStream>> | undefined;
    let afterSequence = 0n;
    let epoch: string | undefined;
    let reconnectDelay = 1_000;
    let priorDropped = 0n;
    let streamDropped = 0n;

    const connect = async () => {
      if (!active || !session.identity.isCurrent()) return;
      let reconnect = true;
      try {
        const opened = await openTraceStream(session.identity, {
          cycleId,
          executionId,
          minLevel: level,
          afterSequence,
          epoch,
        });
        stream = opened;
        if (!active || !session.identity.isCurrent()) {
          await opened.cancel();
          return;
        }
        priorDropped += streamDropped;
        streamDropped = 0n;
        for await (const page of opened) {
          if (!active || !session.identity.isCurrent()) break;
          afterSequence = page.nextSequence;
          epoch = page.epoch;
          streamDropped = page.dropped;
          setDropped(priorDropped + streamDropped);
          setError(null);
          reconnectDelay = 1_000;
          if (page.records.length > 0) {
            setRecords((current) => retained([...current, ...page.records]));
          }
        }
        if (active) setError("Trace stream ended; reconnecting.");
      } catch (failure) {
        if (!active || !session.identity.isCurrent()) return;
        const code = grpcCode(failure);
        if (code === 11) {
          afterSequence = 0n;
          epoch = undefined;
          priorDropped = 0n;
          streamDropped = 0n;
          setRecords(NO_RECORDS);
          setDropped(0n);
          setError("Trace history expired; replaying retained events.");
        } else if (code === 12 || unavailable(failure)) {
          reconnect = false;
          setError("Tracing is not available for this connection.");
        } else if (code === 3 || code === 7 || code === 16 || code === 8) {
          reconnect = false;
          setError("Trace subscription was refused by the backend.");
        } else {
          setError("Trace stream unavailable; reconnecting.");
        }
      } finally {
        const closing = stream;
        stream = undefined;
        await closing?.cancel().catch(() => undefined);
        if (reconnect && active && session.identity.isCurrent()) {
          retry = window.setTimeout(() => void connect(), reconnectDelay);
          reconnectDelay = Math.min(30_000, reconnectDelay * 2);
        }
      }
    };
    void connect();
    return () => {
      active = false;
      if (retry !== undefined) window.clearTimeout(retry);
      void stream?.cancel().catch(() => undefined);
    };
  }, [cycleId, enabled, executionId, level, session]);

  return { dropped, records, error };
}

/* === Private Functions === */
function retained(records: HubRecord[]): HubRecord[] {
  let bytes = 0;
  let start = records.length;
  while (start > 0 && records.length - start < RETAINED_RECORDS) {
    const record = records[start - 1];
    const size =
      256 +
      2 *
        (record.source.length +
          (record.message?.length ?? 0) +
          JSON.stringify(record.details).length);
    if (bytes + size > RETAINED_BYTES) break;
    bytes += size;
    start -= 1;
  }
  return records.slice(start);
}

function grpcCode(error: unknown): number | undefined {
  if (
    !error ||
    typeof error !== "object" ||
    !("kind" in error) ||
    error.kind !== "grpc" ||
    !("detail" in error)
  )
    return undefined;
  const detail = error.detail;
  return detail && typeof detail === "object" && "code" in detail && typeof detail.code === "number"
    ? detail.code
    : undefined;
}

function unavailable(error: unknown): boolean {
  if (
    !error ||
    typeof error !== "object" ||
    !("kind" in error) ||
    error.kind !== "session" ||
    !("detail" in error)
  )
    return false;
  const detail = error.detail;
  return (
    !!detail &&
    typeof detail === "object" &&
    "kind" in detail &&
    (detail.kind === "permission_denied" ||
      detail.kind === "unauthenticated" ||
      detail.kind === "authentication_required")
  );
}
