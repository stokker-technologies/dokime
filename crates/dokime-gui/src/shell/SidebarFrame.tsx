/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Sidebar Frame; The Owning Surface Supplies Navigation.
 */

/* === Imports === */

// Standard Library Imports

import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
// Third Party Imports
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { BrandMark } from "./BrandMark";
import type { SidebarState } from "./use-sidebar-state";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */
export function SidebarFrame({ children, collapsed, width, mode, toggle, resize }: SidebarState & { children: ReactNode }) {
  if (mode === "phone") return null;
  return (
    <nav
      aria-label="Navigation"
      className="relative grid min-h-0 shrink-0 grid-rows-[auto_minmax(0,1fr)] bg-background"
      data-collapsed={collapsed || undefined}
      style={{ width }}
    >
      <div className="flex min-w-0 items-center">
        <div className="min-w-0 flex-1">
          <SidebarBrand collapsed={collapsed} />
        </div>
        {mode === "desktop" ? (
          <Button aria-label={collapsed ? "Expand Sidebar" : "Collapse Sidebar"} size="icon" variant="ghost" onClick={toggle}>
            {collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
          </Button>
        ) : null}
      </div>
      {mode === "desktop" && !collapsed ? (
        <Button
          variant="plain"
          size="inline"
          type="button"
          aria-label="Resize Sidebar"
          className="absolute inset-y-0 right-0 z-10 w-1 cursor-col-resize touch-none hover:bg-primary"
          onKeyDown={(event) => {
            if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
            event.preventDefault();
            const width = event.currentTarget.parentElement?.getBoundingClientRect().width;
            if (width !== undefined) {
              resize(width + (event.key === "ArrowRight" ? 4 : -4));
            }
          }}
          onPointerDown={(event) => event.currentTarget.setPointerCapture(event.pointerId)}
          onPointerMove={(event) => {
            if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
            const left = event.currentTarget.parentElement?.getBoundingClientRect().left ?? 0;
            resize(event.clientX - left);
          }}
          onPointerUp={(event) => event.currentTarget.releasePointerCapture(event.pointerId)}
        />
      ) : null}

      {children}
    </nav>
  );
}

// A Compact Brand Row Above Navigation.
function SidebarBrand({ collapsed }: { collapsed: boolean }) {
  return (
    <div className={collapsed ? "min-w-0 px-2 py-3" : "min-w-0 px-4 py-3"} data-slot="sidebar-brand">
      <BrandMark className="sidebar-brand" logoOnly={collapsed} />
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
