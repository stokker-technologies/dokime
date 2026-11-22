// Dokime — SPDX-License-Identifier: Apache-2.0
import { ArchiveScope, ExecutionOrderKey } from "@generated/host/model";
import { useQuery } from "@tanstack/react-query";
import { useNativeSession } from "@/app/native-session/context";
import { fetchNativeCyclePage, type NativeCycleContinuation, type NativeExecutionSelection } from "@/lib/api/native-checkpoints";
import { NATIVE_MAX_PAGES, NativeApiLimitError } from "@/lib/api/native-values";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { ExecutionBrowsePage, SavedCycleSummary } from "@/lib/types/browse";
import { fetchExecutionPage, groupExecutions } from "./explore-data-source";

type HistoryArchiveView = "Unarchived" | "Archived" | "All";

export const exploreQueryKeys = {
  all: ["explore"] as const,
};

function archiveScope(archive: HistoryArchiveView) {
  switch (archive) {
    case "All":
      return ArchiveScope.ALL;
    case "Archived":
      return ArchiveScope.ARCHIVED;
    case "Unarchived":
      return ArchiveScope.UNARCHIVED;
  }
}

export function historyExecutionOptions(identity: SessionIdentity, archive: HistoryArchiveView) {
  const selection: NativeExecutionSelection = {
    filter: { text: "", statuses: [], templateIds: [], cycleIds: [], archive: archiveScope(archive) },
    order: { key: ExecutionOrderKey.STARTED_AT, descending: true },
  };
  return {
    queryKey: [...exploreQueryKeys.all, "executions", identity.generation, archive],
    queryFn: async ({ signal }: { signal: AbortSignal }) => {
      const pages: ExecutionBrowsePage[] = [];
      let cursor: string | null = null;
      for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
        const page = await fetchExecutionPage(identity, cursor, signal, selection);
        pages.push(page);
        if (!page.next) {
          return pages;
        }
        cursor = page.next;
      }
      throw new NativeApiLimitError("Load History", "Too many execution pages. Archive older entries and reload.");
    },
  };
}

export function historyCycleOptions(identity: SessionIdentity, archive: HistoryArchiveView) {
  const filter = { text: "", statuses: [], archive: archiveScope(archive) };
  return {
    queryKey: [...exploreQueryKeys.all, "cycles", identity.generation, archive],
    queryFn: async ({ signal }: { signal: AbortSignal }) => {
      const entries: SavedCycleSummary[] = [];
      let cursor: NativeCycleContinuation | null = null;
      for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
        const page = await fetchNativeCyclePage(identity, cursor, signal, filter);
        entries.push(...page.entries);
        if (!page.next) {
          return entries;
        }
        cursor = page.next;
      }
      throw new NativeApiLimitError("Load History", "Too many cycle pages. Archive older entries and reload.");
    },
  };
}

export function useExploreHistoryQuery(archive: HistoryArchiveView) {
  const { identity } = useNativeSession();
  const executions = useQuery(historyExecutionOptions(identity, archive));
  const cycles = useQuery(historyCycleOptions(identity, archive));
  return {
    data: executions.data || cycles.data ? groupExecutions(executions.data ?? [], cycles.data ?? []) : undefined,
    isPending: executions.isPending || cycles.isPending,
    isError: executions.isError || cycles.isError,
    error: executions.error ?? cycles.error,
    coverage: executions.data?.at(-1),
  };
}
