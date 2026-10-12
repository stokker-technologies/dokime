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
import { openNativeExecutionProgressStream } from "@/lib/api/native-observation";
import { safeNumber } from "@/lib/api/native-values";

/* === Types === */

export interface LiveExecutionProgress {
  testExecutionId: string;
  stepId: string;
  phase: string;
  completed: number;
  total: number;
  elapsedMs: number;
  remainingMs: number | null;
}

/* === Public Functions === */

export function useExecutionProgress(testExecutionId: string | null): LiveExecutionProgress | null {
  /* A frozen report mounts the same workspace without a connection: there is
     no stream to open, and the header simply states no activity. */
  const session = useNativeSessionOptional();
  const [progress, setProgress] = useState<LiveExecutionProgress | null>(null);

  useEffect(() => {
    setProgress(null);
    if (!testExecutionId || !session) return;

    let active = true;
    let retry: number | undefined;
    let stream: Awaited<ReturnType<typeof openNativeExecutionProgressStream>> | undefined;
    const connect = async () => {
      if (!active || !session.identity.isCurrent()) return;
      try {
        stream = await openNativeExecutionProgressStream(
          { ...session.identity, own: session.own },
          testExecutionId,
        );
        for await (const frame of stream) {
          if (!active || !session.identity.isCurrent()) break;
          const value = frame.progress;
          setProgress(
            frame.cleared || !value
              ? null
              : {
                  testExecutionId: value.testExecutionId,
                  stepId: value.stepId,
                  phase: value.phase,
                  completed: safeNumber(value.completed, "execution progress completed"),
                  total: safeNumber(value.total, "execution progress total"),
                  elapsedMs: safeNumber(value.elapsedMs, "execution progress elapsed"),
                  remainingMs:
                    value.remainingMs === undefined
                      ? null
                      : safeNumber(value.remainingMs, "execution progress ETA"),
                },
          );
        }
      } catch {
        if (active) setProgress(null);
      } finally {
        stream = undefined;
        if (active && session.identity.isCurrent()) {
          retry = window.setTimeout(() => void connect(), 1_000);
        }
      }
    };

    void connect();
    return () => {
      active = false;
      if (retry !== undefined) window.clearTimeout(retry);
      void stream?.cancel();
    };
  }, [session, testExecutionId]);

  return progress;
}
