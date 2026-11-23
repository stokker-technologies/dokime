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
import { type ReactNode, useEffect, useRef } from "react";

/* === Local Imports === */
import { useRuntimeStream } from "@/app/live/use-runtime-stream";
import { PluginStartNotifications } from "@/features/settings/plugins/plugin-queries";
import { AppShellLayout } from "@/shell/AppShellLayout";
import { ConsoleDrawer } from "./ConsoleDrawer";
import { MenuBar } from "./MenuBar";
import { useRouteCycleScope } from "@/shell/route-scope";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { focusPresentation, useFocusMode } from "./focus-mode";
import { settingsParent } from "./navigation-model";
import { PanelHeader } from "./PageShell";
import { PageChromeProvider, usePageChromeState } from "./page-chrome";
import { useNavigationMode } from "./use-navigation-mode";

/* === Constants === */

const TOP_LEVEL = new Set(["/cycles", "/library", "/history", "/settings"]);

/* === Types === */
export type AppShellProps = { children: ReactNode };

/* === Components === */
export function AppShell(props: AppShellProps) {
  useRuntimeStream();
  return (
    <PageChromeProvider>
      <AppShellLayout
        header={<DesktopPanelHeader />}
        menuBar={<MenuBar />}
        console={<RoutedConsoleDrawer />}
        sidebar={<Sidebar />}
        statusBar={<StatusBar />}
      >
        <PluginStartNotifications />
        {props.children}
      </AppShellLayout>
    </PageChromeProvider>
  );
}

/* === Private Components === */
function RoutedConsoleDrawer() {
  const { cycleId, executionId } = useRouteCycleScope();
  return <ConsoleDrawer cycleId={cycleId} executionId={executionId} />;
}

function DesktopPanelHeader() {
  const { focused, setFocused } = useFocusMode();

  /* The focus flag also drives `styles.css`, which grows the plot cards. */
  useEffect(() => {
    if (focused) {
      document.documentElement.dataset.focus = "on";
    } else {
      delete document.documentElement.dataset.focus;
    }
    return () => {
      delete document.documentElement.dataset.focus;
    };
  }, [focused]);

  useEffect(() => {
    if (!focused) {
      return undefined;
    }
    const leave = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) {
        setFocused(false);
      }
    };
    window.addEventListener("keydown", leave);
    return () => window.removeEventListener("keydown", leave);
  }, [focused, setFocused]);

  const chrome = usePageChromeState();
  const presentation = focusPresentation(focused);
  const parent = useParentRoute();
  const router = useRouter();
  return (
    <PanelHeader
      focused={focused}
      title={presentation.heading ? chrome.title : ""}
      actions={presentation.actions ? chrome.actions : null}
      overflow={chrome.overflow}
      onBack={parent && !focused ? () => router.history.push(parent) : undefined}
    />
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
  const mode = useNavigationMode();
  const lastOutsideSettings = useRef<string | null>(null);

  const inSettings = pathname === "/settings" || pathname.startsWith("/settings/");
  if (!inSettings) {
    lastOutsideSettings.current = pathname;
  }

  if (TOP_LEVEL.has(pathname)) {
    return null;
  }
  if (/^\/settings\/plugins\/.+/.test(pathname)) {
    return "/settings/plugins";
  }
  if (inSettings) {
    return mode === "phone" ? settingsParent(pathname) : (lastOutsideSettings.current ?? "/cycles");
  }
  // An execution page goes back to its cycle; the cycle page to Cycles.
  if (pathname.startsWith("/cycles/")) {
    return member ? `${pathname}?view=cycle` : "/cycles";
  }
  if (pathname.startsWith("/history/")) {
    return "/history";
  }
  if (pathname.startsWith("/library/")) {
    return "/library";
  }
  return null;
}
