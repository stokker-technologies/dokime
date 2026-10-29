/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import { z } from "zod";

import { CycleWorkspace } from "@/features/cycle/CycleWorkspace";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { focusSearchSchema } from "@/shell/focus-mode";

const optionalId = z
  .preprocess((value) => {
    const candidate = Array.isArray(value) ? value.at(-1) : value;
    if (typeof candidate !== "string") {
      return undefined;
    }
    const trimmed = candidate.trim();
    return trimmed.length > 0 ? trimmed : undefined;
  }, z.string().optional())
  .catch(undefined);

const cycleWorkspaceSearchSchema = z.object({
  caseId: optionalId,
  executionId: optionalId,
  /** The operator chose the cycle itself, so nothing follows the active execution. */
  view: z.enum(["cycle"]).optional().catch(undefined),
  /** Focus survives a reload and can be linked; see `shell/focus-mode`. */
  focus: focusSearchSchema,
});

export const Route = createFileRoute("/cycles_/$cycleId")({
  component: CycleWorkspaceRoute,
  validateSearch: (search) => cycleWorkspaceSearchSchema.parse(search),
});

function CycleWorkspaceRoute() {
  const { cycleId } = Route.useParams();
  const { caseId, executionId, focus, view } = Route.useSearch();
  const navigate = useNavigate();

  const selectionChanged = useCallback(
    (next: CycleSelection) => {
      void navigate({
        params: { cycleId },
        replace: true,
        search: { caseId: next.caseId, executionId: next.executionId, focus, view: next.view },
        to: "/cycles/$cycleId",
      });
    },
    [cycleId, focus, navigate],
  );

  return (
    <CycleWorkspace
      caseId={caseId}
      cycleId={cycleId}
      executionId={executionId}
      key={cycleId}
      view={view}
      onSelectionChange={selectionChanged}
    />
  );
}
