// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ReactNode } from "react";
import { SvelteMount, useSvelteSlots } from "../components/SvelteMount";
import { useBrand } from "../lib/brand-context";
import SidebarLayout from "./SidebarFrame.svelte";
import type { SidebarState } from "./use-sidebar-state";

export function SidebarFrame({ children, ...state }: SidebarState & { children: ReactNode }) {
  const brand = useBrand();
  const { rendered, portals } = useSvelteSlots({ children });
  return (
    <>
      <SvelteMount component={SidebarLayout} props={{ ...state, brand, ...rendered }} />
      {portals}
    </>
  );
}
