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
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import type { ReactNode } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { type CycleLifecycle, cycleState, STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import type { TestCycle } from "@/lib/types/test";
import { useSidebarState } from "./use-sidebar-state";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** The captured cycle's phase and lifecycle, which its frozen pill states. */
export type FrozenCycleStatus = { phase: TestCycle["phase"]; lifecycle: CycleLifecycle };

/* === Components === */
export function StatusBarFrame({ children }: { children: ReactNode }) {
  const sidebar = useSidebarState();
  return (
    <footer
      className="grid shrink-0 border-t bg-card text-xs text-muted-foreground"
      style={{
        gridTemplateColumns: `${sidebar.width} minmax(0,1fr)`,
        height: "var(--status-bar-height)",
      }}
    >
      <div className="grid place-items-center border-r">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={sidebar.collapsed ? "Expand Sidebar" : "Collapse Sidebar"}
              className="size-[var(--status-bar-height)] rounded-none"
              onClick={sidebar.toggle}
              size="icon-sm"
              variant="ghost"
            >
              {sidebar.collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top">{sidebar.collapsed ? "Expand Sidebar" : "Collapse Sidebar"}</TooltipContent>
        </Tooltip>
      </div>
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
