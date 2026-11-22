/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared live-cycle server state. The Cycles home and the status bar read the
   same two queries so the shell never fetches the cycle list twice. */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient, type UseQueryResult } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { createNativeTestCycle, type NewCycleRequest } from "@/lib/api/native-planning";
import type { PromptRequest } from "@/lib/types";
import type { CycleSummary } from "@/lib/api/native-planning-projection";
import { cyclesHomeQueryKeys, cyclesQueryOptions, pendingPromptsQueryOptions, type CyclesQuerySelection } from "./cycles-query-options";

/* === Query Hooks === */

export function useCyclesQuery(selection: CyclesQuerySelection = {}): UseQueryResult<CycleSummary[]> {
  const session = useNativeSession();
  return useQuery(cyclesQueryOptions(session.identity, selection));
}

export function usePendingPromptsQuery(): UseQueryResult<PromptRequest[]> {
  const session = useNativeSession();
  return useQuery(pendingPromptsQueryOptions(session.identity));
}

/* === Mutation Hooks === */

/** Creates an empty Ready cycle and refreshes every cycle list that shows it. */
export function useCreateCycleMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: NewCycleRequest) => createNativeTestCycle(session.identity, request),
    onSuccess: async () => {
      await Promise.all([
        invalidateWorkflowResourceQueries(queryClient),
        queryClient.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all }),
      ]);
    },
  });
}
