// Dokime — SPDX-License-Identifier: Apache-2.0
import type { Snippet } from "svelte";

type PageChrome = {
  title: string;
  actions?: Snippet;
  overflow?: Snippet;
};

let readPage = $state.raw<(() => PageChrome) | null>(null);

export function publishPageChrome(read: () => PageChrome) {
  $effect(() => {
    readPage = read;
    return () => {
      if (readPage === read) {
        readPage = null;
      }
    };
  });
}

export function currentPageChrome() {
  return readPage?.();
}
