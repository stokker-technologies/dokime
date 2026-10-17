/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Live And Frozen Console Presentation; Reads Arrive From Its Owner.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { ArrowDownToLine, Eraser, X } from "lucide-react";
import { type UIEvent, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useStore } from "zustand";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { recordClockLabel, recordLevelTextClass } from "@/lib/record-display";
import type { HubRecord } from "@/lib/types";
import { cn } from "@/lib/utils";
import { type ConsoleLevel, type ConsoleScope, useUiStore } from "@/stores/ui-store";
import { consoleTailPinned } from "./console-follow";
import { consoleRecordsStore, type ConsoleRecords } from "./console-buffer";

// Local Imports

/* === Re-Exports === */

/* === Constants === */
/** One row height, so the window can be computed instead of measured. */
const ROW_HEIGHT = 22;

/** Rows drawn beyond the viewport on each side, to cover a fast scroll. */
const OVERSCAN = 12;

const LEVEL_ORDER: Record<ConsoleLevel, number> = {
  trace: 0,
  debug: 1,
  info: 2,
  warning: 3,
  error: 4,
};

const LEVEL_LABEL: Record<ConsoleLevel, string> = {
  trace: "Trace",
  debug: "Debug",
  info: "Info",
  warning: "Warning",
  error: "Error",
};

