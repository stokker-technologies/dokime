// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ConnectedSession } from "../native-session/session";
import { type LiveExecutionProgress, observeExecutionProgress } from "./execution-progress";

export function createExecutionProgress(session: () => ConnectedSession, executionId: () => string | null) {
  let progress = $state.raw<LiveExecutionProgress | null>(null);
  const connection = $derived(session());
  $effect(() => {
    const id = executionId();
    progress = null;
    if (id) {
      return observeExecutionProgress(connection, id, (next) => {
        progress = next;
      });
    }
  });
  return () => {
    const id = executionId();
    return id === progress?.testExecutionId ? progress : null;
  };
}
