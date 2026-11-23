/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Suite selection server state. */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { appendNativeTestCycle, previewNativeTestCycle } from "@/lib/api/native-planning";
import type { PreviewTestCycleRequest, TestCycleSelection } from "@/lib/types";
import { fetchNativeSuites } from "../../lib/host/suite-session";
import type { SessionIdentity } from "../../lib/rpc/transport";

/* === Query Keys === */

export const suiteQueryKeys = {
  all: ["suite"] as const,
  suites: () => [...suiteQueryKeys.all, "suites"] as const,
};

/* === Hooks === */

export function useTestSuitesQuery() {
  const session = useNativeSession();
  return useQuery(testSuitesQueryOptions(session.identity));
}

export function testSuitesQueryOptions(identity: SessionIdentity) {
  return {
    queryFn: () => fetchNativeSuites(identity),
    queryKey: [...suiteQueryKeys.suites(), identity.generation],
  };
}

export function usePreviewTestCycleMutation() {
  const session = useNativeSession();
  return useMutation({
    mutationFn: (request: PreviewTestCycleRequest) => previewNativeTestCycle(session.identity, request),
  });
}

/** `cycleId` names the cycle the cases join. Every caller states it. */
export function useAppendTestCycleMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ cycleId, selections }: { cycleId: string; selections: TestCycleSelection[] }) =>
      appendNativeTestCycle(session.identity, cycleId, selections),
    onSuccess: () => invalidateWorkflowResourceQueries(queryClient),
  });
}
