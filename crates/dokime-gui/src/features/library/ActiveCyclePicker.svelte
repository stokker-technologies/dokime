<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { ConnectedSession } from "../../app/native-session/session";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { storeState } from "../../stores/store-state";
  import { useUiStore } from "../../stores/ui-store";
  import NewCycleDialog from "../cycles-home/NewCycleDialog.svelte";
  import ActiveCyclePickerList from "./ActiveCyclePickerList.svelte";
  import type { ActiveCycleState } from "./use-active-cycle";

  let { session, active }: { session: ConnectedSession; active: ActiveCycleState } = $props();
  const readOpen = storeState(useUiStore, (state) => state.activeCyclePickerOpen);
  const { setUiState } = useUiStore.getState();
  let search = $state("");
  let creating = $state(false);

  function close() {
    search = "";
    setUiState({ activeCyclePickerOpen: false });
  }

  function choose(cycleId: string) {
    active.choose(cycleId);
    creating = false;
    close();
  }
</script>

<Dialog.Root
  open={readOpen() && !creating}
  onOpenChange={(open) => {
    if (open) {
      setUiState({ activeCyclePickerOpen: true });
    } else {
      close();
    }
  }}
>
  <DialogContent class="sm:max-w-3xl">
    <div class="flex flex-col gap-2 pr-8" data-slot="dialog-header">
      <Dialog.Title class="font-sans text-base leading-none font-medium">Active Cycle</Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground">Cases added from the Library join the active cycle.</Dialog.Description>
    </div>
    <ActiveCyclePickerList
      activeId={active.cycle?.cycleId ?? null}
      cycles={active.cycles}
      loading={active.loading}
      onChoose={choose}
      onNewCycle={() => (creating = true)}
      onSearch={(next) => (search = next)}
      {search}
    />
  </DialogContent>
</Dialog.Root>

<NewCycleDialog onCreated={choose} onOpenChange={(open) => (creating = open)} open={creating} {session} />
