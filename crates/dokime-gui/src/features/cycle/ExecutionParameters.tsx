// Dokime — SPDX-License-Identifier: Apache-2.0
import { SvelteMount } from "../../components/SvelteMount";
import ExecutionParameterGrid from "./ExecutionParameters.svelte";
import type { ExecutionParameterRow } from "./execution-sections-model";

export function ExecutionParameters({ rows }: { rows: ExecutionParameterRow[] }) {
  return <SvelteMount component={ExecutionParameterGrid} props={{ rows }} />;
}
