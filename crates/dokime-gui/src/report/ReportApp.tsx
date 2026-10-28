/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The frozen report supplies captured evidence to shared presentation views.
 * Its memory router provides presentation state for shell focus and navigation.
 * Selection stays local to the document; no live service is constructed.
 */

import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { useMemo, useState } from "react";

import { DokimeToaster } from "@/components/DokimeToaster";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { CycleSnapshotProvider, useCycleSnapshotSource } from "@/features/cycle/snapshot-source";
import type { ReportPayload } from "@/lib/report-types";
import { ReportCycleWorkspace } from "./ReportCycleWorkspace";
import { ReportShell } from "./ReportShell";
import { cycleSnapshotSource } from "./report-snapshot";

const reportRouter = createRouter({
  history: createMemoryHistory({ initialEntries: ["/"] }),
  routeTree: createRootRoute({ component: ReportSurface }),
});

export function ReportApp({ payload }: { payload: ReportPayload }) {
  const source = useMemo(() => cycleSnapshotSource(payload), [payload]);

  /* The snapshot is provided above the router, and the selection lives inside
     it: a component the router owns must hold its own state, or the router
     never re-renders it when that state changes. */
  return (
    <>
      <CycleSnapshotProvider source={source}>
        <RouterProvider router={reportRouter} />
      </CycleSnapshotProvider>
      {/* Copy and download confirm themselves here as they do live. */}
      <DokimeToaster />
    </>
  );
}

/** The routed surface: the shell, the workspace, and which case is open. */
function ReportSurface() {
  const source = useCycleSnapshotSource();
  const [selection, setSelection] = useState<CycleSelection>({});

  if (!source) return null;

  const plan = source.plan;
  const cycleId = plan?.test_cycle.test_cycle_id ?? "";
  const title = plan?.test_cycle.title ?? "Frozen Report";

  return (
    <ReportShell
      frozen={{
        selection,
        cycleId,
        generatedAt: source.generatedAt,
        onSelect: setSelection,
        status: { lifecycle: plan?.test_cycle.status ?? "planned", phase: plan?.test_cycle.phase ?? "sealed" },
        title,
      }}
    >
      <ReportCycleWorkspace selection={selection} source={source} />
    </ReportShell>
  );
}
