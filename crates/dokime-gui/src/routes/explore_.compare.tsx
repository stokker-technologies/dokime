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

export const Route = createFileRoute("/explore_/compare")({
  beforeLoad: ({ search }) => {
    throw redirect({ replace: true, search, to: "/history/compare" });
  },
  validateSearch: (search) => exploreSearchSchema.parse(search),
});
