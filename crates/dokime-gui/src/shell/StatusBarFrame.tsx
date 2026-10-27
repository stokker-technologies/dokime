/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Footer Geometry, Sidebar Toggle And Frozen Status.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import type { ReactNode } from "react";
import { Badge } from "@/components/ui/badge";

import { type CycleLifecycle, cycleState, STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import type { TestCycle } from "@/lib/types/test";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** The captured cycle's phase and lifecycle, which its frozen pill states. */
export type FrozenCycleStatus = { phase: TestCycle["phase"]; lifecycle: CycleLifecycle };

/* === Components === */
export function StatusBarFrame({ children }: { children: ReactNode }) {
  return (
    <footer className="flex min-w-0 shrink-0 border-t bg-card text-xs text-muted-foreground" style={{ height: "var(--status-bar-height)" }}>
      {children}
    </footer>
  );
}

export function FrozenStatusBar({ status }: { status: FrozenCycleStatus }) {
  const state = cycleState(status.phase, status.lifecycle, false);
  return (
    <StatusBarFrame>
      <div className="flex min-w-0 items-center justify-end gap-6 px-[var(--shell-inset)]">
        <Badge className={STATUS_PILL_CLASS} variant={state.variant}>
          {state.label}
        </Badge>
      </div>
    </StatusBarFrame>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
