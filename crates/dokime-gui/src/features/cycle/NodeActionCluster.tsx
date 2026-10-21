/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The page bar's action cluster, the same on the cycle and the execution
 * viewer and always in one order: the node's primary lifecycle action, Export
 * Evidence, Download Report, the expand control and the node's "…" menu.
 *
 * The primary action is the first action the node's Control card offers when
 * it is not destructive — Start, Pause, Resume, Re-Run, Seal Cycle — wired to
 * the same handler, so the bar and the card never disagree; Stop stays in the
 * card alone. Export Evidence and Download Report are two labelled buttons of
 * one style. Expand names its exit in text while focused: the bar is the only
 * chrome left, and Escape is a shortcut, not the way out.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { Maximize2, Minimize2 } from "lucide-react";
import type { ReactNode } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { focusPresentation, useFocusMode } from "@/shell/focus-mode";
import { CycleExportButton } from "./CycleExportButton";
import { CycleReportButton } from "./CycleReportButton";
import { type CycleMenu, type ExecutionMenu, NodeMenu } from "./NodeMenu";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NodeActionClusterProps = {
  cycleId: string;
  /** The node's primary lifecycle action, or `null` when it has none now. */
  primary: ReactNode;
  /** The node whose evidence and report the buttons save; `null` for a case that never ran. */
  subject: { executionId: string | null; label: string; sealed: boolean } | null;
  menu: CycleMenu | ExecutionMenu;
};

/* === Components === */

export function NodeActionCluster({ cycleId, menu, primary, subject }: NodeActionClusterProps) {
  const focus = useFocusMode();
  const focusLabel = focusPresentation(focus.focused).toggleLabel;
  return (
    <div className="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]" data-node-actions>
      {primary}
      {subject ? (
        <>
          <CycleExportButton cycleId={cycleId} executionId={subject.executionId} sealed={subject.sealed} />
          <CycleReportButton cycleId={cycleId} executionId={subject.executionId} label={subject.label} />
        </>
      ) : null}
      <Button
        aria-label={focusLabel}
        aria-pressed={focus.focused}
        className={focus.focused ? "h-[var(--bar-control-height)] px-3" : "size-[var(--bar-control-height)]"}
        onClick={() => focus.setFocused(!focus.focused)}
        size={focus.focused ? "default" : "icon"}
        title={focus.focused ? `${focusLabel} (Escape)` : focusLabel}
        variant={focus.focused ? "secondary" : "outline"}
      >
        {focus.focused ? (
          <>
            <Minimize2 />
            {focusLabel}
          </>
        ) : (
          <Maximize2 />
        )}
      </Button>
      <NodeMenu menu={menu} />
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
