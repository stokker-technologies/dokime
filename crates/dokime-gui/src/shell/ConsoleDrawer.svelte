<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { SessionIdentity } from "../lib/rpc/transport";
  import { storeState } from "../stores/store-state";
  import { useUiStore } from "../stores/ui-store";
  import { consoleBufferKey, consoleRecordsStore, EMPTY_CONSOLE_BUFFER } from "./console-buffer";
  import { observeConsoleRecords } from "./console-records";
  import { resolvedConsoleScope } from "./console-view-model";
  import ConsoleDrawerView from "./ConsoleDrawerView.svelte";

  let { identity, cycleId, executionId }: { identity: SessionIdentity; cycleId: string | null; executionId: string | null } = $props();
  const readDrawer = storeState(useUiStore, (state) => state.console);
  const drawer = $derived(readDrawer());
  const scope = $derived(resolvedConsoleScope(drawer.scope, cycleId, executionId));
  const scopedCycle = $derived(scope === "instance" ? null : cycleId);
  const scopedExecution = $derived(scope === "execution" ? executionId : null);
  const level = $derived(drawer.level);
  const opened = $derived(drawer.open);
  const key = $derived(consoleBufferKey({ cycleId: scopedCycle, executionId: scopedExecution, level }));
  const readBuffers = storeState(consoleRecordsStore, (state) => state.buffers);
  const readFilter = storeState(consoleRecordsStore, (state) => state.filter);
  const buffer = $derived(readBuffers().get(key) ?? EMPTY_CONSOLE_BUFFER);
  const { setConsoleState } = useUiStore.getState();
  const { setFilter, clear, bindSession } = consoleRecordsStore.getState();

  $effect(() => {
    bindSession(identity.generation);
  });
  $effect(() => {
    if (opened) {
      return observeConsoleRecords(identity, { cycleId: scopedCycle, executionId: scopedExecution, level });
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
  streamed={{ ...buffer, clear: () => clear(key) }}
/>
