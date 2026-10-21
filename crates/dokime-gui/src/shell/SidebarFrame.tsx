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
      <div className="flex min-w-0 flex-col items-center gap-3 px-[var(--shell-gap)] pt-5 pb-8">
        <BrandMark className={collapsed ? "h-7 max-w-full object-contain" : "h-[16.67vh] max-w-full object-contain"} />
        {collapsed ? null : (
          <span className="max-w-full truncate text-center font-heading text-xl font-semibold tracking-wide">{label}</span>
        )}
      </div>

      {children}
    </nav>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
