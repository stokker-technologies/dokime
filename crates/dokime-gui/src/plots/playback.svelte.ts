// Dokime — SPDX-License-Identifier: Apache-2.0

type Selection<T> = { data: T; instants: string[]; index: number; playing: boolean };

/** How Long Playback Shows Each Captured Instant. */
export const PLAYBACK_FRAME_MS = 200;

export function plotPlayback<T>(incoming: () => T, available: () => string[]) {
  let held = $state.raw<Selection<T> | null>(null);
  const data = $derived(held?.data ?? incoming());
  const instants = $derived(held?.instants ?? available());
  const index = $derived(held?.index ?? Math.max(0, instants.length - 1));
  const playing = $derived(held?.playing ?? false);

  $effect(() => {
    if (!playing) {
      return;
    }
    const timer = globalThis.setInterval(() => {
      if (!held?.playing) {
        return;
      }
      const next = Math.min(held.index + 1, held.instants.length - 1);
      held = { ...held, index: next, playing: next < held.instants.length - 1 };
    }, PLAYBACK_FRAME_MS);
    return () => globalThis.clearInterval(timer);
  });

  return {
    get data() {
      return data;
    },
    get instants() {
      return instants;
    },
    get index() {
      return index;
    },
    get instant() {
      return instants[index] ?? null;
    },
    get playing() {
      return playing;
    },
    get following() {
      return held === null;
    },
    seek(next: number, commit = false) {
      const selected = Math.max(0, Math.min(instants.length - 1, next));
      held = commit && selected === instants.length - 1 ? null : { data, instants, index: selected, playing: false };
    },
    latest() {
      held = null;
    },
    pause() {
      held = { data, instants, index, playing: false };
    },
    play() {
      held = { data, instants, index, playing: instants.length > 1 && index < instants.length - 1 };
    },
    replay() {
      held = { data, instants, index: 0, playing: instants.length > 1 };
    },
  };
}
