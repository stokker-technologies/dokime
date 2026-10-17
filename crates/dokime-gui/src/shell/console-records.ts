/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Streams the console's records into the session-scoped buffer store.
 *
 * The hook owns the subscription, never the rows: a route change that moves
 * the drawer to another scope only swaps which buffer it reads and resumes,
 * and closing the drawer stops the stream without forgetting what it showed.
 */

/* === External Imports === */
import { useCallback, useEffect } from "react";
import { useStore } from "zustand";

/* === Local Imports === */
import { useNativeSessionOptional } from "@/app/native-session/context";
import { openTraceStream, type TraceLevel } from "@/lib/api/native-logs";
import { consoleBufferKey, consoleRecordsStore, type ConsoleRecords, EMPTY_CONSOLE_BUFFER } from "./console-buffer";

/* === Constants === */

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
}): ConsoleRecords {
  const session = useNativeSessionOptional();
  const generation = session?.identity.generation ?? null;
  const key = consoleBufferKey({ cycleId, executionId, level });
  const buffer = useStore(consoleRecordsStore, (state) => state.buffers.get(key) ?? EMPTY_CONSOLE_BUFFER);

  useEffect(() => {
    consoleRecordsStore.getState().bindSession(generation);
  }, [generation]);

  useEffect(() => {
    if (!enabled || !session) return undefined;
    const store = consoleRecordsStore.getState;
    store().bindSession(session.identity.generation);
    const held = store().buffers.get(key) ?? EMPTY_CONSOLE_BUFFER;
    let active = true;
    let retry: number | undefined;
    let stream: Awaited<ReturnType<typeof openTraceStream>> | undefined;
    let afterSequence = held.afterSequence;
    let epoch = held.epoch;
    let reconnectDelay = 1_000;
    let priorDropped = held.dropped;
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
          reconnectDelay = 1_000;
          store().receive(key, page, priorDropped + streamDropped);
        }
        if (active) store().setError(key, "Trace stream ended; reconnecting.");
      } catch (failure) {
        if (!active || !session.identity.isCurrent()) return;
        const code = grpcCode(failure);
        if (code === 11) {
          afterSequence = 0n;
          epoch = undefined;
          priorDropped = 0n;
          streamDropped = 0n;
          store().rewind(key);
          store().setError(key, "Trace history expired; replaying retained events.");
        } else if (code === 12 || unavailable(failure)) {
          reconnect = false;
          store().setError(key, "Tracing is not available for this connection.");
        } else if (code === 3 || code === 7 || code === 16 || code === 8) {
          reconnect = false;
          store().setError(key, "Trace subscription was refused by the backend.");
        } else {
          store().setError(key, "Trace stream unavailable; reconnecting.");
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
  }, [cycleId, enabled, executionId, key, level, session]);

  const clear = useCallback(() => consoleRecordsStore.getState().clear(key), [key]);

  return { clear, dropped: buffer.dropped, error: buffer.error, records: buffer.records };
}

/* === Private Functions === */
function grpcCode(error: unknown): number | undefined {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "grpc" || !("detail" in error)) return undefined;
  const detail = error.detail;
  return detail && typeof detail === "object" && "code" in detail && typeof detail.code === "number" ? detail.code : undefined;
}

function unavailable(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "session" || !("detail" in error)) return false;
  const detail = error.detail;
  return (
    !!detail &&
    typeof detail === "object" &&
    "kind" in detail &&
    (detail.kind === "permission_denied" || detail.kind === "unauthenticated" || detail.kind === "authentication_required")
  );
}
