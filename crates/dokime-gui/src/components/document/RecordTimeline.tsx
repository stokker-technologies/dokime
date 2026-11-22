// Dokime — SPDX-License-Identifier: Apache-2.0
import RecordTimelineComponent, { type RecordTimelineProps } from "./RecordTimeline.svelte";
import { SvelteMount } from "../SvelteMount";

export function RecordTimeline(props: RecordTimelineProps) {
  return <SvelteMount component={RecordTimelineComponent} props={props} />;
}
