// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import { CycleSidebarCases } from "@/features/cycle/CycleSidebarCases";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { useBrand } from "@/lib/brand-context";
import { cycleIdOfPathname } from "./route-scope";
import SidebarNavigation from "./Sidebar.svelte";
import { useSidebarState } from "./use-sidebar-state";

export function Sidebar() {
  const session = useNativeSession();
  const brand = useBrand();
  const sidebar = useSidebarState();
  const navigate = useNavigate();
  const { selection, pathname } = useRouterState({
    select: (state) => ({
      selection: state.location.search as CycleSelection,
      pathname: state.location.pathname,
    }),
  });
  const cycleId = cycleIdOfPathname(pathname);
  const cycleCases = cycleId ? <CycleSidebarCases key={cycleId} cycleId={cycleId} selection={selection} /> : null;
  const { rendered, portals } = useSvelteSlots({ cycleCases });

  return (
    <>
      <SvelteMount
        component={SidebarNavigation}
        props={{
          session,
          brand,
          sidebar,
          pathname,
          cycleId,
          ...rendered,
          navigate: (to) => void navigate({ to }),
          openCycle: (id) => void navigate({ params: { cycleId: id }, to: "/cycles/$cycleId" }),
          openSettings: (section) => void navigate({ params: { section }, to: "/settings/$section" }),
        }}
      />
      {portals}
    </>
  );
}
