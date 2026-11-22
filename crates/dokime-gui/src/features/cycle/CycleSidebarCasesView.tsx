// Dokime — SPDX-License-Identifier: Apache-2.0
import { SvelteMount } from "../../components/SvelteMount";
import CycleSidebarTree, { type CycleSidebarCasesViewProps } from "./CycleSidebarCasesView.svelte";

export function CycleSidebarCasesView(props: CycleSidebarCasesViewProps) {
  return (
    <li className="grid min-w-0" role="none">
      <SvelteMount component={CycleSidebarTree} props={props} />
    </li>
  );
}
