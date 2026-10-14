/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useRouter, useRouterState } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { type ReactNode, useEffect, useRef } from "react";

/* === Local Imports === */

import { useRuntimeStream } from "@/app/live/use-runtime-stream";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { CycleLifecycle } from "@/features/cycle/cycle-workspace-model";
import { ApplicationMenu } from "@/shell/ApplicationMenu";
import { ConsoleDrawer } from "@/shell/ConsoleDrawer";
import { useFocusMode } from "@/shell/focus-mode";
import { PageChromeProvider, usePageChromeState } from "@/shell/page-chrome";
import { useRouteCycleScope } from "@/shell/route-scope";
import { Sidebar } from "@/shell/Sidebar";
import { StatusBar } from "@/shell/StatusBar";

/* === Types === */

/**
 * What the shell needs when it frames a frozen report instead of a session.
 *
 * The report is always one cycle, so the sidebar shows that cycle's outline
 * rather than the four destinations, the page bar names it, and the status bar
 * states when the file was written instead of what the backend is doing.
 */
export type FrozenShell = {
  selection: { caseId?: string; executionId?: string };
  cycleId: string;
  status: CycleLifecycle;
  /** ISO timestamp the report was generated at. */
  generatedAt: string | null;
  title: string;
  onSelect: (selection: { caseId?: string; executionId?: string }) => void;
};

export type AppShellProps =
  | { children: ReactNode; live?: true }
  | { children: ReactNode; live: false; frozen: FrozenShell };

/* === Constants === */

/** The four destinations, which have nothing above them to go back to. */
const TOP_LEVEL = new Set(["/cycles", "/library", "/history", "/settings"]);

/* === Components === */

/**
 * The application frame: the one contextual sidebar, page bar, main region,
 * status bar. Left and right regions share one inset and one gap so the three
 * horizontal bands line up on the same grid.
 *
 * `live` is the only switch. With it off the same frame carries a frozen
 * report: the same sidebar, page bar and status bar, holding what a file can
 * honestly hold. There is no second shell.
 */
export function AppShell(props: AppShellProps) {
  // The one live runtime subscription for the whole app. It invalidates the
  // runtime projections every page and the status bar read from, so no route
  // opens a second stream. It never connects without a session.
  useRuntimeStream();

  const frozen = props.live === false ? props.frozen : null;

  return (
    <TooltipProvider>
      <PageChromeProvider>
        <AppFrame frozen={frozen}>{props.children}</AppFrame>
      </PageChromeProvider>
    </TooltipProvider>
  );
}

/* === Private Components === */

/**
 * The frame proper, inside the providers so it can read focus.
 *
 * Focus takes the page bar away and leaves the workspace header and the status
 * bar: the operator keeps the run controls and the connection, and gives up
 * only the chrome that names the page they can already see.
 */
function AppFrame({ children, frozen }: { children: ReactNode; frozen: FrozenShell | null }) {
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
      className="fixed inset-0 grid h-dvh grid-rows-[minmax(0,1fr)_auto] overflow-hidden bg-background text-foreground"
      style={{ fontSize: "var(--shell-text-base)" }}
    >
      <div className="flex min-h-0 min-w-0">
        {frozen ? (
          <Sidebar
            selection={frozen.selection}
            cycleId={frozen.cycleId}
            live={false}
            onSelect={frozen.onSelect}
          />
        ) : (
          <Sidebar />
        )}
        <div className="flex min-w-0 flex-1 flex-col">
          {focused ? null : <PageBar frozen={frozen} />}
          {/* overflow-anchor:none — live plot canvases redraw at stream rate
              inside this region; native scroll anchoring reacts to those
              mutations until Chrome disables it and the viewport drifts. */}
          <main className="min-h-0 min-w-0 flex-1 overflow-y-auto [overflow-anchor:none]">
            {children}
          </main>
          {/* The console sits above the status bar and spans the main column,
              so the evidence shortens rather than the window growing. A frozen
              report has no drawer: its records belong to each case section. */}
          {frozen ? null : <RoutedConsoleDrawer />}
        </div>
      </div>
      {/* The status bar spans the window: its first cell is the sidebar's
          column and owns the collapse toggle. */}
      {frozen ? <StatusBar frozen={frozen} live={false} /> : <StatusBar />}
    </div>
  );
}

