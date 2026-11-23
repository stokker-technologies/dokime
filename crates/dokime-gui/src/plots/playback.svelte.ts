// Dokime — SPDX-License-Identifier: Apache-2.0
import { instantTime } from "./instants";

/** The Wall Clock and the Sample Time Playback Counts From. */
type Clock = { wall: number; time: bigint };
type Selection<T> = { data: T; instants: string[]; index: number; clock: Clock | null };

/** How Often Playback Moves to the Instant Its Clock Has Reached. */
export const PLAYBACK_TICK_MS = 100;

/** Sample Time Played per Unit of Wall Time. */
export const PLAYBACK_SPEEDS = [1, 2, 5, 10, 60, 600] as const;

function started(time: bigint): Clock {
  return { wall: performance.now(), time };
}

/** The Sample Time a Clock Has Reached, in Nanoseconds. */
function reached(clock: Clock, speed: number) {
  return clock.time + BigInt(Math.round((performance.now() - clock.wall) * speed * 1_000_000));
}

/** The Last of the Ascending Times at or Before the Target. */
function lastAtOrBefore(times: bigint[], target: bigint) {
  let low = 0;
  let high = times.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (times[middle] <= target) {
      low = middle + 1;
    } else {
      high = middle;
    }
  }
  return Math.max(0, low - 1);
}

/** Plays Instants in Source-Time Order, as `orderedInstants` Returns Them, in Sample Time at the Chosen Speed. */
export function plotPlayback<T>(incoming: () => T, available: () => string[]) {
  let held = $state.raw<Selection<T> | null>(null);
  let speed = $state<number>(PLAYBACK_SPEEDS[0]);
  const data = $derived(held?.data ?? incoming());
  const instants = $derived(held?.instants ?? available());
  const times = $derived(instants.map((instant) => instantTime(instant) ?? 0n));
  const index = $derived(held?.index ?? Math.max(0, instants.length - 1));
  const playing = $derived(Boolean(held?.clock));

  $effect(() => {
    if (!playing) {
      return;
    }
    const timer = globalThis.setInterval(() => {
      if (!held?.clock) {
        return;
      }
      const next = lastAtOrBefore(times, reached(held.clock, speed));
      if (next !== held.index) {
        held = { ...held, index: next, clock: next < held.instants.length - 1 ? held.clock : null };
      }
    }, PLAYBACK_TICK_MS);
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
    get speed() {
      return speed;
    },
    set speed(next: number) {
      if (held?.clock) {
        held = { ...held, clock: started(reached(held.clock, speed)) };
      }
      speed = next;
    },
    seek(next: number, commit = false) {
      const selected = Math.max(0, Math.min(instants.length - 1, next));
      held = commit && selected === instants.length - 1 ? null : { data, instants, index: selected, clock: null };
    },
    latest() {
      held = null;
    },
    pause() {
      held = { data, instants, index, clock: null };
    },
    play() {
      held = { data, instants, index, clock: index < instants.length - 1 ? started(times[index]) : null };
    },
    replay() {
      held = { data, instants, index: 0, clock: instants.length > 1 ? started(times[0]) : null };
    },
  };
}
