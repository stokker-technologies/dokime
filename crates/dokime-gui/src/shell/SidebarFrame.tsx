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
import { selectLogo, useBrand } from "@/lib/brand";
import { BrandMark } from "./BrandMark";
import { useSidebarState } from "./use-sidebar-state";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */
export function SidebarFrame({ children }: { children: ReactNode }) {
  const { collapsed, width } = useSidebarState();
  return (
    <nav
      aria-label="Navigation"
      className="grid min-h-0 shrink-0 grid-rows-[auto_minmax(0,1fr)] border-r bg-card"
      data-collapsed={collapsed || undefined}
      style={{ width }}
    >
      <SidebarBrand collapsed={collapsed} />

      {children}
    </nav>
  );
}

// The Open Sidebar Stacks A Window-Sized Logo Over Its Centred Wordmark.
export function SidebarBrand({ collapsed }: { collapsed: boolean }) {
  const brand = useBrand();
  // Without artwork the wordmark alone stands for the brand.
  const artwork = selectLogo(brand, "transparent", "light") ?? selectLogo(brand, "transparent", "dark");
  if (collapsed) {
    return (
      <div className="flex min-w-0 justify-center px-[var(--shell-gap)] pt-5 pb-8">
        <BrandMark className="h-7 max-w-full object-contain" />
      </div>
    );
  }
  return (
    <div className="flex min-w-0 flex-col items-center gap-3 px-[var(--shell-gap)] pt-6 pb-10" data-slot="sidebar-brand">
      {artwork ? <BrandMark className="h-[var(--sidebar-logo-height)] max-w-full object-contain" /> : null}
      <span className="max-w-full truncate text-center font-heading text-xl font-semibold tracking-wide">{brand.label}</span>
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