function PageBar({ frozen }: { frozen: FrozenShell | null }) {
  const chrome = usePageChromeState();
  /* A frozen file's page bar names the file, not the cycle: the cycle's own
     label is one line below in the workspace header, and printing it twice
     wasted the only persistent position the report's provenance had. */
  const title = frozen ? frozenReportTitle(frozen.generatedAt) : chrome.title;

  return (
    /* The bar is its controls plus air: `--bar-pad-y` above and below a
       `--bar-control-height` control, `--bar-gap` between the page action and
       the Application menu, and the shell inset at the right edge. It sits on
       the content surface with no rule of its own: the title belongs to the
       page below it. */
    <header
      className="flex shrink-0 items-center justify-between gap-[var(--bar-gap)] px-[var(--shell-inset)] py-[var(--bar-pad-y)]"
      style={{ height: "var(--page-bar-height)" }}
    >
      <div className="flex min-w-0 items-center gap-[var(--shell-gap)]">
        {/* The chevron cell stays reserved on a top-level destination, and in
            the report which has nowhere to go back to, so the title sits on
            one line down every route. */}
        <span className="flex size-[var(--control-height)] shrink-0 items-center justify-center">
          {frozen ? null : <BackButton />}
        </span>
        <h1 className="min-w-0 truncate font-heading font-semibold tracking-wide">{title}</h1>
      </div>
      {/* The slot keeps its width when a page publishes no action so the title
          never shifts between routes. */}
      <div className="flex min-w-32 shrink-0 items-center justify-end gap-[var(--bar-gap)] [&_[data-slot=button]:not([class*='h-['])]:h-[var(--bar-control-height)]">
        {frozen ? null : chrome.actions}
        {frozen ? null : <ApplicationMenu />}
      </div>
    </header>
  );
}

/** The console, scoped to the cycle and attempt the URL names. */
function RoutedConsoleDrawer() {
  const { cycleId, executionId } = useRouteCycleScope();

  return <ConsoleDrawer cycleId={cycleId} executionId={executionId} />;
}

/* === Public Functions === */

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

/* === Private Components === */

/** The chevron, and the router reads it needs. Live routes only. */
function BackButton() {
  const parent = useParentRoute();
  const router = useRouter();

  if (!parent) return null;

  return (
    <Button
      aria-label="Back"
      className="size-[var(--control-height)]"
      onClick={() => router.history.push(parent)}
      size="icon"
      type="button"
      variant="ghost"
    >
      <ChevronLeft />
    </Button>
  );
}

/* === Private Functions === */

/**
 * The panel the Back chevron leaves to, or null on a top-level destination.
 *
 * A settings section has no structural parent — the operator came to it from
 * whatever they were doing — so it goes back to the last route outside
 * Settings, and to Cycles when this session has none.
 */
function useParentRoute(): string | null {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const lastOutsideSettings = useRef<string | null>(null);

  const inSettings = pathname === "/settings" || pathname.startsWith("/settings/");
  if (!inSettings) lastOutsideSettings.current = pathname;

  if (TOP_LEVEL.has(pathname)) return null;
  if (/^\/settings\/plugins\/.+/.test(pathname)) return "/settings/plugins";
  if (inSettings) return lastOutsideSettings.current ?? "/cycles";
  if (pathname.startsWith("/cycles/")) return "/cycles";
  if (pathname.startsWith("/history/")) return "/history";
  if (pathname.startsWith("/library/")) return "/library";
  return null;
}
