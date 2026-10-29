/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Primary Lifecycle Command, Focus Control And Secondary Menu. */

import { Maximize2, Minimize2 } from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { focusPresentation, useFocusMode } from "@/shell/focus-mode";
import { CycleExportMenuItem } from "./CycleExportMenuItem";
import { CycleReportMenuItem } from "./CycleReportMenuItem";
import { type CycleMenu, type ExecutionMenu, NodeMenu } from "./NodeMenu";

export type NodeActionClusterProps = {
  cycleId: string;
  /** The node's primary lifecycle action, or `null` when it has none now. */
  primary: ReactNode;
  /** The node whose evidence and report the buttons save; `null` for a case that never ran. */
  subject: { executionId: string | null; label: string; sealed: boolean } | null;
  menu: CycleMenu | ExecutionMenu;
};

export function NodeActionCluster({ cycleId, menu, primary, subject }: NodeActionClusterProps) {
  const focus = useFocusMode();
  const focusLabel = focusPresentation(focus.focused).toggleLabel;
  return (
    <div className="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]" data-node-actions>
      {primary}
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
      <NodeMenu menu={menu}>
        {subject ? (
          <>
            <CycleExportMenuItem cycleId={cycleId} executionId={subject.executionId} sealed={subject.sealed} />
            <CycleReportMenuItem cycleId={cycleId} executionId={subject.executionId} label={subject.label} />
          </>
        ) : null}
      </NodeMenu>
    </div>
  );
}
