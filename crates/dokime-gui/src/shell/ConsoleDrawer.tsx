/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The console: what the backend said, in the order it said it.
 *
 * A drawer above the status bar, spanning the main column, so the evidence
 * keeps its shape while the operator reads the tail. The rows are records —
 * the same projection the evidence pane renders as cards — shown as the dense
 * mono grid a log wants: time, level, source, message.
 *
 * With `live={false}` the same component is the report's record table: the
 * level filter survives, because a threshold is a way of reading a file, and
 * everything that would suggest a running backend does not.
 */

/* === External Imports === */

import { Eraser, X } from "lucide-react";
import {
  type UIEvent,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { recordClockLabel, recordLevelVariant } from "@/lib/record-display";
import type { HubRecord } from "@/lib/types";
import { cn } from "@/lib/utils";
import { type ConsoleLevel, type ConsoleScope, useUiStore } from "@/stores/ui-store";
import { useConsoleRecords } from "./console-records";

/* === Types === */

export type ConsoleDrawerProps =
  | {
      /** The cycle the route carries, or null away from a cycle. */
      cycleId: string | null;
      /** The attempt the operator selected, or null. */
      executionId: string | null;
      live?: true;
    }
  | {
      live: false;
      /** The attempt's records, already frozen into the file. */
      records: HubRecord[];
    };

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

/* === Components === */

export function ConsoleDrawer(props: ConsoleDrawerProps) {
  if (props.live === false) {
    return <FrozenConsole records={props.records} />;
  }
  return <LiveConsole cycleId={props.cycleId} executionId={props.executionId} />;
}

/* === Public Functions === */

/** Records at or above `level`, then matching `filter`, oldest first. */
export function visibleConsoleRecords(
  records: HubRecord[],
  level: ConsoleLevel,
  filter: string,
): HubRecord[] {
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

/* === Private Components === */

/**
 * The drawer proper: a toolbar on one grid, then the windowed rows.
 *
 * Only the rows scroll, so the toolbar stays reachable while a running cycle
 * pushes hundreds of records past it.
 */
function LiveConsole({
  cycleId,
  executionId,
}: {
  cycleId: string | null;
  executionId: string | null;
}) {
  const drawer = useUiStore((state) => state.console);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const [filter, setFilter] = useState("");

  const scope = resolvedScope(drawer.scope, cycleId, executionId);
  const streamed = useConsoleRecords({
    cycleId: scope === "instance" ? null : cycleId,
    enabled: drawer.open,
    executionId: scope === "execution" ? executionId : null,
    level: drawer.level,
  });
  const [clearedThrough, setClearedThrough] = useState<string | null>(null);
  const records = useMemo(() => {
    if (!clearedThrough) return streamed.records;
    const sequence = BigInt(clearedThrough.slice("trace-".length));
    return streamed.records.filter(
      (record) => BigInt(record.record_id.slice("trace-".length)) > sequence,
    );
  }, [clearedThrough, streamed.records]);

  useEffect(() => {
    if (streamed.records.length === 0) setClearedThrough(null);
  }, [streamed.records]);
  const rows = useMemo(
    () => visibleConsoleRecords(records, drawer.level, filter),
    [drawer.level, filter, records],
  );

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
      className="grid shrink-0 grid-rows-[auto_auto_minmax(0,1fr)] border-t bg-card"
      style={{ height: `${drawer.height}px` }}
    >
      <HeightHandle
        height={drawer.height}
        onHeightChange={(height) => setConsoleState({ height })}
      />

      <div className="grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto_auto] items-center gap-3 border-b px-[var(--shell-inset)] py-1.5">
        <ToggleGroup
          onValueChange={(next) => next && setConsoleState({ scope: next as ConsoleScope })}
          type="single"
          value={drawer.scope}
          variant="outline"
        >
          {(["instance", "cycle", "execution"] as ConsoleScope[]).map((option) => (
            <ToggleGroupItem
              disabled={scopeDisabled(option, cycleId, executionId)}
              key={option}
              value={option}
            >
              {SCOPE_LABEL[option]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>

        <Select
          onValueChange={(next) => setConsoleState({ level: next as ConsoleLevel })}
          value={drawer.level}
        >
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

        <span className="flex items-center gap-2 text-xs text-muted-foreground">
          <Switch
            aria-label="Follow"
            checked={drawer.follow}
            onCheckedChange={(follow) => setConsoleState({ follow })}
          />
          Follow
        </span>

        <Button
          onClick={() => setClearedThrough(streamed.records.at(-1)?.record_id ?? null)}
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
              {streamed.dropped.toString()} older {streamed.dropped === 1n ? "event" : "events"}{" "}
              dropped
            </p>
          ) : null}
        </div>
        <RecordWindow follow={drawer.follow} rows={rows} />
      </div>
    </section>
  );
}

/**
 * The same rows, frozen: no scope, no follow, no clear, no stream. The level
 * threshold stays because it is a way of reading, not a control over a run.
 */
function FrozenConsole({ records }: { records: HubRecord[] }) {
  const [level, setLevel] = useState<ConsoleLevel>("info");
  const rows = useMemo(() => visibleConsoleRecords(records, level, ""), [level, records]);

  return (
    <section aria-label="Console" className="grid gap-2">
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3">
        <h3 className="font-heading text-sm leading-6 font-semibold tracking-wide uppercase">
          Console
        </h3>
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
      {rows.length === 0 ? (
        <p className="text-sm text-muted-foreground">No records at this level.</p>
      ) : (
        <div className="rounded-md border">
          {rows.map((record) => (
            <ConsoleRow key={record.record_id} record={record} />
          ))}
        </div>
      )}
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
 */
function RecordWindow({ follow, rows }: { follow: boolean; rows: HubRecord[] }) {
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

  useLayoutEffect(() => {
    const node = viewport.current;
    if (node && follow && tail) node.scrollTop = node.scrollHeight;
  }, [follow, tail]);

  const first = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const last = Math.min(rows.length, Math.ceil((scrollTop + height) / ROW_HEIGHT) + OVERSCAN);
  const visible = rows.slice(first, last);

  const onScroll = (event: UIEvent<HTMLDivElement>) => setScrollTop(event.currentTarget.scrollTop);

  if (rows.length === 0) {
    return (
      <p className="px-[var(--shell-inset)] py-4 text-sm text-muted-foreground">
        No records at this level yet.
      </p>
    );
  }

  return (
    <div
      className="min-h-0 overflow-y-auto [overflow-anchor:none]"
      onScroll={onScroll}
      ref={viewport}
    >
      <div className="relative" style={{ height: `${rows.length * ROW_HEIGHT}px` }}>
        <div className="absolute inset-x-0" style={{ top: `${first * ROW_HEIGHT}px` }}>
          {visible.map((record) => (
            <ConsoleRow key={record.record_id} record={record} />
          ))}
        </div>
      </div>
    </div>
  );
}

/** One record on the four shared columns. */
function ConsoleRow({ record }: { record: HubRecord }) {
  const [expanded, setExpanded] = useState(false);
  const level = record.level;
  const label =
    typeof record.details.trace_level === "string"
      ? record.details.trace_level
      : LEVEL_LABEL[level];

  return (
    <button
      className={cn(
        "grid w-full grid-cols-[6rem_5rem_10rem_minmax(0,1fr)] items-start gap-3 px-[var(--shell-inset)] text-left font-mono text-xs tabular-nums hover:bg-muted/40",
        expanded ? "py-1" : "overflow-hidden",
      )}
      onClick={() => setExpanded((current) => !current)}
      style={expanded ? undefined : { height: `${ROW_HEIGHT}px` }}
      type="button"
    >
      <span className="truncate text-muted-foreground">{recordClockLabel(record.timestamp)}</span>
      <Badge
        className="h-4 justify-self-start px-1 text-[0.65rem]"
        variant={recordLevelVariant(level)}
      >
        {label}
      </Badge>
      <span className="truncate text-muted-foreground">{record.source}</span>
      <span className={cn("min-w-0", expanded ? "break-words whitespace-pre-wrap" : "truncate")}>
        {record.message ?? ""}
      </span>
    </button>
  );
}

/** The drag handle that gives the drawer its height. */
function HeightHandle({
  height,
  onHeightChange,
}: {
  height: number;
  onHeightChange: (height: number) => void;
}) {
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
        const move = (next: PointerEvent) =>
          onHeightChange(clampHeight(start + (origin - next.clientY)));
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
function resolvedScope(
  scope: ConsoleScope,
  cycleId: string | null,
  executionId: string | null,
): ConsoleScope {
  if (scope === "execution" && !executionId) return cycleId ? "cycle" : "instance";
  if (scope === "cycle" && !cycleId) return "instance";
  return scope;
}

function scopeDisabled(
  scope: ConsoleScope,
  cycleId: string | null,
  executionId: string | null,
): boolean {
  if (scope === "instance") return false;
  if (scope === "cycle") return !cycleId;
  return !executionId;
}
