/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import type { ReactNode } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

/* === Types === */

type AppShellLayoutProps = {
  children: ReactNode;
  header: ReactNode;
  sidebar: ReactNode;
  statusBar?: ReactNode;
  menuBar?: ReactNode;
  console?: ReactNode;
};

/* === Components === */

export function AppShellLayout({ children, header, menuBar, sidebar, statusBar, console }: AppShellLayoutProps) {
  return (
    <TooltipProvider>
      <div
        className={cn(
          "fixed inset-0 grid h-dvh overflow-hidden bg-background text-foreground",
          menuBar ? "grid-rows-[var(--menu-bar-height)_minmax(0,1fr)_auto] pb-16 tablet:pb-0" : "grid-rows-[minmax(0,1fr)_auto]",
        )}
        style={{ fontSize: "var(--shell-text-base)" }}
      >
        {menuBar}
        <div className="flex min-h-0 min-w-0">
          {sidebar}
          <div className="flex min-w-0 flex-1 flex-col overflow-hidden bg-card">
            {header}
            {/* overflow-anchor:none — live plot canvases redraw at stream rate
              inside this region; native scroll anchoring reacts to those
              mutations until Chrome disables it and the viewport drifts. */}
            <main className="min-h-0 min-w-0 flex-1 overflow-y-auto [overflow-anchor:none]">{children}</main>
            {/* The Console Shortens The Evidence Viewport Above The Status Bar. */}
            {console}
          </div>
        </div>
        {statusBar}
      </div>
    </TooltipProvider>
  );
}
