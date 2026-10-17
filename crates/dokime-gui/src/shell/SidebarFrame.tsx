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

// Third Party Imports
import type { ReactNode } from "react";
import { useBrand } from "@/lib/brand";
import { BrandMark } from "./BrandMark";
import { useSidebarState } from "./use-sidebar-state";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */
export function SidebarFrame({ children }: { children: ReactNode }) {
  const { label } = useBrand();
  const { collapsed, width } = useSidebarState();
  return (
    <nav
      aria-label="Navigation"
      className="grid min-h-0 shrink-0 grid-rows-[auto_minmax(0,1fr)] border-r bg-card"
      data-collapsed={collapsed || undefined}
      style={{ width }}
    >
      {/* The mark and the product name, once, as one centred block at the top
          of the one sidebar. It is exactly the page bar's height, so the brand
          bar and the page title bar end on one line at every density. The
          height is fixed, so the first destination starts on the same y
          whether the rail is open or collapsed. */}
      <div className="grid place-items-center px-[var(--shell-gap)]" style={{ height: "var(--page-bar-height)" }}>
        <div className="flex min-w-0 items-center justify-center gap-2">
          <BrandMark />
          {collapsed ? null : <span className="truncate font-heading font-semibold tracking-wide">{label}</span>}
        </div>
      </div>

      {children}
    </nav>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
