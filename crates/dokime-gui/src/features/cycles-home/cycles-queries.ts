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

import { useQuery, type UseQueryResult } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import type { PromptRequest } from "@/lib/types";
import type { CycleSummary } from "@/lib/api/native-planning-projection";
import { cyclesQueryOptions, pendingPromptsQueryOptions, type CyclesQuerySelection } from "./cycles-query-options";

/* === Query Hooks === */

export function useCyclesQuery(selection: CyclesQuerySelection = {}): UseQueryResult<CycleSummary[]> {
  const session = useNativeSession();
  return useQuery(cyclesQueryOptions(session.identity, selection));
}

export function usePendingPromptsQuery(): UseQueryResult<PromptRequest[]> {
  const session = useNativeSession();
  return useQuery(pendingPromptsQueryOptions(session.identity));
}
