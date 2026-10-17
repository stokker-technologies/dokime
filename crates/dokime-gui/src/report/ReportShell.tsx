/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


/**
 * Frozen Snapshot Composition Of The Shared Shell Frames.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import type { ReactNode } from "react";

// Local Imports
import { CycleSidebarCasesView } from "@/features/cycle/CycleSidebarCasesView";
import { useCycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { AppShellLayout, type FrozenShell } from "@/shell/AppShellLayout";
import { SidebarFrame } from "@/shell/SidebarFrame";
import { FrozenStatusBar } from "@/shell/StatusBarFrame";



/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */
export function ReportShell({ children, frozen }: { children: ReactNode; frozen: FrozenShell }) {
  const snapshot = useCycleSnapshotSource();
  if (!snapshot) throw new Error("A frozen report requires its captured cycle snapshot.");
  return (
    <AppShellLayout
      frozen={frozen}
      sidebar={
        <SidebarFrame>
          <ul className="grid min-h-0 content-start gap-0.5 overflow-y-auto px-[var(--shell-gap)] pt-2 pb-2">
            <CycleSidebarCasesView
              selection={frozen.selection}
              cycleId={frozen.cycleId}
              detailQuery={{
                data: { plan: snapshot.plan, executions: snapshot.executions },
                isError: false,
                error: null,
              }}
              live={false}
              onSelect={frozen.onSelect}
            />
          </ul>
        </SidebarFrame>
      }
      statusBar={<FrozenStatusBar status={frozen.status} />}
    >
      {children}
    </AppShellLayout>
  );
}


/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
