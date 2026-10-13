/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The expanded inner view of one record row.
 *
 * A reader opens a record to learn three things: what happened, how long ago,
 * and what it carried. The panel answers them in that order on the row's own
 * column tracks, so the age sits under the row's clock, the source under the
 * row's source, and the payload labels share one fixed column. Scope and
 * sequence are provenance, not content: one muted line at the foot, with the
 * full record one click further in the details dialog.
 *
 * Live and the sealed report share this panel. Live, the age ticks while the
 * panel is open; in a report an "ago" figure would depend on when the file is
 * opened, so the absolute time leads instead.
 */

/* === External Imports === */

import { Check, Clipboard } from "lucide-react";
import { useEffect, useState } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { recordMessage, recordTitle } from "@/lib/dashboard";
import {
  formatRecordValue,
  isDatetimeValue,
  recordLevelLabel,
  recordLevelVariant,
} from "@/lib/record-display";
import { absoluteTimestampLabel, parseTimestampMs, relativeAgeLabel } from "@/lib/time-format";
import type { HubRecord } from "@/lib/types";
import { cn, isRecord, numericValue } from "@/lib/utils";

/* === Types === */

type RecordRowPanelProps = {
  /** False in the frozen report: the age does not tick and the absolute time leads. */
  live: boolean;
  onInspect: () => void;
  record: HubRecord;
};

type ScopeEntry = {
  label: string;
  value: string | null | undefined;
};

/* === Constants === */

/** The record row's column tracks — time, level, source, message, type — shared by its panel. */
export const RECORD_ROW_COLUMNS = "grid grid-cols-[6rem_5.5rem_10rem_minmax(0,1fr)_auto] gap-x-3";

/** Characters a long text value shows before it offers to expand. */
const LONG_TEXT_LIMIT = 160;

const AGE_TICK_MS = 1000;

/* === Components === */

export function RecordRowPanel({ live, onInspect, record }: RecordRowPanelProps) {
  const now = useLiveNow(live);
  const absolute = absoluteTimestampLabel(record.timestamp);
  const title = recordMessage(record) || recordTitle(record);
  const payload = Object.entries(record.details);

  return (
    <div
      className={cn(
        RECORD_ROW_COLUMNS,
        "items-baseline gap-y-2 border-t border-border/60 bg-muted/10 px-3 py-2.5 text-xs",
      )}
      data-testid="record-row-panel"
    >
      {/* What happened, and when. */}
      <div className="col-span-2 grid min-w-0 gap-0.5">
        <time
          className="truncate font-mono text-sm tabular-nums text-foreground"
          dateTime={record.timestamp}
          title={absolute}
        >
          {live ? relativeAgeLabel(record.timestamp, now) : absolute}
        </time>
        {live ? <span className="truncate text-muted-foreground">{absolute}</span> : null}
      </div>
      <div className="col-span-2 grid min-w-0 gap-0.5">
        <code className="truncate font-mono text-muted-foreground" title={record.record_type}>
          {record.record_type}
        </code>
        <span className="text-sm font-medium [overflow-wrap:anywhere]">{title}</span>
      </div>
      <div className="grid justify-items-end gap-0.5">
        <Badge className="h-4 px-1 text-[0.65rem]" variant={recordLevelVariant(record.level)}>
          {recordLevelLabel(record.level)}
        </Badge>
        <button
          className="whitespace-nowrap font-medium text-primary hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          onClick={onInspect}
          type="button"
        >
          Inspect full record details
        </button>
      </div>

      {/* What it carried. */}
      {payload.length ? (
        <dl className="col-span-full grid grid-cols-subgrid items-baseline gap-y-1 border-t border-border/60 pt-2">
          {payload.map(([key, value]) => (
            <PayloadEntry key={key} name={key} value={value} />
          ))}
        </dl>
      ) : (
        <p className="col-span-full border-t border-border/60 pt-2 text-muted-foreground">
          This record carries no payload.
        </p>
      )}

      {/* Where it came from. */}
      <span className="col-span-2 truncate font-mono tabular-nums text-muted-foreground">
        {record.sequence !== null && record.sequence !== undefined ? `seq ${record.sequence}` : ""}
      </span>
      <span className="truncate text-muted-foreground" title={record.source}>
        {record.source}
      </span>
      <ScopeLine record={record} />
    </div>
  );
}

