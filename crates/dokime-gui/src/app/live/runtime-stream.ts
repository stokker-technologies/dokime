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
import type { QueryClient } from "@tanstack/svelte-query";

/* === Local Imports === */

import type { ConnectedSession } from "@/app/native-session/session";
import { invalidateRuntimeDocuments, invalidateRuntimeProjectionQueries } from "@/features/cycle/server-state";
import { workflowQueryKeys } from "@/features/cycle/workflow-query-keys";
import { cyclesHomeQueryKeys } from "@/features/cycles-home/cycles-query-options";
import type { RuntimeStateFrame } from "@/lib/api/native-observation";
import { openNativeRuntimeStream } from "@/lib/api/native-observation";
import { isClientCancellation, isHostReopen } from "@/lib/rpc/errors";
import { type BackendOperationsPatch, hubStore } from "@/stores/hub-store";
import {
  createEvidenceRefresh,
  createRuntimeProjectionRefreshCoordinator,
  runtimeStateOf,
  settledOf,
  type RuntimeProjectionRefreshCoordinator,
} from "./runtime-refresh";

/* === Constants === */

const GUI_FILE_REFRESH_MS = 250;
/** The Most Often a Running Cycle Re-Reads Its Open Documents. */
const GUI_DOCUMENT_REFRESH_MS = 5_000;
const GUI_RECONNECT_MS = 1_000;
const EMPTY_OPERATIONS: NonNullable<BackendOperationsPatch["operations"]> = [];

/* === Runtime stream ===
   The single live subscription. Native frames carry an observed cursor and
   lifecycle flags; the generation-scoped queries re-read each affected
   projection through the same native session. */

/* === Public Functions === */

export function observeRuntime(session: ConnectedSession): () => void {
  const { queryClient } = session;
  const { setHubState } = hubStore.getState();
  let active = true;
  let stream: Awaited<ReturnType<typeof openNativeRuntimeStream>> | undefined;
  let retry: number | undefined;
  const fileCoordinator = createRuntimeProjectionRefreshCoordinator(async () => {
    if (!active || !session.identity.isCurrent()) {
      return;
    }
    await queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "execution-files"] }, { cancelRefetch: false });
  });
  const fileRefresh = createEvidenceRefresh(() => {
    const ensureTrailing = queryClient.isFetching({ queryKey: [...workflowQueryKeys.all, "execution-files"] }) > 0;
    void fileCoordinator.request({ ensureTrailing }).catch((error: unknown) => {
      if (!active || !session.identity.isCurrent()) {
        return;
      }
      toast.error(failureOf(error, "Execution Files Failed"), { context: "background" });
    });
  }, GUI_FILE_REFRESH_MS);
  const documentRefresh = createEvidenceRefresh(() => {
    invalidateRuntimeDocuments(queryClient).catch((error: unknown) => {
      if (!active || !session.identity.isCurrent()) {
        return;
      }
      toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
      setHubState({ error: failureOf(error, "Runtime Stream Failed").message });
    });
  }, GUI_DOCUMENT_REFRESH_MS);
  /* A cycle that settles or seals re-reads its documents at once: live growth
     is throttled, but the settled evidence must not wait behind it. */
  let observedSettled: string | null = null;
  let settled = false;
  const refreshCoordinator = createRuntimeProjectionRefreshCoordinator(async () => {
    await Promise.all([
      invalidateRuntimeProjectionQueries(queryClient),
      queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }, { cancelRefetch: false }),
      queryClient.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all }, { cancelRefetch: false }),
    ]);
    if (settled) {
      settled = false;
      documentRefresh.now();
    }
  });

  const connect = async () => {
    if (!active || !session.identity.isCurrent()) {
      return;
    }
    // A Host Deadline Reopens The Watch At Once, Silently.
    let reopen = false;
    try {
      stream = await openNativeRuntimeStream({ ...session.identity, own: session.own }, undefined);
      if (!active || !session.identity.isCurrent()) {
        await stream.cancel();
        return;
      }
      setHubState({ connection: "connected", error: null });
      let observed: string | null = null;
      for await (const frame of stream) {
        if (!active) {
          break;
        }
        fileRefresh.observe(frame);
        documentRefresh.observe(frame);
        const current = settledOf(frame);
        if (current !== null && current !== observedSettled) {
          settled ||= observedSettled !== null;
          observedSettled = current;
        }
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
      reopen = isHostReopen(error);
      if (active && session.identity.isCurrent() && !isClientCancellation(error) && !reopen) {
        toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
        setHubState({
          error: failureOf(error, "Operation Failed").message,
        });
      }
    } finally {
      stream = undefined;
      if (active && session.identity.isCurrent()) {
        if (reopen) {
          void connect();
        } else {
          setHubState({ connection: "reconnecting", operations: EMPTY_OPERATIONS });
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
    refreshCoordinator.dispose();
    documentRefresh.dispose();
    fileRefresh.dispose();
    fileCoordinator.dispose();
    void stream?.cancel();
  };
}

/* === Private Functions === */

function handleRuntimeFrame(
  frame: RuntimeStateFrame,
  observed: string | null,
  queryClient: QueryClient,
  refreshCoordinator: RuntimeProjectionRefreshCoordinator,
  setHubState: (patch: BackendOperationsPatch) => void,
  sessionGeneration: string,
  isActive: () => boolean,
): string | null {
  setHubState({ error: null, operations: EMPTY_OPERATIONS });
  if (frame.overview?.shuttingDown) {
    setHubState({ connection: "reconnecting" });
  }
  /* Execution files refresh separately on operation changes; the other
     projections follow changes to the state cursor. */
  const cursor = runtimeStateOf(frame);
  if (frame.opening || (cursor !== null && cursor !== observed)) {
    const ensureTrailing =
      queryClient.isFetching({
        predicate: (query) => query.queryKey[0] === workflowQueryKeys.all[0] && query.queryKey.at(-1) === sessionGeneration,
      }) > 0;
    void refreshCoordinator.request({ ensureTrailing }).catch((error: unknown) => {
      if (!isActive()) {
        return;
      }
      toast.error(failureOf(error, "Runtime Stream Failed"), { context: "background" });
      setHubState({ error: failureOf(error, "Runtime Stream Failed").message });
    });
  }
  return cursor ?? observed;
}
