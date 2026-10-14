/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import type { NativeExecutionSelection } from "@/lib/api/native-checkpoints";
import { fetchExecutionPage, groupExecutions } from "./explore-data-source";

/* === Types === */

/** Which rows a history read admits by their archive mark. */
export type HistoryArchiveView = "Unarchived" | "Archived" | "All";

/* === Query Keys === */

export const exploreQueryKeys = {
  all: ["explore"] as const,
  history: (text = "", archive: HistoryArchiveView = "Unarchived") =>
    ["explore", "executions", { archive, text }] as const,
};

/* === Query Hooks === */

/**
 * The merged execution browse, selected and ordered server-side.
 *
 * `archive` selects the rows by their archive mark: the default list leaves out
 * executions archived on their own or with their cycle, the Archived view
 * lists only those, and a direct link reads both. `text` is matched by the server against each entry's title, its execution
 * identifier and its suite label. The selection is memoized on that one string
 * and forwarded unchanged on every continuation, because the contract rejects
 * a changed selection rather than restarting the read; a different `text` is a
 * different query key and therefore a new read from the first page.
 */
export function useExploreHistoryQuery(text = "", archive: HistoryArchiveView = "Unarchived") {
  const session = useNativeSession();
  const needle = text.trim();
  const selection = useMemo<NativeExecutionSelection>(
    () => ({
      filter: {
        text: needle,
        statuses: [],
        templateIds: [],
        cycleIds: [],
        archive: { HarmosVariant: { case: archive, value: {} } },
      },
      order: { key: { HarmosVariant: { case: "StartedAt", value: {} } }, descending: true },
    }),
    [archive, needle],
  );
  const query = useInfiniteQuery({
    queryKey: exploreQueryKeys.history(needle, archive),
    initialPageParam: null as string | null,
    queryFn: ({ pageParam, signal }) =>
      fetchExecutionPage(session.identity, pageParam, signal, selection),
    getNextPageParam: (page) => page.next ?? undefined,
    staleTime: 30_000,
  });
  return {
    ...query,
    data: query.data ? groupExecutions(query.data.pages) : undefined,
    coverage: query.data?.pages.at(-1),
  };
}
