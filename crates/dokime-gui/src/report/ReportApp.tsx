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

/* === External Imports === */

import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { useState } from "react";

/* === Local Imports === */

import { DokimeToaster } from "@/components/DokimeToaster";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import type { CycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { CycleSnapshotProvider, useCycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { ReportCycleWorkspace } from "./ReportCycleWorkspace";
import { ReportShell } from "./ReportShell";

/* === Constants === */

/* === Types === */

type ReportAppProps = {
  source: CycleSnapshotSource;
  executionId: string | null;
};

/* === Components === */

export function ReportApp({ source, executionId }: ReportAppProps) {
  /* The snapshot is provided above the router, and the selection lives inside
     it: a component the router owns must hold its own state, or the router
     never re-renders it when that state changes. */
  const [router] = useState(() => reportRouter(executionId));
  return (
    <>
      <CycleSnapshotProvider source={source}>
        <RouterProvider router={router} />
      </CycleSnapshotProvider>
      {/* Copy and download confirm themselves here as they do live. */}
      <DokimeToaster />
    </>
  );
}

/* === Private Components === */

/** The routed surface: the shell, the workspace, and which case is open. */
function ReportSurface({ executionId }: { executionId: string | null }) {
  const source = useCycleSnapshotSource();
  const [selection, setSelection] = useState<CycleSelection>(executionId ? { executionId } : {});

  if (!source) {
    return null;
  }

  const plan = source.plan;
  const cycleId = plan?.test_cycle.test_cycle_id ?? "";
  const title = plan?.test_cycle.title ?? "Frozen Report";

  return (
    <ReportShell
      executionId={executionId}
      frozen={{
        selection,
        cycleId,
        generatedAt: source.generatedAt,
        onSelect: executionId ? undefined : setSelection,
        title,
      }}
    >
      {source.warnings?.length ? (
        <aside aria-label="Report Notices" className="mx-4 mt-4 rounded-lg border border-warning/40 bg-warning/10 px-4 py-3 text-sm">
          <ul className="grid gap-2">
            {source.warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </aside>
      ) : null}
      <ReportCycleWorkspace selection={selection} source={source} />
    </ReportShell>
  );
}

/* === Private Functions === */

function reportRouter(executionId: string | null) {
  return createRouter({
    history: createMemoryHistory({ initialEntries: ["/"] }),
    routeTree: createRootRoute({ component: () => <ReportSurface executionId={executionId} /> }),
  });
}
