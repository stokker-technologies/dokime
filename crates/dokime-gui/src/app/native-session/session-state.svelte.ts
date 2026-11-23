// Dokime — SPDX-License-Identifier: Apache-2.0
import { untrack } from "svelte";
import type { NativeSession } from "./session";

export type SessionOwner = Pick<NativeSession, "snapshot" | "select" | "disconnect"> & {
  subscribe(update: () => void): () => void;
};

export function sessionState(owner: () => SessionOwner) {
  let state = $state.raw(untrack(() => owner().snapshot()));
  $effect(() => {
    const current = owner();
    function update() {
      state = current.snapshot();
    }
    const unsubscribe = current.subscribe(update);
    update();
    return unsubscribe;
  });
  return () => state;
}
