/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { type ReactNode, useState } from "react";
import { CycleSidebarCasesView } from "@/features/cycle/CycleSidebarCasesView";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import type { CycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { AppShellLayout } from "@/shell/AppShellLayout";
import { PanelHeader } from "@/shell/PageShell";
import { SidebarFrame } from "@/shell/SidebarFrame";
import { sidebarFrameWidth, sidebarWidth } from "@/shell/navigation-model";
import { useNavigationMode } from "@/shell/use-navigation-mode";

/* === Types === */

type ReportShellProps = {
  children: ReactNode;
  selection: CycleSelection;
  source: CycleSnapshotSource;
  onSelect?: (selection: CycleSelection) => void;
};

/* === Components === */

export function ReportShell({ children, selection, source, onSelect }: ReportShellProps) {
  const mode = useNavigationMode();
  const [collapsed, setCollapsed] = useState(false);
  const [width, setWidth] = useState(256);
  const shownCollapsed = mode !== "desktop" || collapsed;
  const header = (
    <PanelHeader
      title={frozenReportTitle(source.generatedAt)}
      onBack={onSelect && (selection.caseId || selection.executionId) ? () => onSelect({}) : undefined}
    />
  );
  return (
    <AppShellLayout
      header={header}
      sidebar={
        onSelect ? (
          <SidebarFrame
            mode={mode}
            collapsed={shownCollapsed}
            width={sidebarFrameWidth(mode, shownCollapsed, width)}
            toggle={() => setCollapsed(!collapsed)}
            resize={(next) => setWidth(sidebarWidth(next))}
          >
            <ul className="grid min-h-0 content-start gap-1 overflow-y-auto px-[var(--shell-gap)] pt-2 pb-2">
              <CycleSidebarCasesView
                selection={selection}
                detailQuery={{
                  data: { plan: source.plan, executions: source.executions },
                  isError: false,
                  error: null,
                }}
                live={false}
                onSelect={onSelect}
              />
            </ul>
          </SidebarFrame>
        ) : null
      }
    >
      {children}
    </AppShellLayout>
  );
}

/* === Private Functions === */

function frozenReportTitle(generatedAt: string | null): string {
  const at = generatedAt ? new Date(generatedAt) : null;
  if (!at || Number.isNaN(at.getTime())) {
    return "Frozen Report";
  }
  return `Frozen Report \u00b7 Generated ${at.toLocaleString()}`;
}
