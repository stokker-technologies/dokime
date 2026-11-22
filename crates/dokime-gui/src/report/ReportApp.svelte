<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import DokimeToaster from "../components/DokimeToaster.svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import type { CycleSelection } from "../features/cycle/cycle-workspace-model";
  import type { CycleSnapshotSource } from "../features/cycle/snapshot-source";
  import type { Brand } from "../lib/brand";
  import type { NotificationFailure } from "../lib/notification-details";
  import { occurrenceKeys } from "../lib/utils";
  import ReportCycleWorkspace from "./ReportCycleWorkspace.svelte";
  import ReportShell from "./ReportShell.svelte";

  type Props =
    | { source: CycleSnapshotSource; executionId: string | null; brand: Brand; failure?: never }
    | { source?: never; executionId?: never; brand?: never; failure: NotificationFailure };
  let { source, executionId, brand, failure }: Props = $props();
  let selected = $state<CycleSelection>({});
  const selection = $derived(executionId ? { executionId } : selected);
  const warnings = $derived(occurrenceKeys(source?.warnings ?? [], (warning) => warning));

  function select(next: CycleSelection) {
    selected = next;
  }
</script>

{#if source && brand}
  <ReportShell {brand} {selection} {source} onSelect={executionId === null ? select : undefined}>
    {#if warnings.length}
      <aside aria-label="Report Notices" class="mx-4 mt-4 rounded-lg border border-warning/40 bg-warning/10 px-4 py-3 text-sm">
        <ul class="grid gap-2">
          {#each warnings as { key, value } (key)}
            <li>{value}</li>
          {/each}
        </ul>
      </aside>
    {/if}
    <ReportCycleWorkspace {selection} {source} />
  </ReportShell>
{:else if failure}
  <ErrorCard {failure} />
{/if}
<DokimeToaster />
