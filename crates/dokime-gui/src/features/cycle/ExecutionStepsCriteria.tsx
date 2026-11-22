// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentProps } from "svelte";
import { SvelteMount } from "../../components/SvelteMount";
import ExecutionStepsCriteriaView from "./ExecutionStepsCriteria.svelte";

export function ExecutionStepsCriteria(props: ComponentProps<typeof ExecutionStepsCriteriaView>) {
  return <SvelteMount component={ExecutionStepsCriteriaView} props={props} />;
}
