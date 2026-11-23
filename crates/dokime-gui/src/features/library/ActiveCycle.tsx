// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentProps } from "svelte";
import { SvelteMount } from "@/components/SvelteMount";
import ActiveCycleDestination from "./ActiveCycleInline.svelte";

export function ActiveCycleInline(props: ComponentProps<typeof ActiveCycleDestination>) {
  return <SvelteMount component={ActiveCycleDestination} props={props} />;
}
