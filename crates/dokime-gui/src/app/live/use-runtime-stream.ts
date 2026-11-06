/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { toast } from "@/lib/notifications";
import { failureOf } from "@/lib/notification-details";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { invalidateRuntimeDocuments, invalidateRuntimeProjectionQueries, workflowQueryKeys } from "@/features/cycle/server-state";
import { cyclesHomeQueryKeys } from "@/features/cycles-home/cycles-queries";
import type { RuntimeStateFrame } from "@/lib/api/native-observation";
import { openNativeRuntimeStream } from "@/lib/api/native-observation";
import { operationsFromFrame } from "@/shell/status-bar-operations";
import { type BackendOperationsPatch, useHubStore } from "@/stores/hub-store";
import {
  createExecutionFileRefresh,
  createRuntimeProjectionRefreshCoordinator,
  createThrottledRefresh,
  type RuntimeProjectionRefreshCoordinator,
} from "./runtime-refresh";

/* === Constants === */

const GUI_FILE_REFRESH_MS = 250;
const GUI_DOCUMENT_REFRESH_MS = 5_000;
const GUI_RECONNECT_MS = 1_000;

/** The most often a running cycle re-reads its open documents. */
const DOCUMENT_REFRESH_MS = GUI_DOCUMENT_REFRESH_MS;

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
    const fileCoordinator = createRuntimeProjectionRefreshCoordinator(async () => {
      if (!active || !session.identity.isCurrent()) return;
      await queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "execution-files"] }, { cancelRefetch: false });
    });
    const fileRefresh = createExecutionFileRefresh(() => {
      const ensureTrailing = queryClient.isFetching({ queryKey: [...workflowQueryKeys.all, "execution-files"] }) > 0;
      void fileCoordinator.request({ ensureTrailing }).catch((error: unknown) => {
        if (!active || !session.identity.isCurrent()) return;
        toast.error(failureOf(error, "Execution Files Failed"), { context: "background" });
      });
    }, GUI_FILE_REFRESH_MS);
    const documentRefresh = createThrottledRefresh(() => {
      invalidateRuntimeDocuments(queryClient).catch((error: unknown) => {
        if (!active || !session.identity.isCurrent()) return;
        toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
        setHubState({ error: failureOf(error, "Runtime Stream Failed").message });
      });
    }, DOCUMENT_REFRESH_MS);
    const refreshCoordinator = createRuntimeProjectionRefreshCoordinator(async () => {
      await Promise.all([
        invalidateRuntimeProjectionQueries(queryClient),
        queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }, { cancelRefetch: false }),
        queryClient.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all }, { cancelRefetch: false }),
      ]);
      documentRefresh.request();
    });

    const connect = async () => {
      if (!active || !session.identity.isCurrent()) return;
      try {
        stream = await openNativeRuntimeStream({ ...session.identity, own: session.own }, undefined);
        if (!active || !session.identity.isCurrent()) {
          await stream.cancel();
          return;
        }
        setHubState({ connection: "connected", error: null });
        let observed: string | null = null;
        for await (const frame of stream) {
          if (!active) break;
          fileRefresh.observe(frame);
          observed = handleRuntimeFrame(
            frame,
            observed,
            queryClient,
            refreshCoordinator,
            setHubState,
            session.identity.generation,
            () => active && session.identity.isCurrent(),
          );
        }
      } catch (error: unknown) {
        if (active && session.identity.isCurrent()) {
          toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
          setHubState({
            error: failureOf(error, "Operation Failed").message,
          });
        }
      } finally {
        stream = undefined;
        if (active && session.identity.isCurrent()) {
          setHubState({ connection: "reconnecting", operations: [] });
          retry = window.setTimeout(() => void connect(), GUI_RECONNECT_MS);
        }
      }
    };

    void connect();
    return () => {
      active = false;
      if (retry !== undefined) window.clearTimeout(retry);
      refreshCoordinator.dispose();
      documentRefresh.dispose();
      fileRefresh.dispose();
      fileCoordinator.dispose();
      void stream?.cancel();
    };
  }, [queryClient, session, setHubState]);
}

/* === Private Functions === */

function handleRuntimeFrame(
  frame: RuntimeStateFrame,
  observed: string | null,
  queryClient: ReturnType<typeof useQueryClient>,
  refreshCoordinator: RuntimeProjectionRefreshCoordinator,
  setHubState: (patch: BackendOperationsPatch) => void,
  sessionGeneration: string,
  isActive: () => boolean,
): string | null {
  setHubState({
    error: null,
    lastStreamFrameAt: Date.now(),
    operations: operationsFromFrame(frame.operations),
  });
  if (frame.shuttingDown) {
    setHubState({ connection: "reconnecting" });
  }
  /* Execution files refresh separately on operation changes; the other
     projections follow changes to the state cursor. */
  const cursor = frame.cursor ? `${frame.cursor.epoch}/${frame.cursor.stream}/${frame.cursor.position}` : null;
  if (frame.resynchronized || (cursor !== null && cursor !== observed)) {
    const ensureTrailing =
      queryClient.isFetching({
        predicate: (query) => query.queryKey[0] === workflowQueryKeys.all[0] && query.queryKey.at(-1) === sessionGeneration,
      }) > 0;
    void refreshCoordinator.request({ ensureTrailing }).catch((error: unknown) => {
      if (!isActive()) return;
      toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
      setHubState({ error: failureOf(error, "Runtime Stream Failed").message });
    });
  }
  return cursor ?? observed;
}
