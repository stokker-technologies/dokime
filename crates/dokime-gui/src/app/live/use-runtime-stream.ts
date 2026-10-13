/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import {
  invalidateRuntimeProjectionQueries,
  workflowQueryKeys,
} from "@/features/cycle/server-state";
import { openNativeRuntimeStream } from "@/lib/api/native-observation";
import type { RuntimeStateFrame } from "@/lib/api/native-observation";
import { useHubStore } from "@/stores/hub-store";
import {
  createRuntimeProjectionRefreshCoordinator,
  type RuntimeProjectionRefreshCoordinator,
} from "./runtime-refresh";

/* === Runtime stream ===
   The single live subscription. Native frames carry an observed cursor and
   lifecycle flags; the generation-scoped queries re-read each affected
   projection through the same native session. */

/* === Public Functions === */

export function useRuntimeStream(): void {
  /* Optional, because the frozen report mounts the same shell with nothing to
     subscribe to. The effect below simply never connects. */
  const session = useNativeSessionOptional();
  const queryClient = useQueryClient();
  const setHubState = useHubStore((state) => state.setHubState);

  useEffect(() => {
    if (!session) return undefined;
    let active = true;
    let stream: Awaited<ReturnType<typeof openNativeRuntimeStream>> | undefined;
    let retry: number | undefined;
    const refreshCoordinator = createRuntimeProjectionRefreshCoordinator(async () => {
      await Promise.all([
        invalidateRuntimeProjectionQueries(queryClient),
        queryClient.invalidateQueries(
          { queryKey: workflowQueryKeys.runtimeShell() },
          { cancelRefetch: false },
        ),
      ]);
    });

    const connect = async () => {
      if (!active || !session.identity.isCurrent()) return;
      try {
        stream = await openNativeRuntimeStream(
          { ...session.identity, own: session.own },
          undefined,
        );
        if (!active || !session.identity.isCurrent()) {
          await stream.cancel();
          return;
        }
        setHubState({ connection: "connected", error: null });
        for await (const frame of stream) {
          if (!active) break;
          handleRuntimeFrame(
            frame,
            queryClient,
            refreshCoordinator,
            setHubState,
            session.identity.generation,
            () => active && session.identity.isCurrent(),
          );
        }
      } catch (error: unknown) {
        if (active && session.identity.isCurrent()) {
          setHubState({
            error: error instanceof Error ? error.message : String(error),
          });
        }
      } finally {
        stream = undefined;
        if (active && session.identity.isCurrent()) {
          setHubState({ connection: "reconnecting" });
          retry = window.setTimeout(() => void connect(), 1_000);
        }
      }
    };

    void connect();
    return () => {
      active = false;
      if (retry !== undefined) window.clearTimeout(retry);
      refreshCoordinator.dispose();
      void stream?.cancel();
    };
  }, [queryClient, session, setHubState]);
}

/* === Private Functions === */

function handleRuntimeFrame(
  frame: RuntimeStateFrame,
  queryClient: ReturnType<typeof useQueryClient>,
  refreshCoordinator: RuntimeProjectionRefreshCoordinator,
  setHubState: (patch: {
    connection?: "connecting" | "connected" | "reconnecting";
    error?: string | null;
    lastStreamFrameAt?: number | null;
  }) => void,
  sessionGeneration: string,
  isActive: () => boolean,
) {
  setHubState({ lastStreamFrameAt: Date.now(), error: null });
  if (frame.shuttingDown) {
    setHubState({ connection: "reconnecting" });
  }
  if (frame.resynchronized || frame.cursor) {
    const ensureTrailing =
      queryClient.isFetching({
        predicate: (query) =>
          query.queryKey[0] === workflowQueryKeys.all[0] &&
          query.queryKey.at(-1) === sessionGeneration,
      }) > 0;
    void refreshCoordinator.request({ ensureTrailing }).catch((error: unknown) => {
      if (!isActive()) return;
      setHubState({ error: error instanceof Error ? error.message : String(error) });
    });
  }
}
