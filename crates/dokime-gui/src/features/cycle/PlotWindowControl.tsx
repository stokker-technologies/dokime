// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentProps } from "svelte";
import { SvelteMount } from "../../components/SvelteMount";
import PlotWindowControlView from "./PlotWindowControl.svelte";

export function PlotWindowControl(props: ComponentProps<typeof PlotWindowControlView>) {
  return props.live ? <SvelteMount component={PlotWindowControlView} props={props} /> : null;
}
