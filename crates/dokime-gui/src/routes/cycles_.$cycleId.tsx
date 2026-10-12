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
import { z } from "zod";

/* === Local Imports === */

import { CycleWorkspace } from "@/features/cycle/CycleWorkspace";
import { focusSearchSchema } from "@/shell/focus-mode";
import { usePageChrome } from "@/shell/page-chrome";

/* === Constants === */

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
  /** Focus survives a reload and can be linked; see `shell/focus-mode`. */
  focus: focusSearchSchema,
});

/* === Route === */

export const Route = createFileRoute("/cycles_/$cycleId")({
  component: CycleWorkspaceRoute,
  validateSearch: (search) => cycleWorkspaceSearchSchema.parse(search),
});

/* === Components === */

function CycleWorkspaceRoute() {
  const { cycleId } = Route.useParams();
  const { caseId, executionId, focus } = Route.useSearch();
  const navigate = useNavigate();
  usePageChrome({ title: "Cycle" });

  const selectionChanged = useCallback(
    (next: { caseId?: string; executionId?: string }) => {
      void navigate({
        params: { cycleId },
        replace: true,
        search: { caseId: next.caseId, executionId: next.executionId, focus },
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
      onSelectionChange={selectionChanged}
    />
  );
}
