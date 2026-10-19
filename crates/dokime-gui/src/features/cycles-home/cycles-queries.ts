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

import { type UseQueryResult, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { invalidateWorkflowResourceQueries, workflowQueryKeys } from "@/features/cycle/server-state";
import { createNativeTestCycle, listNativeCycles, type NativeCycleSelection, type NewCycleRequest } from "@/lib/api/native-planning";
import { fetchNativePendingPrompts } from "@/lib/api/native-prompts";
import type { Wire } from "@/lib/rpc/canonical";
import type { PromptRequest } from "@/lib/types";
import type { Row2 as GeneratedCycleSummary } from "../../../.generated/dokime/v1/ExecutionMessages/GetCycle/messages_pb";

type CycleSummary = Wire<GeneratedCycleSummary>;

/* === Types === */

export type CyclesQuerySelection = {
  /** Matched server-side against the cycle's label and identifier. */
  text?: string;
  /* True lists only archived cycles, false only the others. False is the
     default so the status bar and an unfiltered Cycles home ask the identical
     question and share one cached read rather than paginating the whole list
     twice every five seconds. */
  archived?: boolean;
};

/* === Query Keys === */

export const cyclesHomeQueryKeys = {
  all: ["cycles-home"] as const,
  cycles: (generation: string, selection: CyclesQuerySelection = {}) =>
    [...cyclesHomeQueryKeys.all, "cycles", generation, { archived: selection.archived ?? false, text: selection.text ?? "" }] as const,
};

/* === Constants === */

/* === Public Functions === */

/**
 * The selection and order every page of one read repeats.
 *
 * The server's default order is UPDATED_AT ascending, so newest-first is a
 * request the client has to make.
 */
export function cycleListSelection(selection: CyclesQuerySelection): NativeCycleSelection {
  return {
    filter: {
      text: selection.text?.trim() ?? "",
      statuses: [],
      archive: {
        HarmosVariant: selection.archived ? { case: "Archived", value: {} } : { case: "Unarchived", value: {} },
      },
    },
    order: { key: { HarmosVariant: { case: "UpdatedAt", value: {} } }, descending: true },
  };
}

/* === Query Hooks === */

/**
 * Every page of Execution.ListCycles under one server-side selection.
 *
 * The selection belongs to the caller: the Cycles home filters and the status
 * bar counts, and a bar that inherited the home's filter box would report an
 * activity count of whatever the operator happened to be typing.
 */
export function useCyclesQuery(selection: CyclesQuerySelection = {}): UseQueryResult<CycleSummary[]> {
  const session = useNativeSession();
  const archived = selection.archived ?? false;
  const text = selection.text ?? "";
  return useQuery({
    queryFn: () => listNativeCycles(session.identity, cycleListSelection({ archived, text })),
    queryKey: cyclesHomeQueryKeys.cycles(session.identity.generation, { archived, text }),
    refetchInterval: 5_000,
    staleTime: 2_000,
  });
}

/** Every pending prompt across cycles, hydrated through the existing adapter. */
export function usePendingPromptsQuery(): UseQueryResult<PromptRequest[]> {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => fetchNativePendingPrompts(session.identity),
    // The workflow prompt key, so every runtime change that invalidates the
    // cycle's prompts refreshes the navigation and status counts at once.
    queryKey: [...workflowQueryKeys.prompts(), session.identity.generation],
    refetchInterval: 5_000,
    staleTime: 2_000,
  });
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
