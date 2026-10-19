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
import type { ExploreTarget } from "@/features/explore/explore-contracts";
import { exploreSearchSchema, searchFromSelectedIds, selectedIdsFromSearch, toggleSelectedId } from "@/features/explore/explore-search";

/* === Route === */

export const Route = createFileRoute("/history")({
  component: HistoryRoute,
  validateSearch: (search) => exploreSearchSchema.parse(search),
});

/* === Components === */

function HistoryRoute() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const selectedIds = selectedIdsFromSearch(search);

  const toggleSelected = useCallback(
    (cycleId: string) => {
      void navigate({
        replace: true,
        search: searchFromSelectedIds(toggleSelectedId(selectedIdsFromSearch(search), cycleId)),
        to: "/history",
      });
    },
    [navigate, search],
  );

  const openCompare = useCallback(() => {
    void navigate({
      search: searchFromSelectedIds(selectedIdsFromSearch(search)),
      to: "/history/compare",
    });
  }, [navigate, search]);

  const openEntry = useCallback(
    (target: ExploreTarget) => {
      const searchState = searchFromSelectedIds(selectedIdsFromSearch(search));
      switch (target.kind) {
        case "cycle":
          void navigate({
            params: { cycleId: target.id },
            search: searchState,
            to: "/history/$cycleId",
          });
          break;
        case "execution":
          void navigate({
            params: { executionId: target.id },
            search: searchState,
            to: "/history/executions/$executionId",
          });
          break;
      }
    },
    [navigate, search],
  );

  return (
    <ExploreSurface
      comparing={false}
      onBack={() => undefined}
      onCompare={openCompare}
      onOpenEntry={openEntry}
      onToggleSelected={toggleSelected}
      selectedIds={selectedIds}
    />
  );
}
