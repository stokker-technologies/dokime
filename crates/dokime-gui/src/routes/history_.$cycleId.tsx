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

import {
  exploreSearchSchema,
  searchFromSelectedIds,
  selectedIdsFromSearch,
} from "@/features/explore/explore-search";
import { HistoryDetailSurface } from "@/features/explore/HistoryDetailSurface";

/* === Route === */

export const Route = createFileRoute("/history_/$cycleId")({
  component: HistoryCycleDetailRoute,
  validateSearch: (search) => exploreSearchSchema.parse(search),
});

/* === Components === */

function HistoryCycleDetailRoute() {
  const { cycleId } = Route.useParams();
  const search = Route.useSearch();
  const navigate = useNavigate();

  const back = useCallback(() => {
    void navigate({ search: searchFromSelectedIds(selectedIdsFromSearch(search)), to: "/history" });
  }, [navigate, search]);

  const openCycle = useCallback(
    (cycleId: string) => {
      void navigate({
        params: { cycleId },
        search: searchFromSelectedIds(selectedIdsFromSearch(search)),
        to: "/history/$cycleId",
      });
    },
    [navigate, search],
  );

  return (
    <HistoryDetailSurface
      key={cycleId}
      onBack={back}
      onOpenCycle={openCycle}
      target={{ id: cycleId, kind: "cycle" }}
    />
  );
}
