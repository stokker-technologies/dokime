// Dokime — SPDX-License-Identifier: Apache-2.0
import { createSubscriber } from "svelte/reactivity";
import type { StoreApi } from "zustand/vanilla";

export function storeState<State, Value>(store: Pick<StoreApi<State>, "getState" | "subscribe">, select: (state: State) => Value) {
  const track = createSubscriber((update) => {
    let value = select(store.getState());
    return store.subscribe((next) => {
      const selected = select(next);
      if (!Object.is(value, selected)) {
        value = selected;
        update();
      }
    });
  });
  return () => {
    track();
    return select(store.getState());
  };
}
