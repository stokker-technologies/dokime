// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
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
  return (
    <SvelteMount
      component={SidebarNavigation}
      props={{
        session,
        brand,
        sidebar,
        pathname,
        cycleId,
        selection,
        selectCycleMember: (cycleId, next) =>
          void navigate({
            params: { cycleId },
            replace: true,
            search: (current: Record<string, unknown>) => ({
              ...current,
              caseId: next.caseId,
              executionId: next.executionId,
              view: next.view,
            }),
            to: "/cycles/$cycleId",
          }),
        addCases: (id) => void navigate({ search: { appendTo: id }, to: "/library" }),
        navigate: (to) => void navigate({ to }),
        openCycle: (id) => void navigate({ params: { cycleId: id }, to: "/cycles/$cycleId" }),
        openSettings: (section) => void navigate({ params: { section }, to: "/settings/$section" }),
      }}
    />
  );
}
