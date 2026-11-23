<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { titleCaseIdentifier } from "../../components/start-cycle-form-model";
  import type { SuiteTemplateEntry } from "../../lib/types";

  let { template }: { template: SuiteTemplateEntry } = $props();
  const steps = $derived([...template.steps].sort((left, right) => left.order - right.order));
</script>

{#if steps.length}
  <section aria-label="Procedure Steps" class="mx-4 grid gap-2 rounded-lg bg-muted/20 p-3">
    <h3 class="text-sm font-semibold">Procedure Steps</h3>
    <ol class="grid gap-2 sm:grid-cols-2">
      {#each steps as entry, index (entry.step_id)}
        <li data-step-id={entry.step_id} class="flex items-start gap-2 text-sm">
          <span class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/15 text-xs tabular-nums text-primary"
            >{index + 1}</span
          >
          <span class="pt-0.5" title={entry.description ?? undefined}>{entry.label ?? titleCaseIdentifier(entry.method_name)}</span>
        </li>
      {/each}
    </ol>
  </section>
{/if}
