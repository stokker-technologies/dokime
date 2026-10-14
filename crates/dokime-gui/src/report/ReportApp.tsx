/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The frozen report, which is the application with `live={false}`.
 *
 * There is no report frame and no report workspace: this file is boot, not
 * rendering. It puts the three things the shared components expect around
 * them — a query client nothing fetches through, a router nothing navigates
 * with, and the snapshot the workspace's hooks read instead of the backend —
 * and then mounts `AppShell` and `CycleWorkspace`.
 *
 * The router is a memory router with a root route and no children. The shell
 * reads the location (focus mode, the back chevron, the console scope) through
 * router hooks; giving it a router that goes nowhere is a smaller, more honest
 * change than teaching five components to survive without one.
 *
 * Selection is React state here rather than a URL: a single file has no
 * address bar of its own to keep it in.
 */

/* === External Imports === */

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { useMemo, useState } from "react";

/* === Local Imports === */

import { DokimeToaster } from "@/components/DokimeToaster";
import { CycleWorkspace } from "@/features/cycle/CycleWorkspace";
import { CycleSnapshotProvider, useCycleSnapshotSource } from "@/features/cycle/snapshot-source";
import type { ReportPayload } from "@/lib/report-types";
import { AppShell } from "@/shell/AppShell";
import { cycleSnapshotSource } from "./report-snapshot";

/* === Constants === */

/**
 * Nothing is ever fetched: every hook on this path answers from the snapshot
 * and leaves its query disabled. The client exists because those hooks still
 * call `useQuery`, which is what keeps one set of components.
 */
const idleQueryClient = new QueryClient({
  defaultOptions: { queries: { gcTime: Infinity, retry: false, staleTime: Infinity } },
});

const reportRouter = createRouter({
  history: createMemoryHistory({ initialEntries: ["/"] }),
  routeTree: createRootRoute({ component: ReportSurface }),
});

/* === Components === */

export function ReportApp({ payload }: { payload: ReportPayload }) {
  const source = useMemo(() => cycleSnapshotSource(payload), [payload]);

  /* The snapshot is provided above the router, and the selection lives inside
     it: a component the router owns must hold its own state, or the router
     never re-renders it when that state changes. */
  return (
    <QueryClientProvider client={idleQueryClient}>
      <CycleSnapshotProvider source={source}>
        <RouterProvider router={reportRouter} />
      </CycleSnapshotProvider>
      {/* Copy and download confirm themselves here as they do live. */}
      <DokimeToaster />
    </QueryClientProvider>
  );
}

/* === Private Components === */

/** The routed surface: the shell, the workspace, and which case is open. */
function ReportSurface() {
  const source = useCycleSnapshotSource();
  const [selection, setSelection] = useState<{ caseId?: string; executionId?: string }>({});

  if (!source) return null;

  const plan = source.plan;
  const cycleId = plan?.test_cycle.test_cycle_id ?? "";
  const title = plan?.test_cycle.title ?? "Frozen Report";

  return (
    <AppShell
      frozen={{
        selection,
        cycleId,
        generatedAt: source.generatedAt,
        onSelect: setSelection,
        status: plan?.test_cycle.status ?? "planned",
        title,
      }}
      live={false}
    >
      <CycleWorkspace
        caseId={selection.caseId}
        cycleId={cycleId}
        executionId={selection.executionId}
        live={false}
        onSelectionChange={setSelection}
      />
    </AppShell>
  );
}

/* === Private Functions === */
