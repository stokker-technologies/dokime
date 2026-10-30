/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import { exploreQueryKeys, useExploreHistoryQuery } from "@/features/explore/explore-queries";
import { HistoryComparisonView } from "@/features/explore/HistoryComparisonView";
import { HistoryLibrary } from "@/features/explore/HistoryLibrary";
import { setNativeCycleArchived, setNativeExecutionArchived } from "@/lib/api/native-execution";
import type { ExecutionBrowsePage } from "@/lib/types/view-contracts";
import { PageNotice, PageShell } from "@/shell/PageShell";
import { type ExploreEntry, type ExploreTarget, exploreEntryKey } from "./explore-contracts";

const GUI_FILTER_DEBOUNCE_MS = 250;

/* === Types === */

type ExploreSurfaceProps = {
  comparing: boolean;
  onBack: () => void;
  onCompare: () => void;
  onOpenEntry: (target: ExploreTarget) => void;
  onToggleSelected: (entryKey: string) => void;
  selectedIds: Set<string>;
};

/* === Components === */

/**
 * Browse execution history, grouped by cycle where associated, and compare
 * selected entries. The shell owns the frame: this surface renders content
 * only and publishes its title through the page chrome. Selection is owned by
 * the route's URL search state, so it survives navigation.
 */
export function ExploreSurface({ comparing, onBack, onCompare, onOpenEntry, onToggleSelected, selectedIds }: ExploreSurfaceProps) {
  const [query, setQuery] = useState("");
  /* The box types faster than a round trip; the read follows the operator
     rather than every keystroke, and the box itself never lags. */
  const [selection, setSelection] = useState("");
  const [archived, setArchived] = useState(false);
  const cyclesQuery = useExploreHistoryQuery(selection, archived ? "Archived" : "Unarchived");
  const session = useNativeSession();
  const queryClient = useQueryClient();
  /* A row reads archived when its executions or its whole cycle are, so
     Unarchive clears both marks for everything the row lists. */
  const unarchive = useMutation({
    mutationFn: async (entry: ExploreEntry) => {
      if (entry.target.kind === "cycle") {
        await setNativeCycleArchived(session.identity, entry.target.id, false);
      }
      for (const execution of [...entry.executions, ...entry.unrun]) {
        if (execution.archived) {
          await setNativeExecutionArchived(session.identity, execution.metadata.execution_id, false);
        }
      }
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: exploreQueryKeys.all }),
  });
  const onUnarchive = useCallback(
    (entry: ExploreEntry) =>
      void unarchive
        .mutateAsync(entry)
        .then(() => toast.success("Unarchived"))
        .catch((error: unknown) =>
          toast.error("The Archive Mark Could Not Be Removed", {
            description: error instanceof Error ? error.message : String(error),
          }),
        ),
    [unarchive.mutateAsync],
  );
  const cycles = cyclesQuery.data ?? [];

  useEffect(() => {
    const handle = window.setTimeout(() => setSelection(query), GUI_FILTER_DEBOUNCE_MS);
    return () => window.clearTimeout(handle);
  }, [query]);

  const selectedCycles = useMemo(() => cycles.filter((cycle) => selectedIds.has(exploreEntryKey(cycle.target))), [cycles, selectedIds]);

  return (
    <PageShell trail={comparing ? ["History", "Compare"] : ["History"]}>
      {cyclesQuery.isError ? <PageNotice tone="alert">Unable to load execution history: {cyclesQuery.error?.message}</PageNotice> : null}
      {cyclesQuery.coverage && !cyclesQuery.coverage.discovery_complete ? (
        <IncompleteHistoryNotice coverage={cyclesQuery.coverage} />
      ) : null}
      {comparing ? (
        <HistoryComparisonView cycles={selectedCycles} onBack={onBack} />
      ) : (
        <div aria-busy={cyclesQuery.isPending} className="min-w-0">
          <HistoryLibrary
            archived={archived}
            cycles={cycles}
            loading={cyclesQuery.isPending}
            onCompare={onCompare}
            onToggleArchived={() => setArchived((current) => !current)}
            onUnarchive={onUnarchive}
            onOpenEntry={onOpenEntry}
            onQueryChange={setQuery}
            onToggleSelected={onToggleSelected}
            query={query}
            selectedIds={selectedIds}
          />
        </div>
      )}
      {cyclesQuery.hasNextPage ? (
        <div>
          <Button
            className="h-[var(--control-height)]"
            disabled={cyclesQuery.isFetchingNextPage}
            onClick={() => void cyclesQuery.fetchNextPage()}
            variant="outline"
          >
            {cyclesQuery.isFetchingNextPage ? "Loading…" : "Load More History"}
          </Button>
        </div>
      ) : null}
    </PageShell>
  );
}

/* === Private Components === */

/**
 * One quiet line above the list, and the reasons only when there are any: a
 * source that could not be read, or executions claiming the same identity.
 */
function IncompleteHistoryNotice({ coverage }: { coverage: ExecutionBrowsePage }) {
  const reasons = [...new Set(coverage.issues.map((issue) => `${issue.source ?? "Storage"}: ${issue.message}`))];
  const conflicts = coverage.conflicts.length + coverage.omitted_conflicts;
  if (conflicts > 0) {
    reasons.push(
      conflicts === 1
        ? "1 execution identity is claimed by more than one source."
        : `${conflicts} execution identities are claimed by more than one source.`,
    );
  }

  return (
    <PageNotice>
      <p>History is incomplete: some sources or executions could not be read.</p>
      {reasons.length ? (
        <ul className="list-disc pl-4">
          {reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
    </PageNotice>
  );
}
