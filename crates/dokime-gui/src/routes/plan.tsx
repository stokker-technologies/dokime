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

import { workflowSearchSchema } from "@/routes/route-state";

/* === Route === */

/** Legacy workflow lens. The cycle workspace replaced all three lenses. */
export const Route = createFileRoute("/plan")({
  beforeLoad: ({ search }) => {
    if (search.cycleId) {
      throw redirect({
        params: { cycleId: search.cycleId },
        replace: true,
        search: { executionId: search.executionId },
        to: "/cycles/$cycleId",
      });
    }
    throw redirect({ replace: true, to: "/cycles" });
  },
  validateSearch: workflowSearchSchema,
});
