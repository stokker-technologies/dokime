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
import { useCallback, useContext } from "react";
import { z } from "zod";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { TraceabilitySaveContext } from "@/components/traceability/TraceabilityJsonDialog";
import CycleWorkspace from "@/features/cycle/CycleWorkspace.svelte";
import { useReportDefaults } from "@/lib/application-settings";
import { usePlotPalette } from "@/plots/use-plot-palette";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { focusSearchSchema, useFocusMode } from "@/shell/focus-mode";

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
  /** The operator chose the cycle itself, so nothing follows the active execution. */
  view: z.enum(["cycle"]).optional().catch(undefined),
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
  const { caseId, executionId, focus, view } = Route.useSearch();
  const navigate = useNavigate();
  const focusMode = useFocusMode();
  const session = useNativeSession();
  const plotPalette = usePlotPalette();
  const reportDefaults = useReportDefaults();
  const traceabilitySave = useContext(TraceabilitySaveContext);

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
    <SvelteMount
      key={cycleId}
      component={CycleWorkspace}
      props={{
        cycleId,
        context: { session, plotPalette, reportDefaults, traceabilitySave },
        focus: focusMode,
        selection: { caseId, executionId, view },
        onSelectionChange: selectionChanged,
        openCycle: (id) => void navigate({ to: "/cycles/$cycleId", params: { cycleId: id } }),
        addCases: (id) => void navigate({ to: "/library", search: { appendTo: id } }),
      }}
    />
  );
}
