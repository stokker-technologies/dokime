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

/** Explore became History; the compare selection survives the move. */
export const Route = createFileRoute("/explore")({
  beforeLoad: ({ search }) => {
    throw redirect({ replace: true, search, to: "/history" });
  },
  validateSearch: (search) => exploreSearchSchema.parse(search),
});
