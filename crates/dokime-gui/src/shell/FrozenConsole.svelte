<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import SelectField from "../components/ui/SelectField.svelte";
  import type { HubRecord } from "../lib/types";
  import ConsoleRecordList from "./ConsoleRecordList.svelte";
  import { type ConsoleLevel, consoleLevelOptions, visibleConsoleRecords } from "./console-view-model";

  let { records }: { records: HubRecord[] } = $props();
  let level = $state<ConsoleLevel>("info");
  let follow = $state(true);
  const rows = $derived(visibleConsoleRecords(records, level, ""));
</script>

<section aria-label="Console" class="grid gap-2">
  <div class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3">
    <h3 class="font-sans text-sm leading-6 font-semibold tracking-wide uppercase">Console</h3>
    <SelectField label="Console Level" class="w-32" size="sm" value={level} options={consoleLevelOptions} onChange={(next) => {
      level = next;
    }} />
  </div>
  <div class="rounded-lg border">
    <ConsoleRecordList class="h-96" empty="No records at this level." {follow} {rows} onFollowChange={(next) => {
      follow = next;
    }} />
  </div>
</section>
