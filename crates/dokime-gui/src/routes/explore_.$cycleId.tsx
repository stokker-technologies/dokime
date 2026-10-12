/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute, redirect } from "@tanstack/react-router";

/* === Local Imports === */

import { exploreSearchSchema } from "@/features/explore/explore-search";

/* === Route === */

export const Route = createFileRoute("/explore_/$cycleId")({
  beforeLoad: ({ params, search }) => {
    throw redirect({
      params: { cycleId: params.cycleId },
      replace: true,
      search,
      to: "/history/$cycleId",
    });
  },
  validateSearch: (search) => exploreSearchSchema.parse(search),
});
