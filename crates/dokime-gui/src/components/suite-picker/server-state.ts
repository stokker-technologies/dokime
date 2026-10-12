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

import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import { invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { fetchNativeSuites } from "@/lib/api/native-catalog";
import {
  appendNativeTestCycle,
  createNativeTestCycle,
  listNativeCycles,
  previewNativeTestCycle,
} from "@/lib/api/native-planning";
import type { PreviewTestCycleRequest, StartTestCycleRequest } from "@/lib/types";

/* === Query Keys === */

export const suiteQueryKeys = {
  all: ["suite"] as const,
  cycleLabel: (cycleId: string) => [...suiteQueryKeys.all, "cycle-label", cycleId] as const,
  preview: (request: PreviewTestCycleRequest) =>
    [...suiteQueryKeys.all, "preview", request] as const,
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
    mutationFn: (request: PreviewTestCycleRequest) =>
      previewNativeTestCycle(session.identity, request),
  });
}

/** Creates the cycle and leaves it Ready; the operator admits the run. */
export function useCreateTestCycleMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: StartTestCycleRequest) =>
      createNativeTestCycle(session.identity, request),
    onSuccess: async () => {
      await Promise.all([
        invalidateWorkflowResourceQueries(queryClient),
        queryClient.invalidateQueries({ queryKey: suiteQueryKeys.all }),
      ]);
    },
  });
}

/** `cycleId` names the cycle the cases join. Every caller states it. */
export function useAppendTestCycleMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ cycleId, request }: { cycleId: string; request: StartTestCycleRequest }) =>
      appendNativeTestCycle(session.identity, cycleId, request),
    onSuccess: async () => {
      await Promise.all([
        invalidateWorkflowResourceQueries(queryClient),
        queryClient.invalidateQueries({ queryKey: suiteQueryKeys.all }),
      ]);
    },
  });
}

/**
 * The label of the cycle the Library is appending to.
 *
 * One bounded ListCycles read; the cycle's full plan would cost a ListCases
 * and a GetCase per case for a single string.
 */
export function useCycleLabelQuery(cycleId: string | undefined) {
  const session = useNativeSession();
  return useQuery({
    enabled: Boolean(cycleId),
    queryFn: async () => {
      const cycles = await listNativeCycles(session.identity);
      const summary = cycles.find((candidate) => candidate.cycleId === cycleId);
      return summary?.metadata?.label || (cycleId ?? "");
    },
    queryKey: [...suiteQueryKeys.cycleLabel(cycleId ?? ""), session.identity.generation],
    staleTime: 30_000,
  });
}
