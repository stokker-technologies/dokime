// Dokime — SPDX-License-Identifier: Apache-2.0
import { useState } from "react";
import type { ComponentProps } from "svelte";
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import NewCycleDialog from "@/features/cycles-home/NewCycleDialog.svelte";
import { useUiStore } from "@/stores/ui-store";
import ActiveCycleDestination from "./ActiveCycleInline.svelte";
import ActiveCyclePickerList from "./ActiveCyclePickerList.svelte";
import { useActiveCycle } from "./use-active-cycle";

export function ActiveCycleInline(props: ComponentProps<typeof ActiveCycleDestination>) {
  return <SvelteMount component={ActiveCycleDestination} props={props} />;
}

export function ActiveCyclePicker() {
  const session = useNativeSession();
  const open = useUiStore((state) => state.activeCyclePickerOpen);
  const setUiState = useUiStore((state) => state.setUiState);
  const active = useActiveCycle();
  const [search, setSearch] = useState("");
  const [creating, setCreating] = useState(false);
  const close = () => {
    setSearch("");
    setUiState({ activeCyclePickerOpen: false });
  };

  return (
    <>
      <Dialog onOpenChange={(next) => (next ? setUiState({ activeCyclePickerOpen: true }) : close())} open={open && !creating}>
        <DialogContent className="sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>Active Cycle</DialogTitle>
            <DialogDescription>Cases added from the Library join the active cycle.</DialogDescription>
          </DialogHeader>
          <SvelteMount
            component={ActiveCyclePickerList}
            props={{
              activeId: active.cycle?.cycleId ?? null,
              cycles: active.cycles,
              loading: active.loading,
              onChoose: (cycleId) => {
                active.choose(cycleId);
                close();
              },
              onNewCycle: () => setCreating(true),
              onSearch: setSearch,
              search,
            }}
          />
        </DialogContent>
      </Dialog>
      <SvelteMount
        component={NewCycleDialog}
        props={{
          onCreated: (cycleId) => {
            active.choose(cycleId);
            setCreating(false);
            close();
          },
          onOpenChange: setCreating,
          open: creating,
          session,
        }}
      />
    </>
  );
}