function PayloadEntry({ name, value }: { name: string; value: unknown }) {
  return (
    <>
      <dt className="col-span-2 truncate font-mono text-muted-foreground" title={name}>
        {name}
      </dt>
      <dd className="col-span-3 min-w-0">
        <PayloadValue value={value} />
      </dd>
    </>
  );
}

/** One payload value: a leaf, or one level of a list or map whose own children print as leaves. */
function PayloadValue({ value }: { value: unknown }) {
  if (Array.isArray(value)) {
    if (!value.length) {
      return <span className="text-muted-foreground">Empty list</span>;
    }
    if (value.every(isLeaf)) {
      const items = value.map(formatLeaf);
      return (
        <LeafValue
          mono={items.every((item) => item.mono)}
          value={items.map((item) => item.text).join(", ")}
        />
      );
    }
    return <NestedGrid entries={value.map((item, index) => [String(index), item] as const)} />;
  }
  if (isNested(value)) {
    const entries = Object.entries(value);
    if (!entries.length) {
      return <span className="text-muted-foreground">Empty map</span>;
    }
    return <NestedGrid entries={entries} />;
  }
  return <LeafValue value={value} />;
}

function NestedGrid({ entries }: { entries: ReadonlyArray<readonly [string, unknown]> }) {
  return (
    <dl className="grid grid-cols-[minmax(0,8rem)_minmax(0,1fr)] items-baseline gap-x-3 gap-y-0.5 border-l border-border/60 pl-3">
      {entries.map(([key, nested]) => (
        <NestedEntry key={key} name={key} value={nested} />
      ))}
    </dl>
  );
}

function NestedEntry({ name, value }: { name: string; value: unknown }) {
  return (
    <>
      <dt className="truncate font-mono text-muted-foreground" title={name}>
        {name}
      </dt>
      <dd className="min-w-0">
        <LeafValue value={value} />
      </dd>
    </>
  );
}

/** A single printed value; anything deeper than one level prints as compact JSON. */
function LeafValue({ mono: monoOverride, value }: { mono?: boolean; value: unknown }) {
  const [expanded, setExpanded] = useState(false);
  const formatted = isLeaf(value)
    ? formatLeaf(value)
    : { mono: true, text: compactJson(value), title: undefined };
  const { text, title } = formatted;
  const mono = monoOverride ?? formatted.mono;
  const long = text.length > LONG_TEXT_LIMIT || text.includes("\n");
  const shown = long && !expanded ? `${firstLine(text).slice(0, LONG_TEXT_LIMIT)}…` : text;

  return (
    <span className="min-w-0">
      <span
        className={cn(
          "[overflow-wrap:anywhere]",
          mono ? "font-mono tabular-nums" : "text-sm",
          expanded && "whitespace-pre-wrap",
        )}
        title={title}
      >
        {shown}
      </span>
      {long ? (
        <button
          className="ml-2 font-medium text-primary hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          onClick={() => setExpanded((current) => !current)}
          type="button"
        >
          {expanded ? "Show less" : "Show all"}
        </button>
      ) : null}
    </span>
  );
}

/** Cycle, execution and case on fixed tracks, so the same scope reads in the same place on every row. */
function ScopeLine({ record }: { record: HubRecord }) {
  const scope = record.scope;
  const entries: ScopeEntry[] = [
    { label: "cycle", value: scope?.test_cycle_id },
    { label: "execution", value: scope?.test_execution_id },
    { label: "case", value: scope?.test_case_id },
  ];
  if (!entries.some((entry) => entry.value)) {
    return <span className="col-span-2 text-muted-foreground">Runtime scope</span>;
  }
  return (
    <span className="col-span-2 grid min-w-0 grid-cols-3 items-baseline gap-x-3">
      {entries.map((entry) =>
        entry.value ? (
          <ScopeId key={entry.label} label={entry.label} value={entry.value} />
        ) : (
          <span key={entry.label} />
        ),
      )}
    </span>
  );
}

