<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "../app/native-session/session";
  import { createRecordsQuery } from "../features/cycle/live-document.svelte";
  import { failureOf } from "../lib/notification-details";
  import { storeState } from "../stores/store-state";
  import { uiStore } from "../stores/ui-store";
  import { consoleRecordsStore, EMPTY_CONSOLE_BUFFER } from "./console-buffer";
  import { observeConsoleRecords } from "./console-records";
  import { consoleRecordScope, recordsAfter } from "./console-view-model";
  import ConsoleDrawerView from "./ConsoleDrawerView.svelte";

  let { session, cycleId, executionId }: { session: ConnectedSession; cycleId: string | null; executionId: string | null } = $props();
  const readDrawer = storeState(uiStore, (state) => state.console);
  const drawer = $derived(readDrawer());
  const level = $derived(drawer.level);
  const opened = $derived(drawer.open);
  // Cycle and Execution Read the Records the Document Shows; the Instance Reads the Backend's Trace.
  const scope = $derived(consoleRecordScope(drawer.scope, cycleId, executionId));
  const records = createRecordsQuery(
    () => session,
    () => ({ enabled: opened && scope !== null, scope }),
  );
  const readBuffers = storeState(consoleRecordsStore, (state) => state.buffers);
  const readFilter = storeState(consoleRecordsStore, (state) => state.filter);
  const { setConsoleState } = uiStore.getState();
  const { setFilter, bindSession } = consoleRecordsStore.getState();
  const trace = $derived(scope === null ? (readBuffers().get(level) ?? EMPTY_CONSOLE_BUFFER) : null);
  const held = $derived(trace?.records ?? records.data ?? []);
  // Both Sources Outlive the Drawer, so Clear Hides the Rows Already Held Instead of Dropping Them.
  const clearKey = $derived(JSON.stringify(scope));
  let cleared = $state<Record<string, string | undefined>>({});

  $effect(() => {
    bindSession(session.identity.generation);
    cleared = {};
  });
  $effect(() => {
    if (opened && scope === null) {
      return observeConsoleRecords(session.identity, level);
    }
  });
</script>

<ConsoleDrawerView
  {cycleId}
  {executionId}
  {drawer}
  {setConsoleState}
  filter={readFilter()}
  {setFilter}
  streamed={{
    clear: () => {
      cleared[clearKey] = held.at(-1)?.record_id;
    },
    dropped: trace?.dropped ?? 0n,
    error: trace?.error ?? null,
    failure: trace ? trace.failure : records.error ? failureOf(records.error, "Records Unavailable") : undefined,
    records: recordsAfter(held, cleared[clearKey]),
  }}
/>
