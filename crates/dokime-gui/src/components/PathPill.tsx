// Dokime — SPDX-License-Identifier: Apache-2.0
import PathPillComponent, { type PathPillProps } from "./PathPill.svelte";
import { SvelteMount } from "./SvelteMount";

export function PathPill(props: PathPillProps) {
  return <SvelteMount component={PathPillComponent} props={props} />;
}
