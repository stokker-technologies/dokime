/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Streams the backend's trace into the session-scoped buffer store.
 *
 * The subscription owns no rows: changing the level only swaps which buffer
 * the drawer reads, and closing the drawer stops the stream without
 * forgetting what it showed.
 */

/* === External Imports === */
import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";

/* === Local Imports === */
import type { SessionIdentity } from "@/lib/rpc/transport";
import { openTraceStream, NativeTraceHistoryExpired, type TraceLevel } from "@/lib/api/native-logs";
import { normalizeRpcError } from "@/lib/rpc/errors";
import { consoleRecordsStore } from "@/shell/console-buffer";

/* === Constants === */

const GUI_CONSOLE_RECONNECT_MAX_MS = 30_000;
const GUI_RECONNECT_MS = 1_000;

/* === Public Functions === */

export function observeConsoleRecords(identity: SessionIdentity, level: TraceLevel): () => void {
  const store = consoleRecordsStore.getState;
  store().bindSession(identity.generation);
  let active = true;
  let retry: number | undefined;
  let stream: Awaited<ReturnType<typeof openTraceStream>> | undefined;
  let reconnectDelay = GUI_RECONNECT_MS;

  const connect = async () => {
    if (!active || !identity.isCurrent()) {
      return;
    }
    let reconnect = true;
    try {
      const opened = await openTraceStream(identity, level);
      if (!active || !identity.isCurrent()) {
        await opened.cancel();
        return;
      }
      stream = opened;
      for await (const page of opened) {
        if (!active || !identity.isCurrent()) {
          break;
        }
        reconnectDelay = GUI_RECONNECT_MS;
        store().receive(level, page);
      }
      if (active) {
        store().setError(level, "Trace stream ended; reconnecting.");
      }
    } catch (failure) {
      if (!active || !identity.isCurrent()) {
        return;
      }
      const code = grpcCode(failure);
      const record = failureOf(normalizeRpcError(failure), "Trace Stream Failed");
      toast.error(record, { context: "background" });
      const cause = record.message;
      if (code === 11 || failure instanceof NativeTraceHistoryExpired) {
        store().setError(level, `Trace history expired; replaying retained events. ${cause}`, record);
      } else if (code === 12 || unavailable(failure)) {
        reconnect = false;
        store().setError(level, `Tracing is not available for this connection. ${cause}`, record);
      } else if (code === 3 || code === 7 || code === 16 || code === 8) {
        reconnect = false;
        store().setError(level, `Trace subscription was refused by the backend. ${cause}`, record);
      } else {
        store().setError(level, `Trace stream unavailable; reconnecting. ${cause}`, record);
      }
    } finally {
      const closing = stream;
      stream = undefined;
      await closing?.cancel().catch(() => undefined);
      if (reconnect && active && identity.isCurrent()) {
        retry = window.setTimeout(() => void connect(), reconnectDelay);
        reconnectDelay = Math.min(GUI_CONSOLE_RECONNECT_MAX_MS, reconnectDelay * 2);
      }
    }
  };
  void connect();
  return () => {
    active = false;
    if (retry !== undefined) {
      window.clearTimeout(retry);
    }
    void stream?.cancel().catch(() => undefined);
  };
}

/* === Private Functions === */
function grpcCode(error: unknown): number | undefined {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "grpc" || !("detail" in error)) {
    return undefined;
  }
  const detail = error.detail;
  return detail && typeof detail === "object" && "code" in detail && typeof detail.code === "number" ? detail.code : undefined;
}

function unavailable(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "session" || !("detail" in error)) {
    return false;
  }
  const detail = error.detail;
  return (
    !!detail &&
    typeof detail === "object" &&
    "kind" in detail &&
    (detail.kind === "permission_denied" || detail.kind === "unauthenticated" || detail.kind === "authentication_required")
  );
}
