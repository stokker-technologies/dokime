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

import { useNativeSessionOptional } from "@/app/native-session/context";
import { openNativeExecutionProgressStream } from "@/lib/api/native-observation";
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

/* === Public Functions === */

export function useExecutionProgress(testExecutionId: string | null): LiveExecutionProgress | null {
  /* A frozen report mounts the same workspace without a connection: there is
     no stream to open, and the header simply states no activity. */
  const session = useNativeSessionOptional();
  const [progress, setProgress] = useState<LiveExecutionProgress | null>(null);

  useEffect(() => {
    setProgress(null);
    if (!testExecutionId || !session) {
      return;
    }

    let active = true;
    let retry: number | undefined;
    let stream: Awaited<ReturnType<typeof openNativeExecutionProgressStream>> | undefined;
    const connect = async () => {
      if (!active || !session.identity.isCurrent()) {
        return;
      }
      let reconnect = true;
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
          if (frame.finished) {
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
        if (active) {
          setProgress(null);
        }
      } finally {
        stream = undefined;
        if (reconnect && active && session.identity.isCurrent()) {
          retry = window.setTimeout(() => void connect(), GUI_RECONNECT_MS);
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
  }, [session, testExecutionId]);

  return progress;
}
