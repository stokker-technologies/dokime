// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentProps } from "svelte";
import { SvelteMount } from "../../components/SvelteMount";
import NodeSummaryView from "./NodeSummary.svelte";

export function NodeSummary(props: ComponentProps<typeof NodeSummaryView>) {
  return <SvelteMount component={NodeSummaryView} props={props} />;
}