function ScopeId({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const handle = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(handle);
  }, [copied]);

  async function copyValue() {
    if (!navigator.clipboard?.writeText) {
      return;
    }
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  }

  return (
    <span className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-baseline gap-x-1.5 text-muted-foreground">
      <span>{label}</span>
      <span className="truncate font-mono text-foreground" title={value}>
        {shortIdentifier(value)}
      </span>
      <button
        aria-label={`Copy ${label} id ${value}`}
        className="grid size-4 place-items-center self-center rounded text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onClick={() => void copyValue()}
        title={copied ? "Copied" : `Copy ${value}`}
        type="button"
      >
        {copied ? <Check size={11} /> : <Clipboard size={11} />}
      </button>
    </span>
  );
}

/* === Private Functions === */

/** Wall-clock milliseconds that tick once a second while `live`, and stand still otherwise. */
function useLiveNow(live: boolean): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!live) {
      return;
    }
    setNow(Date.now());
    const handle = window.setInterval(() => setNow(Date.now()), AGE_TICK_MS);
    return () => window.clearInterval(handle);
  }, [live]);
  return now;
}

/**
 * Whether `value` prints as one leaf. The wire wraps a datetime, a wide
 * integer, a non-finite number and a serialized duration in small maps; each
 * of those is one value, not a map to open.
 */
function isLeaf(value: unknown): boolean {
  return !Array.isArray(value) && !isNested(value);
}

function isNested(value: unknown): value is Record<string, unknown> {
  return (
    isRecord(value) &&
    !isDatetimeValue(value) &&
    numericValue(value) === null &&
    durationSeconds(value) === null
  );
}

function formatLeaf(value: unknown): { mono: boolean; text: string; title?: string } {
  if (isDatetimeValue(value)) {
    return { mono: true, text: absoluteTimestampLabel(value.$datetime), title: value.$datetime };
  }
  const duration = durationSeconds(value);
  if (duration !== null) {
    return { mono: true, text: durationText(duration), title: `${duration} s` };
  }
  if (typeof value === "string") {
    if (isTimestampString(value)) {
      return { mono: true, text: absoluteTimestampLabel(value), title: value };
    }
    return { mono: isIdentifierLike(value), text: value || "“”" };
  }
  if (value === null || value === undefined) {
    return { mono: false, text: "—" };
  }
  return { mono: true, text: formatRecordValue(value) };
}

/** Seconds in a Rust `Duration` as serde writes it, `{ secs, nanos }`, or null for anything else. */
function durationSeconds(value: unknown): number | null {
  if (!isRecord(value)) {
    return null;
  }
  const keys = Object.keys(value);
  if (keys.length !== 2 || typeof value.secs !== "number" || typeof value.nanos !== "number") {
    return null;
  }
  return value.secs + value.nanos / 1e9;
}

function durationText(seconds: number): string {
  if (seconds < 1) {
    return `${(seconds * 1000).toFixed(seconds < 0.01 ? 3 : 1)} ms`;
  }
  if (seconds < 60) {
    return `${seconds.toFixed(seconds < 10 ? 3 : 1)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} min ${Math.floor(seconds % 60)} s`;
  }
  return `${Math.floor(minutes / 60)} h ${minutes % 60} min`;
}

function isTimestampString(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/.test(value) && parseTimestampMs(value) !== null;
}

/** A token with no spaces — an id, a path, a version — reads best in mono. */
function isIdentifierLike(value: string): boolean {
  return value.length > 0 && value.length <= 96 && /^[\w.:/@#+-]+$/.test(value);
}

/** A UUID or long hex id shortens to its first eight characters; any other id stays whole. */
function shortIdentifier(value: string): string {
  return /^[0-9a-f]{8}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{12}$/i.test(value) ||
    /^[0-9a-f]{16,}$/i.test(value)
    ? value.slice(0, 8)
    : value;
}

function compactJson(value: unknown): string {
  try {
    return JSON.stringify(value) ?? "—";
  } catch {
    return String(value);
  }
}

function firstLine(text: string): string {
  const newline = text.indexOf("\n");
  return newline === -1 ? text : text.slice(0, newline);
}
