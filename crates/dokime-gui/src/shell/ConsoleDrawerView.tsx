/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Live Console Presentation; The Desktop Shell Owns Reads.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { ErrorCard } from "@/components/ErrorCard";
import { Eraser, X } from "lucide-react";
import { useCallback, useEffect, useMemo } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import SelectField from "../components/ui/SelectField.svelte";
import { ToggleGroup, ToggleGroupItem } from "../components/ui/toggle-group";
import type { HubRecord } from "@/lib/types";
import type { ConsoleScope, ConsoleState } from "../stores/ui-store";
import type { ConsoleRecords } from "./console-buffer";
import { SvelteMount } from "../components/SvelteMount";
import ConsoleRecordList from "./ConsoleRecordList.svelte";
import { type ConsoleLevel, consoleLevelOptions, visibleConsoleRecords } from "./console-view-model";

/* === Re-Exports === */

/* === Constants === */

const SCOPE_LABEL: Record<ConsoleScope, string> = {
  cycle: "Cycle",
  execution: "Execution",
  instance: "Instance",
};

const MIN_HEIGHT = 120;

const MAX_HEIGHT = 900;

/* === Types === */

/* === Components === */
export function LiveConsoleView({
  cycleId,
  executionId,
  streamed,
  drawer,
  setConsoleState,
  filter,
  setFilter,
}: {
  cycleId: string | null;
  executionId: string | null;
  streamed: ConsoleRecords;
  drawer: ConsoleState;
  setConsoleState: (patch: Partial<ConsoleState>) => void;
  filter: string;
  setFilter: (filter: string) => void;
}) {
  const rows = useMemo(() => visibleConsoleRecords(streamed.records, drawer.level, filter), [drawer.level, filter, streamed.records]);

  const close = useCallback(() => setConsoleState({ open: false }), [setConsoleState]);

  useEffect(() => {
    if (!drawer.open) {
      return undefined;
    }
    const leave = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) {
        event.preventDefault();
        close();
      }
    };
    document.addEventListener("keydown", leave);
    return () => document.removeEventListener("keydown", leave);
  }, [close, drawer.open]);

  if (!drawer.open) {
    return null;
  }

  return (
    <section
      aria-label="Console"
      className="grid max-h-[80%] min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] border-t bg-card"
      style={{ height: `${drawer.height}px` }}
    >
      <HeightHandle height={drawer.height} onHeightChange={(height) => setConsoleState({ height })} />

      <div className="grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-3 border-b px-[var(--shell-inset)] py-2">
        <ToggleGroup
          onValueChange={(next) => next && setConsoleState({ scope: next as ConsoleScope })}
          type="single"
          value={drawer.scope}
          variant="outline"
        >
          {(["instance", "cycle", "execution"] as ConsoleScope[]).map((option) => (
            <ToggleGroupItem disabled={scopeDisabled(option, cycleId, executionId)} key={option} value={option}>
              {SCOPE_LABEL[option]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>

        <SvelteMount
          component={SelectField<ConsoleLevel>}
          props={{
            label: "Console Level",
            class: "w-32",
            size: "sm",
            value: drawer.level,
            options: consoleLevelOptions,
            onChange: (level) => setConsoleState({ level }),
          }}
        />

        <Input
          aria-label="Filter Records"
          className="h-8 min-w-0"
          onChange={(event) => setFilter(event.target.value)}
          placeholder="Filter Records"
          value={filter}
        />

        <Button
          onClick={streamed.clear}
          size="sm"
          title="Clear the rows shown here; the backend retains its trace history."
          variant="outline"
        >
          <Eraser data-icon="inline-start" />
          Clear
        </Button>

        <Button aria-label="Close Console" onClick={close} size="icon-sm" variant="ghost">
          <X />
        </Button>
      </div>

      <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
        <div>
          {streamed.failure ? (
            <ErrorCard failure={streamed.failure} />
          ) : streamed.error ? (
            <p role="status" className="px-[var(--shell-inset)] py-1 text-xs text-muted-foreground">
              {streamed.error}
            </p>
          ) : null}
          {streamed.dropped > 0n ? (
            <p className="px-[var(--shell-inset)] py-1 font-mono text-xs text-muted-foreground">
              {streamed.dropped.toString()} older {streamed.dropped === 1n ? "event" : "events"} dropped
            </p>
          ) : null}
        </div>
        <SvelteMount
          component={ConsoleRecordList}
          props={{
            empty: "No records at this level yet.",
            follow: drawer.follow,
            onFollowChange: (follow: boolean) => setConsoleState({ follow }),
            rows,
          }}
        />
      </div>
    </section>
  );
}

/** The drag handle that gives the drawer its height. */
function HeightHandle({ height, onHeightChange }: { height: number; onHeightChange: (height: number) => void }) {
  return (
    <hr
      aria-label="Console Height"
      aria-orientation="horizontal"
      aria-valuenow={height}
      className="h-1 cursor-row-resize border-0 bg-transparent hover:bg-border"
      onKeyDown={(event) => {
        if (event.key === "ArrowUp") onHeightChange(clampHeight(height + 24));
        if (event.key === "ArrowDown") onHeightChange(clampHeight(height - 24));
      }}
      onPointerDown={(event) => {
        const origin = event.clientY;
        const start = height;
        const move = (next: PointerEvent) => onHeightChange(clampHeight(start + (origin - next.clientY)));
        const stop = () => {
          window.removeEventListener("pointermove", move);
          window.removeEventListener("pointerup", stop);
        };
        window.addEventListener("pointermove", move);
        window.addEventListener("pointerup", stop);
      }}
      tabIndex={0}
    />
  );
}

/* === Private Functions === */

function clampHeight(value: number): number {
  return Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, Math.round(value)));
}

/** A scope the route cannot fill falls back to the widest one it can. */
export function resolvedConsoleScope(scope: ConsoleScope, cycleId: string | null, executionId: string | null): ConsoleScope {
  if (scope === "execution" && !executionId) {
    return cycleId ? "cycle" : "instance";
  }
  if (scope === "cycle" && !cycleId) {
    return "instance";
  }
  return scope;
}

function scopeDisabled(scope: ConsoleScope, cycleId: string | null, executionId: string | null): boolean {
  if (scope === "instance") {
    return false;
  }
  if (scope === "cycle") {
    return !cycleId;
  }
  return !executionId;
}

/* === Hooks === */

/* === Public Functions === */
/** Warnings and errors that arrived after the drawer was last opened. */
export function consoleAttentionCount(records: HubRecord[], since: number): number {
  return records.filter((record) => {
    if (record.level !== "warning" && record.level !== "error") {
      return false;
    }
    const at = Date.parse(record.timestamp);
    return !Number.isFinite(at) || at > since;
  }).length;
}

/* === Private Functions === */
