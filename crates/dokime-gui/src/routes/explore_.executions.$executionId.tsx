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

export const Route = createFileRoute("/explore_/executions/$executionId")({
  beforeLoad: ({ params, search }) => {
    throw redirect({
      params: { executionId: params.executionId },
      replace: true,
      search,
      to: "/history/executions/$executionId",
    });
  },
  validateSearch: (search) => exploreSearchSchema.parse(search),
});