const LEVELS: ConsoleLevel[] = ["trace", "debug", "info", "warning", "error"];

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
}: {
  cycleId: string | null;
  executionId: string | null;
  streamed: ConsoleRecords;
}) {
  const drawer = useUiStore((state) => state.console);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const filter = useStore(consoleRecordsStore, (state) => state.filter);
  const setFilter = useStore(consoleRecordsStore, (state) => state.setFilter);

  const rows = useMemo(() => visibleConsoleRecords(streamed.records, drawer.level, filter), [drawer.level, filter, streamed.records]);

  const close = useCallback(() => setConsoleState({ open: false }), [setConsoleState]);

  useEffect(() => {
    if (!drawer.open) return undefined;
    const leave = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) close();
    };
    window.addEventListener("keydown", leave);
    return () => window.removeEventListener("keydown", leave);
  }, [close, drawer.open]);

  if (!drawer.open) return null;

  return (
    <section
      aria-label="Console"
      className="grid max-h-[80%] min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] border-t bg-card"
      style={{ height: `${drawer.height}px` }}
    >
      <HeightHandle height={drawer.height} onHeightChange={(height) => setConsoleState({ height })} />

      <div className="grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-3 border-b px-[var(--shell-inset)] py-1.5">
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

        <Select onValueChange={(next) => setConsoleState({ level: next as ConsoleLevel })} value={drawer.level}>
          <SelectTrigger className="w-32" size="sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LEVELS.map((level) => (
              <SelectItem key={level} value={level}>
                {LEVEL_LABEL[level]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

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
          {streamed.error ? (
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
        <RecordWindow
          empty="No records at this level yet."
          follow={drawer.follow}
          onFollowChange={(follow) => setConsoleState({ follow })}
          rows={rows}
        />
      </div>
    </section>
  );
}

/**
 * The same rows, frozen: no scope, no follow, no clear, no stream. The level
 * threshold stays because it is a way of reading, not a control over a run.
 * The window is the live one, bounded, so a long attempt scrolls in place
 * instead of pushing the rest of the case past the page.
 */
export function FrozenConsole({ records }: { records: HubRecord[] }) {
  const [level, setLevel] = useState<ConsoleLevel>("info");
  const rows = useMemo(() => visibleConsoleRecords(records, level, ""), [level, records]);

  return (
    <section aria-label="Console" className="grid gap-2">
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3">
        <h3 className="font-heading text-sm leading-6 font-semibold tracking-wide uppercase">Console</h3>
        <Select onValueChange={(next) => setLevel(next as ConsoleLevel)} value={level}>
          <SelectTrigger className="w-32" size="sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LEVELS.map((option) => (
              <SelectItem key={option} value={option}>
                {LEVEL_LABEL[option]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="rounded-md border">
        <RecordWindow className="max-h-96" empty="No records at this level." follow={false} rows={rows} />
      </div>
    </section>
  );
}

/**
 * The windowed list.
 *
 * Every row is one height, so the visible slice is arithmetic rather than
 * measurement, and a cycle with fifty thousand records costs the same as one
 * with fifty. No virtualisation dependency: the repository has none, and one
 * fixed row height does not need one.
 *
 * With `onFollowChange` the list keeps its newest row in view until the
 * operator scrolls up, and offers the way back while they read.
 */
function RecordWindow({
  className,
  empty,
  follow,
  onFollowChange,
  rows,
}: {
  /** Classes for the scrolling viewport, such as a height bound. */
  className?: string;
  empty: string;
  follow: boolean;
  onFollowChange?: (follow: boolean) => void;
  rows: HubRecord[];
}) {
  const viewport = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [height, setHeight] = useState(0);
  const populated = rows.length > 0;
  const tail = rows.at(-1)?.record_id;

  useLayoutEffect(() => {
    if (!populated) return undefined;
    const node = viewport.current;
    if (!node) return undefined;
    const observer = new ResizeObserver(() => setHeight(node.clientHeight));
    observer.observe(node);
    setHeight(node.clientHeight);
    return () => observer.disconnect();
  }, [populated]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: a resize re-pins a following list
  useLayoutEffect(() => {
    const node = viewport.current;
    if (node && follow && tail) node.scrollTop = node.scrollHeight;
  }, [follow, height, tail]);

  const first = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const last = Math.min(rows.length, Math.ceil((scrollTop + height) / ROW_HEIGHT) + OVERSCAN);
  const visible = rows.slice(first, last);

  const onScroll = (event: UIEvent<HTMLDivElement>) => {
    const node = event.currentTarget;
    setScrollTop(node.scrollTop);
    if (!onFollowChange) return;
    const next = consoleTailPinned(node);
    if (next !== follow) onFollowChange(next);
  };

  if (rows.length === 0) {
    return <p className="px-[var(--shell-inset)] py-4 text-sm text-muted-foreground">{empty}</p>;
  }

  return (
    <div className="relative grid min-h-0 grid-rows-[minmax(0,1fr)]">
      <div className={cn("min-h-0 overflow-y-auto [overflow-anchor:none]", className)} onScroll={onScroll} ref={viewport}>
        <div className="relative" style={{ height: `${rows.length * ROW_HEIGHT}px` }}>
          <div className="absolute inset-x-0" style={{ top: `${first * ROW_HEIGHT}px` }}>
            {visible.map((record) => (
              <ConsoleRow key={record.record_id} record={record} />
            ))}
          </div>
        </div>
      </div>
      {onFollowChange && !follow ? (
        <Button
          className="absolute right-[var(--shell-inset)] bottom-2 shadow-sm"
          onClick={() => onFollowChange(true)}
          size="sm"
          variant="secondary"
        >
          <ArrowDownToLine data-icon="inline-start" />
          Jump to Latest
        </Button>
      ) : null}
    </div>
  );
}

/** One record on the four shared columns. */
function ConsoleRow({ record }: { record: HubRecord }) {
  const [expanded, setExpanded] = useState(false);
  const level = record.level;
  const label = typeof record.details.trace_level === "string" ? record.details.trace_level : LEVEL_LABEL[level];

  return (
    <button
      className={cn(
        "grid w-full grid-cols-[5rem_3.75rem_9rem_minmax(0,1fr)] items-start gap-2 px-[var(--shell-inset)] text-left font-mono text-xs tabular-nums hover:bg-muted/40",
        expanded ? "py-1" : "overflow-hidden",
      )}
      onClick={() => setExpanded((current) => !current)}
      style={expanded ? undefined : { height: `${ROW_HEIGHT}px` }}
      type="button"
    >
      <span className="truncate text-muted-foreground">{recordClockLabel(record.timestamp)}</span>
      <span className={cn("truncate uppercase", recordLevelTextClass(level))}>{label}</span>
      <span className="truncate text-muted-foreground">{record.source}</span>
      <span className={cn("min-w-0", expanded ? "break-words whitespace-pre-wrap" : "truncate")}>{record.message ?? ""}</span>
    </button>
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
  if (scope === "execution" && !executionId) return cycleId ? "cycle" : "instance";
  if (scope === "cycle" && !cycleId) return "instance";
  return scope;
}

function scopeDisabled(scope: ConsoleScope, cycleId: string | null, executionId: string | null): boolean {
  if (scope === "instance") return false;
  if (scope === "cycle") return !cycleId;
  return !executionId;
}

/* === Hooks === */

/* === Public Functions === */
export function visibleConsoleRecords(records: HubRecord[], level: ConsoleLevel, filter: string): HubRecord[] {
  const threshold = LEVEL_ORDER[level];
  const needle = filter.trim().toLowerCase();
  return records.filter((record) => {
    if (LEVEL_ORDER[record.level] < threshold) return false;
    if (!needle) return true;
    return `${record.source} ${record.message ?? ""}`.toLowerCase().includes(needle);
  });
}

/** Warnings and errors that arrived after the drawer was last opened. */
export function consoleAttentionCount(records: HubRecord[], since: number): number {
  return records.filter((record) => {
    if (record.level !== "warning" && record.level !== "error") return false;
    const at = Date.parse(record.timestamp);
    return !Number.isFinite(at) || at > since;
  }).length;
}

/* === Private Functions === */
