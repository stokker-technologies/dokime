<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import PathPill from "../../components/PathPill.svelte";
  import { isPathLike } from "../../lib/path-display";
  import { executionParameterColumns, type ExecutionParameterRow } from "./execution-sections-model";

  let { rows }: { rows: ExecutionParameterRow[] } = $props();
  const columns = $derived(executionParameterColumns(rows));
  const labelMinimum = "12rem";
  const halfGap = "2.5rem";
</script>

<dl
  class="grid min-w-0 items-baseline gap-y-2 font-mono text-xs"
  data-testid="execution-parameters"
  style:--parameter-value={`${columns.value}ch`}
  style:--parameter-unit={`${columns.unit}ch`}
  style:--parameter-half={`calc(${labelMinimum} + ${columns.value}ch + ${columns.unit}ch + 2rem)`}
  style:grid-template-columns={`repeat(auto-fill, minmax(min(100%, max(var(--parameter-half), calc((100% - ${halfGap}) / 2))), 1fr))`}
  style:column-gap={halfGap}
>
  {#each rows as row (row.key)}
    <div
      class="grid min-w-0 grid-cols-[minmax(0,1fr)_var(--parameter-value)_var(--parameter-unit)] items-baseline gap-x-2 leading-5"
      data-parameter={row.key}
    >
      <dt class="min-w-0 pr-3 font-sans text-muted-foreground [overflow-wrap:anywhere]" title={row.key}>{row.label}</dt>
      <dd class="min-w-0 text-right tabular-nums [overflow-wrap:anywhere]">
        {#if isPathLike(row.text, row.key)}
          <PathPill align="right" compact copy value={row.text} />
        {:else}
          {row.text}
        {/if}
      </dd>
      <dd class="text-muted-foreground">{row.unit ?? ""}</dd>
    </div>
  {/each}
</dl>
