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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useRouter, useRouterState } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { type ReactNode, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { focusPresentation, useFocusMode } from "./focus-mode";
import { PageChromeProvider, usePageChromeState } from "./page-chrome";
import type { FrozenCycleStatus } from "./StatusBarFrame";

// Local Imports

/* === Re-Exports === */

/* === Constants === */
/** The four destinations, which have nothing above them to go back to. */
const TOP_LEVEL = new Set(["/cycles", "/library", "/history", "/settings"]);

/* === Types === */
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

/* === Components === */
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
        <div className="flex min-w-0 flex-1 flex-col">
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
      {/* The status bar spans the window: its first cell is the sidebar's
          column and owns the collapse toggle. */}
      {statusBar}
    </div>
  );
}

function PageBar({ focused, frozen }: { focused: boolean; frozen: FrozenShell | null }) {
  const chrome = usePageChromeState();
  const { actions, heading } = focusPresentation(focused, frozen !== null);
  /* A frozen file's page bar names the file, not the cycle: the cycle's own
     label is one line below in the workspace header, and printing it twice
     wasted the only persistent position the report's provenance had. */
  const title = frozen ? frozenReportTitle(frozen.generatedAt) : chrome.title;

  return (
    /* The bar is its controls plus air: `--bar-pad-y` above and below a
       `--bar-control-height` control, `--bar-gap` between page actions, and the
       shell inset at the right edge. It sits on the content surface with no
       rule of its own: the title belongs to the page below it. */
    <header
      className="flex shrink-0 items-center justify-between gap-[var(--bar-gap)] px-[var(--shell-inset)] py-[var(--bar-pad-y)]"
      style={{ height: "var(--page-bar-height)" }}
    >
      {/* The chevron cell stays reserved on a top-level destination, and in
          the report which has nowhere to go back to, so the title sits on one
          line down every route. Focus drops the whole heading; the empty cell
          keeps the actions at the right edge. */}
      <div className="flex min-w-0 items-center gap-[var(--shell-gap)]">
        {heading ? (
          <>
            <span className="flex size-[var(--control-height)] shrink-0 items-center justify-center">{frozen ? null : <BackButton />}</span>
            <h1 className="min-w-0 truncate font-heading font-semibold tracking-wide">{title}</h1>
          </>
        ) : null}
      </div>
      {/* The slot keeps its width when a page publishes no action so the title
          never shifts between routes. */}
      <div className="flex min-w-32 shrink-0 items-center justify-end gap-[var(--bar-gap)] [&_[data-slot=button]:not([class*='h-['])]:h-[var(--bar-control-height)]">
        {actions ? chrome.actions : null}
      </div>
    </header>
  );
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
  const member = useRouterState({
    select: (state) => {
      const search = state.location.search as { caseId?: unknown; executionId?: unknown };
      return Boolean(search.caseId || search.executionId);
    },
  });
  const lastOutsideSettings = useRef<string | null>(null);

  const inSettings = pathname === "/settings" || pathname.startsWith("/settings/");
  if (!inSettings) lastOutsideSettings.current = pathname;

  if (TOP_LEVEL.has(pathname)) return null;
  if (/^\/settings\/plugins\/.+/.test(pathname)) return "/settings/plugins";
  if (inSettings) return lastOutsideSettings.current ?? "/cycles";
  // An execution page goes back to its cycle; the cycle page to Cycles.
  if (pathname.startsWith("/cycles/")) return member ? `${pathname}?view=cycle` : "/cycles";
  if (pathname.startsWith("/history/")) return "/history";
  if (pathname.startsWith("/library/")) return "/library";
  return null;
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
