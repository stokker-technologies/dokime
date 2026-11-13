/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Selection<T> = { data: T; instants: string[]; index: number; playing: boolean };

/* === Public Functions === */

export function instantTime(instant: string): bigint | null {
  const millis = Date.parse(instant);
  if (!Number.isFinite(millis)) return null;
  const fraction = instant.match(/\.(\d+)(?:Z|[+-]\d\d:\d\d)$/)?.[1] ?? "";
  return BigInt(millis) * 1_000_000n + BigInt(fraction.padEnd(9, "0").slice(3, 9) || "0");
}

export function orderedInstants(values: Iterable<string>): string[] {
  const times = new Map<bigint, string>();
  for (const value of values) {
    const time = instantTime(value);
    if (time !== null && !times.has(time)) times.set(time, value);
  }
  return [...times].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([, value]) => value);
}

/* === Hooks === */

export function usePlotPlayback<T>(incoming: T, available: string[]) {
  // A Held Payload Is The Loaded Playback Boundary. Normal Live Delivery Continues Independently.
  const [held, setHeld] = useState<Selection<T> | null>(null);
  const data = held?.data ?? incoming;
  const instants = held?.instants ?? available;
  const index = held?.index ?? Math.max(0, instants.length - 1);
  const playing = held?.playing ?? false;
  useEffect(() => {
    if (!playing) return;
    const timer = globalThis.setInterval(() => {
      setHeld((selection) => {
        if (!selection?.playing) return selection;
        const next = Math.min(selection.index + 1, selection.instants.length - 1);
        return { ...selection, index: next, playing: next < selection.instants.length - 1 };
      });
    }, 200);
    return () => globalThis.clearInterval(timer);
  }, [playing]);
  const seek = (next: number, commit = false) => {
    const selected = Math.max(0, Math.min(instants.length - 1, next));
    setHeld(commit && selected === instants.length - 1 ? null : { data, instants, index: selected, playing: false });
  };
  return {
    data,
    instants,
    index,
    instant: instants[index] ?? null,
    playing,
    following: held === null,
    seek,
    latest: () => setHeld(null),
    pause: () => setHeld({ data, instants, index, playing: false }),
    play: () => setHeld({ data, instants, index, playing: instants.length > 1 && index < instants.length - 1 }),
    replay: () => setHeld({ data, instants, index: 0, playing: instants.length > 1 }),
  };
}

/* === Components === */

export function PlotPlayback({ title, playback }: { title: string; playback: Omit<ReturnType<typeof usePlotPlayback>, "data"> }) {
  const { index, instants, instant, playing, following } = playback;
  const disabled = instants.length < 2;
  const interacting = useRef(false);
  const value = useMemo(() => [index], [index]);
  return (
    <fieldset className="grid min-w-0 gap-2" aria-label={`${title} Playback`} data-playback-instant={instant ?? ""}>
      <Slider
        aria-label={`${title} Instant`}
        aria-valuetext={instant ?? "No Instants"}
        min={0}
        max={Math.max(1, instants.length - 1)}
        step={1}
        value={value}
        disabled={disabled}
        onKeyDown={() => {
          interacting.current = true;
        }}
        onPointerDown={() => {
          interacting.current = true;
        }}
        onBlur={() => {
          interacting.current = false;
        }}
        onValueChange={([next]) => {
          if (interacting.current) playback.seek(next);
        }}
        onValueCommit={([next]) => {
          interacting.current = false;
          playback.seek(next, true);
        }}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="outline" size="xs" disabled={disabled || playing} onClick={playback.play}>
          Play
        </Button>
        <Button variant="outline" size="xs" disabled={!playing} onClick={playback.pause}>
          Pause
        </Button>
        <Button variant="outline" size="xs" disabled={disabled} onClick={playback.replay}>
          Replay
        </Button>
        <Button variant="outline" size="xs" disabled={!instants.length || following} onClick={playback.latest}>
          Latest
        </Button>
        <output className="min-w-0 truncate text-xs tabular-nums text-muted-foreground">{instant ?? "No Instants"}</output>
      </div>
    </fieldset>
  );
}

/* === Private Functions === */
