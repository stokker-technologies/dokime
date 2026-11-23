// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ConnectedSession } from "@/app/native-session/session";
import { openNativeExecutionProgressStream } from "@/lib/api/native-observation";
import { isHostReopen } from "@/lib/rpc/errors";
import { safeNumber } from "@/lib/api/native-values";

const GUI_RECONNECT_MS = 1_000;

/* === Types === */

export interface LiveExecutionProgress {
  testExecutionId: string;
  label: string;
  phase: string;
  completed: number;
  total: number;
}

export function observeExecutionProgress(
  session: ConnectedSession,
  testExecutionId: string,
  setProgress: (progress: LiveExecutionProgress | null) => void,
): () => void {
  let active = true;
  let retry: number | undefined;
  let stream: Awaited<ReturnType<typeof openNativeExecutionProgressStream>> | undefined;
  const connect = async () => {
    if (!active || !session.identity.isCurrent()) {
      return;
    }
    let reconnect = true;
    // A Host Deadline Reopens The Watch At Once And Keeps The Shown Progress.
    let reopen = false;
    try {
      stream = await openNativeExecutionProgressStream({ ...session.identity, own: session.own }, testExecutionId);
      if (!active || !session.identity.isCurrent()) {
        await stream.cancel();
        return;
      }
      for await (const frame of stream) {
        if (!active || !session.identity.isCurrent()) {
          break;
        }
        const value = frame.progress;
        setProgress(
          !value
            ? null
            : {
                testExecutionId: value.testExecutionId,
                label: value.label,
                phase: value.phase,
                completed: safeNumber(value.completed, "execution progress completed"),
                total: safeNumber(value.total, "execution progress total"),
              },
        );
        if (frame.sealed) {
          reconnect = false;
          break;
        }
      }
    } catch (error) {
      const detail = error && typeof error === "object" && "detail" in error ? error.detail : null;
      const refused =
        error &&
        typeof error === "object" &&
        "kind" in error &&
        error.kind === "grpc" &&
        detail &&
        typeof detail === "object" &&
        "code" in detail &&
        (detail.code === 3 || detail.code === 9);
      reconnect = !refused;
      reopen = isHostReopen(error);
      if (active && !reopen) {
        setProgress(null);
      }
    } finally {
      stream = undefined;
      if (reconnect && active && session.identity.isCurrent()) {
        if (reopen) {
          void connect();
        } else {
          retry = window.setTimeout(() => void connect(), GUI_RECONNECT_MS);
        }
      }
    }
  };

  void connect();
  return () => {
    active = false;
    if (retry !== undefined) {
      window.clearTimeout(retry);
    }
    void stream?.cancel();
  };
}
