/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Application And Frozen Report Frame; Resourceful Regions Are Supplied.
 */

import { useRouter, useRouterState } from "@tanstack/react-router";
import { type ReactNode, useEffect, useRef } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { focusPresentation, useFocusMode } from "./focus-mode";
import { settingsParent } from "./navigation-model";
import { PanelHeader } from "./PageShell";
import { PageChromeProvider, usePageChromeState } from "./page-chrome";
import type { FrozenCycleStatus } from "./StatusBarFrame";
import { useNavigationMode } from "./use-sidebar-state";

/** The four destinations, which have nothing above them to go back to. */
const TOP_LEVEL = new Set(["/cycles", "/library", "/history", "/settings"]);

export type FrozenShell = {
  selection: { caseId?: string; executionId?: string };
  cycleId: string;
  status: FrozenCycleStatus;
  /** ISO timestamp the report was generated at. */
  generatedAt: string | null;
  title: string;
  onSelect: (selection: { caseId?: string; executionId?: string }) => void;
};

type AppShellLayoutProps = {
  children: ReactNode;
  frozen: FrozenShell | null;
  sidebar: ReactNode;
  statusBar: ReactNode;
  menuBar?: ReactNode;
  console?: ReactNode;
};

export function AppShellLayout(props: AppShellLayoutProps) {
  return (
    <TooltipProvider>
      <PageChromeProvider>
        <AppFrame {...props} />
      </PageChromeProvider>
    </TooltipProvider>
  );
}

function AppFrame({ children, frozen, menuBar, sidebar, statusBar, console }: AppShellLayoutProps) {
  const { focused, setFocused } = useFocusMode();

  /* The focus flag also drives `styles.css`, which grows the plot cards. */
  useEffect(() => {
    if (focused) document.documentElement.dataset.focus = "on";
    else delete document.documentElement.dataset.focus;
    return () => {
      delete document.documentElement.dataset.focus;
    };
  }, [focused]);

  useEffect(() => {
    if (!focused) return undefined;
    const leave = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) setFocused(false);
    };
    window.addEventListener("keydown", leave);
    return () => window.removeEventListener("keydown", leave);
  }, [focused, setFocused]);

  return (
    <div
      className={cn(
        "fixed inset-0 grid h-dvh overflow-hidden bg-background text-foreground",
        !frozen && "pb-16 tablet:pb-0",
        frozen ? "grid-rows-[minmax(0,1fr)_auto]" : "grid-rows-[var(--menu-bar-height)_minmax(0,1fr)_auto]",
      )}
      style={{ fontSize: "var(--shell-text-base)" }}
    >
      {/* The menu bar spans the window above the sidebar, at its own height
          token so the rows below never move when a menu opens. A frozen
          report has none: it neither navigates nor reaches the desktop. */}
      {menuBar}
      <div className="flex min-h-0 min-w-0">
        {sidebar}
        <div className="m-2 tablet:ml-0 flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border bg-card shadow-sm">
          <PageBar focused={focused} frozen={frozen} />
          {/* overflow-anchor:none — live plot canvases redraw at stream rate
              inside this region; native scroll anchoring reacts to those
              mutations until Chrome disables it and the viewport drifts. */}
          <main className="min-h-0 min-w-0 flex-1 overflow-y-auto [overflow-anchor:none]">{children}</main>
          {/* The console sits above the status bar and spans the main column,
              so the evidence shortens rather than the window growing. A frozen
              report has no drawer: its records belong to each case section. */}
          {console}
        </div>
      </div>
      {/* Execution focus stays visible across destinations. */}
      {statusBar}
    </div>
  );
}

function PageBar({ focused, frozen }: { focused: boolean; frozen: FrozenShell | null }) {
  const chrome = usePageChromeState();
  const presentation = focusPresentation(focused, frozen !== null);
  if (frozen)
    return (
      <PanelHeader
        title={frozenReportTitle(frozen.generatedAt)}
        onBack={frozen.selection.caseId || frozen.selection.executionId ? () => frozen.onSelect({}) : undefined}
      />
    );
  return (
    <LivePanelHeader
      focused={focused}
      title={presentation.heading ? chrome.title : ""}
      actions={presentation.actions ? chrome.actions : null}
      overflow={chrome.overflow}
    />
  );
}

function LivePanelHeader({
  title,
  actions,
  overflow,
  focused,
}: {
  title: string;
  actions: ReactNode;
  overflow: ReactNode;
  focused: boolean;
}) {
  const parent = useParentRoute();
  const router = useRouter();
  return (
    <PanelHeader
      title={title}
      actions={actions}
      overflow={overflow}
      onBack={parent && !focused ? () => router.history.push(parent) : undefined}
    />
  );
}

/**
 * What a frozen report's page bar says: what the file is, and when it was
 * written. Provenance belongs beside the document's identity, where a reader
 * cannot miss it, rather than in footer text.
 */
export function frozenReportTitle(generatedAt: string | null): string {
  const at = generatedAt ? new Date(generatedAt) : null;
  if (!at || Number.isNaN(at.getTime())) return "Frozen Report";
  return `Frozen Report \u00b7 Generated ${at.toLocaleString()}`;
}

/**
 * The panel the Back chevron leaves to, or null on a top-level destination.
 *
 * A settings section has no structural parent — the operator came to it from
 * whatever they were doing — so it goes back to the last route outside
 * Settings, and to Cycles when this session has none.
 */
function useParentRoute(): string | null {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const member = useRouterState({
    select: (state) => {
      const search = state.location.search as { caseId?: unknown; executionId?: unknown };
      return Boolean(search.caseId || search.executionId);
    },
  });
  const mode = useNavigationMode();
  const lastOutsideSettings = useRef<string | null>(null);

  const inSettings = pathname === "/settings" || pathname.startsWith("/settings/");
  if (!inSettings) lastOutsideSettings.current = pathname;

  if (TOP_LEVEL.has(pathname)) return null;
  if (/^\/settings\/plugins\/.+/.test(pathname)) return "/settings/plugins";
  if (inSettings) return mode === "phone" ? settingsParent(pathname) : (lastOutsideSettings.current ?? "/cycles");
  // An execution page goes back to its cycle; the cycle page to Cycles.
  if (pathname.startsWith("/cycles/")) return member ? `${pathname}?view=cycle` : "/cycles";
  if (pathname.startsWith("/history/")) return "/history";
  if (pathname.startsWith("/library/")) return "/library";
  return null;
}
