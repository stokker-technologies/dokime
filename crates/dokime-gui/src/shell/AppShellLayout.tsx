// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ReactNode } from "react";
import { SvelteMount, useSvelteSlots } from "../components/SvelteMount";
import { TooltipProvider } from "../components/ui/tooltip";
import ShellLayout from "./AppShellLayout.svelte";

type Props = {
  children: ReactNode;
  header: ReactNode;
  sidebar: ReactNode;
  statusBar?: ReactNode;
  menuBar?: ReactNode;
  console?: ReactNode;
};

export function AppShellLayout(props: Props) {
  const { rendered, portals } = useSvelteSlots(props);
  return (
    <TooltipProvider>
      <SvelteMount component={ShellLayout} props={rendered} />
      {portals}
    </TooltipProvider>
  );
}
