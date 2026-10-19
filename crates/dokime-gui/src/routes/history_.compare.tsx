/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";

/* === Local Imports === */

import { ExploreSurface } from "@/features/explore/ExploreSurface";
import { exploreSearchSchema, searchFromSelectedIds, selectedIdsFromSearch, toggleSelectedId } from "@/features/explore/explore-search";

/* === Route === */

export const Route = createFileRoute("/history_/compare")({
  component: HistoryCompareRoute,
  validateSearch: (search) => exploreSearchSchema.parse(search),
});

/* === Components === */

function HistoryCompareRoute() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const selectedIds = selectedIdsFromSearch(search);

  const back = useCallback(() => {
    void navigate({ search: searchFromSelectedIds(selectedIdsFromSearch(search)), to: "/history" });
  }, [navigate, search]);

  const toggleSelected = useCallback(
    (cycleId: string) => {
      void navigate({
        replace: true,
        search: searchFromSelectedIds(toggleSelectedId(selectedIdsFromSearch(search), cycleId)),
        to: "/history/compare",
      });
    },
    [navigate, search],
  );

  return (
    <ExploreSurface
      comparing
      onBack={back}
      onCompare={() => undefined}
      onOpenEntry={() => undefined}
      onToggleSelected={toggleSelected}
      selectedIds={selectedIds}
    />
  );
}
