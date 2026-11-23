<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import RecordTimeline from "../../components/document/RecordTimeline.svelte";
  import type { HubRecord } from "../../lib/types";
  import ExecutionSectionCard from "./ExecutionSectionCard.svelte";

  let { live, records }: { live: boolean; records: HubRecord[] } = $props();
</script>

<ExecutionSectionCard empty={records.length ? null : "No records yet."} title="Records">
  {#if records.length}
    <RecordTimeline {live} {records} />
  {/if}
</ExecutionSectionCard>
