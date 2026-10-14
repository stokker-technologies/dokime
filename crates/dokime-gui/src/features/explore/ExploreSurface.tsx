/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useEffect, useMemo, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { useExploreHistoryQuery } from "@/features/explore/explore-queries";
import { HistoryComparisonView } from "@/features/explore/HistoryComparisonView";
import { HistoryLibrary } from "@/features/explore/HistoryLibrary";
import type { ExecutionBrowsePage } from "@/lib/types/view-contracts";
import { usePageChrome } from "@/shell/page-chrome";
import { type ExploreTarget, exploreEntryKey } from "./explore-contracts";

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
export function ExploreSurface({
  comparing,
  onBack,
  onCompare,
  onOpenEntry,
  onToggleSelected,
  selectedIds,
}: ExploreSurfaceProps) {
  const [query, setQuery] = useState("");
  /* The box types faster than a round trip; the read follows the operator
     rather than every keystroke, and the box itself never lags. */
  const [selection, setSelection] = useState("");
  const cyclesQuery = useExploreHistoryQuery(selection);
  const cycles = cyclesQuery.data ?? [];

  useEffect(() => {
    const handle = window.setTimeout(() => setSelection(query), 250);
    return () => window.clearTimeout(handle);
  }, [query]);

  usePageChrome({ title: comparing ? "Compare" : "History" });

  const selectedCycles = useMemo(
    () => cycles.filter((cycle) => selectedIds.has(exploreEntryKey(cycle.target))),
    [cycles, selectedIds],
  );

  return (
    <div className="grid content-start">
      {cyclesQuery.isError ? (
        <p className="px-[var(--shell-inset)] py-[var(--shell-gap)] text-destructive" role="alert">
          Unable to load execution history: {cyclesQuery.error.message}
        </p>
      ) : null}
      {cyclesQuery.coverage && !cyclesQuery.coverage.discovery_complete ? (
        <IncompleteHistoryNotice coverage={cyclesQuery.coverage} />
      ) : null}
      {comparing ? (
        <HistoryComparisonView cycles={selectedCycles} onBack={onBack} />
      ) : (
        <HistoryLibrary
          cycles={cycles}
          loading={cyclesQuery.isPending}
          onCompare={onCompare}
          onOpenEntry={onOpenEntry}
          onQueryChange={setQuery}
          onToggleSelected={onToggleSelected}
          query={query}
          selectedIds={selectedIds}
        />
      )}
      {cyclesQuery.hasNextPage ? (
        <div className="px-[var(--shell-inset)] py-[var(--shell-gap)]">
          <Button
            className="h-[var(--control-height)]"
            disabled={cyclesQuery.isFetchingNextPage}
            onClick={() => void cyclesQuery.fetchNextPage()}
            variant="outline"
          >
            {cyclesQuery.isFetchingNextPage ? "Loading…" : "Load More Executions"}
          </Button>
        </div>
      ) : null}
    </div>
  );
}

/* === Private Components === */

/**
 * One quiet line above the list, and the reasons only when there are any: a
 * source that could not be read, or executions claiming the same identity.
 */
function IncompleteHistoryNotice({ coverage }: { coverage: ExecutionBrowsePage }) {
  const reasons = [
    ...new Set(coverage.issues.map((issue) => `${issue.source ?? "Storage"}: ${issue.message}`)),
  ];
  const conflicts = coverage.conflicts.length + coverage.omitted_conflicts;
  if (conflicts > 0) {
    reasons.push(
      conflicts === 1
        ? "1 execution identity is claimed by more than one source."
        : `${conflicts} execution identities are claimed by more than one source.`,
    );
  }

  return (
    <div
      className="grid gap-1 border-b px-[var(--shell-inset)] py-[var(--shell-gap)] text-sm text-muted-foreground"
      role="status"
    >
      <p>History is incomplete: some sources or executions could not be read.</p>
      {reasons.length ? (
        <ul className="list-disc pl-5">
          {reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
