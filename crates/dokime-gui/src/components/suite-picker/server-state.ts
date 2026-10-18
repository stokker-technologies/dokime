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
import { fetchNativeSuites } from "@/lib/api/native-catalog";
import { appendNativeTestCycle, listNativeCycles, previewNativeTestCycle } from "@/lib/api/native-planning";
import type { PreviewTestCycleRequest, TestCycleSelection } from "@/lib/types";

/* === Query Keys === */

export const suiteQueryKeys = {
  all: ["suite"] as const,
  appendCycles: () => [...suiteQueryKeys.all, "append-cycles"] as const,
  preview: (request: PreviewTestCycleRequest) => [...suiteQueryKeys.all, "preview", request] as const,
  suites: () => [...suiteQueryKeys.all, "suites"] as const,
};

/* === Hooks === */

export function useTestSuitesQuery() {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => fetchNativeSuites(session.identity),
    queryKey: [...suiteQueryKeys.suites(), session.identity.generation],
    staleTime: 60_000,
  });
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
    onSuccess: async () => {
      await Promise.all([invalidateWorkflowResourceQueries(queryClient), queryClient.invalidateQueries({ queryKey: suiteQueryKeys.all })]);
    },
  });
}

/** Active destinations; choosing one remains an explicit operator action. */
export function useAppendCyclesQuery() {
  const session = useNativeSession();
  return useQuery({
    queryFn: async () => {
      const cycles = await listNativeCycles(session.identity);
      return cycles.filter((cycle) => cycle.phase?.HarmosVariant.case === "Active");
    },
    queryKey: [...suiteQueryKeys.appendCycles(), session.identity.generation],
    staleTime: 1_000,
  });
}
