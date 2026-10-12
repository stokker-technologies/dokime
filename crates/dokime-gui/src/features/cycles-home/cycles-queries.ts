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

import { type UseQueryResult, useQuery } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import { listNativeCycles, type NativeCycleSelection } from "@/lib/api/native-planning";
import { fetchNativePendingPrompts } from "@/lib/api/native-prompts";
import {
  CycleOrderKey,
  CycleStatus,
  type CycleSummary,
} from "@/lib/rpc/generated/dokime/v1/execution_pb";
import type { PromptRequest } from "@/lib/types";

/* === Types === */

export type CyclesQuerySelection = {
  /** Matched server-side against label, identifier and admitted suite label. */
  text?: string;
  /* False names the other lifecycles; an empty `statuses` is no constraint.
     False is the default so the status bar and an unfiltered Cycles home ask
     the identical question and share one cached read rather than paginating
     the whole list twice every five seconds. */
  includeCleared?: boolean;
};

/* === Query Keys === */

export const cyclesHomeQueryKeys = {
  all: ["cycles-home"] as const,
  cycles: (generation: string, selection: CyclesQuerySelection = {}) =>
    [
      ...cyclesHomeQueryKeys.all,
      "cycles",
      generation,
      { includeCleared: selection.includeCleared ?? false, text: selection.text ?? "" },
    ] as const,
  pendingPrompts: (generation: string) =>
    [...cyclesHomeQueryKeys.all, "pending-prompts", generation] as const,
};

/* === Constants === */

/**
 * The lifecycles a list without Show Cleared admits.
 *
 * An empty `statuses` is no constraint, so excluding one lifecycle means
 * naming the other ten. UNSPECIFIED is not among them: it is not a lifecycle
 * a cycle can be in.
 */
const UNCLEARED_STATUSES = [
  CycleStatus.PREVIEWED,
  CycleStatus.READY,
  CycleStatus.RUNNING,
  CycleStatus.PAUSED,
  CycleStatus.STOPPING,
  CycleStatus.COMPLETED,
  CycleStatus.FAILED,
  CycleStatus.BLOCKED,
  CycleStatus.STOPPED,
];

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
      statuses: selection.includeCleared === false ? UNCLEARED_STATUSES : [],
      suiteIds: [],
    },
    order: { key: CycleOrderKey.UPDATED_AT, descending: true },
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
export function useCyclesQuery(
  selection: CyclesQuerySelection = {},
): UseQueryResult<CycleSummary[]> {
  const session = useNativeSession();
  const includeCleared = selection.includeCleared ?? false;
  const text = selection.text ?? "";
  return useQuery({
    queryFn: () => listNativeCycles(session.identity, cycleListSelection({ includeCleared, text })),
    queryKey: cyclesHomeQueryKeys.cycles(session.identity.generation, { includeCleared, text }),
    refetchInterval: 5_000,
    staleTime: 2_000,
  });
}

/** Every pending prompt across cycles, hydrated through the existing adapter. */
export function usePendingPromptsQuery(): UseQueryResult<PromptRequest[]> {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => fetchNativePendingPrompts(session.identity),
    queryKey: cyclesHomeQueryKeys.pendingPrompts(session.identity.generation),
    refetchInterval: 5_000,
    staleTime: 2_000,
  });
}
