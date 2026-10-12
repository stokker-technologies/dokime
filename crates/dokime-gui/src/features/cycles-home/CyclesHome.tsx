/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { LIST_STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import {
  applyFrozenOrder,
  type CycleGroup,
  type CycleRow,
  cycleRows,
  groupCycleRows,
} from "@/features/cycles-home/cycles-model";
import { useCyclesQuery, usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { compactAgeLabel } from "@/lib/time-format";
import { cn } from "@/lib/utils";
import { usePageChrome } from "@/shell/page-chrome";

/* === Constants === */

/** One fixed pill height and measure per density keeps the status column on one
    baseline and one width, whatever a status is called. */
const PILL_CLASS = LIST_STATUS_PILL_CLASS;

/** One grid for the column header and every row beneath it. */
const ROW_GRID =
  "grid items-center gap-[var(--shell-gap)] grid-cols-[minmax(0,1fr)_6rem_7rem_7rem_6rem]";

/* === Components === */

/**
 * Home. Three fixed groups over every loaded cycle, a filter box, and one way
 * into a cycle: its row.
 */
export function CyclesHome() {
  const navigate = useNavigate();
  const [filter, setFilter] = useState("");
  const [showCleared, setShowCleared] = useState(false);
  /* The box types faster than a round trip; the request follows the operator
     rather than every keystroke, and the box itself never lags. */
  const [selection, setSelection] = useState("");
  const cycles = useCyclesQuery({ includeCleared: showCleared, text: selection });
  const prompts = usePendingPromptsQuery();
  const [frozenIds, setFrozenIds] = useState<readonly string[]>([]);
  const frozen = useRef(false);
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const handle = window.setInterval(() => setNow(Date.now()), 15_000);
    return () => window.clearInterval(handle);
  }, []);

  useEffect(() => {
    const handle = window.setTimeout(() => setSelection(filter), 250);
    return () => window.clearTimeout(handle);
  }, [filter]);

  const actions = useMemo(
    () => (
      <Button
        className="h-[var(--control-height)]"
        onClick={() => void navigate({ to: "/library" })}
        type="button"
      >
        New Cycle
      </Button>
    ),
    [navigate],
  );
  usePageChrome({ actions, title: "Cycles" });

  const attentionByCycleId = useMemo(() => {
    const counts = new Map<string, number>();
    for (const prompt of prompts.data ?? []) {
      const cycleId = prompt.scope?.test_cycle_id;
      if (cycleId) {
        counts.set(cycleId, (counts.get(cycleId) ?? 0) + 1);
      }
    }
    return counts;
  }, [prompts.data]);

  const rows = useMemo(
    () => cycleRows(cycles.data ?? [], attentionByCycleId),
    [attentionByCycleId, cycles.data],
  );
  const groups = useMemo(() => {
    const grouped = groupCycleRows(rows, selection.trim().length > 0);
    return frozenIds.length
      ? grouped.map((group) => ({ ...group, rows: applyFrozenOrder(group.rows, frozenIds) }))
      : grouped;
  }, [frozenIds, rows, selection]);

  // Row order is frozen while the pointer is down so a refresh cannot move the
  // row out from under the operator mid-click. Only the id sequence is held;
  // each row keeps rendering its latest data.
  const freeze = useCallback(() => {
    if (frozen.current) {
      return;
    }
    frozen.current = true;
    setFrozenIds(groups.flatMap((group) => group.rows.map((row) => row.cycleId)));
  }, [groups]);
  const release = useCallback(() => {
    frozen.current = false;
    setFrozenIds([]);
  }, []);

  return (
    <div
      className="grid content-start gap-6 p-[var(--shell-inset)]"
      onPointerCancel={release}
      onPointerDown={freeze}
      onPointerUp={release}
    >
      {/* A toolbar row: its controls are centred with the bar's own air above
          and below, and the same 12px gap every other bar uses. */}
      <div className="flex flex-wrap items-center justify-between gap-[var(--bar-gap)] py-[var(--bar-pad-y)]">
        <Input
          aria-label="Filter Cycles"
          className="h-[var(--bar-control-height)] w-full sm:w-80"
          onChange={(event) => setFilter(event.target.value)}
          placeholder="Filter Cycles"
          value={filter}
        />
        <Button
          aria-pressed={showCleared}
          className="h-[var(--bar-control-height)]"
          onClick={() => setShowCleared((current) => !current)}
          type="button"
          variant="outline"
        >
          {showCleared ? "Hide Cleared" : "Show Cleared"}
        </Button>
      </div>

      {cycles.isError ? (
        <p className="text-destructive" role="alert">
          The cycle list could not be read. Check the backend connection in the status bar.
        </p>
      ) : null}

      {groups.map((group) => (
        <CycleGroupSection
          group={group}
          key={group.id}
          loading={cycles.isPending}
          now={now}
          onOpen={(cycleId) => void navigate({ params: { cycleId }, to: "/cycles/$cycleId" })}
        />
      ))}
    </div>
  );
}

function CycleGroupSection({
  group,
  loading,
  now,
  onOpen,
}: {
  group: CycleGroup;
  loading: boolean;
  now: number;
  onOpen: (cycleId: string) => void;
}) {
  // An empty group is one quiet line — heading, count, what that means — so the
  // groups that hold work own the page.
  if (!group.rows.length) {
    return (
      <section aria-labelledby={`cycle-group-${group.id}`} className="grid">
        <p className="flex flex-wrap items-baseline gap-2">
          <span className="font-heading font-semibold tracking-wide" id={`cycle-group-${group.id}`}>
            {group.title}
          </span>
          <span className="font-mono text-xs tabular-nums text-muted-foreground">0</span>
          <span className="text-sm text-muted-foreground">
            {loading ? "Loading cycles…" : group.emptyText}
          </span>
        </p>
      </section>
    );
  }

  return (
    <section aria-labelledby={`cycle-group-${group.id}`} className="grid gap-1">
      <div className="flex items-baseline gap-2">
        <h2 className="font-heading font-semibold tracking-wide" id={`cycle-group-${group.id}`}>
          {group.title}
        </h2>
        <span className="font-mono text-xs tabular-nums text-muted-foreground">
          {group.rows.length}
        </span>
      </div>
      <div
        aria-hidden
        className={cn(
          ROW_GRID,
          "border-b pb-1 text-xs font-medium tracking-wide text-muted-foreground uppercase",
        )}
      >
        <span>Cycle</span>
        <span>Attention</span>
        <span>Status</span>
        <span>Cases</span>
        <span>Updated</span>
      </div>
      <ul className="grid">
        {group.rows.map((row) => (
          <li key={row.cycleId}>
            <CycleRowButton now={now} onOpen={onOpen} row={row} />
          </li>
        ))}
      </ul>
    </section>
  );
}

function CycleRowButton({
  now,
  onOpen,
  row,
}: {
  now: number;
  onOpen: (cycleId: string) => void;
  row: CycleRow;
}) {
  // The subtitle says something the title does not: the suite the cycle came
  // from, or when it was created when the suite repeats the title.
  const subtitle =
    row.suiteLabel && row.suiteLabel !== row.label
      ? row.suiteLabel
      : row.createdAtIso
        ? `Created ${new Date(row.createdAtIso).toLocaleString()}`
        : null;

  return (
    <button
      className={cn(
        "w-full min-h-[var(--list-row-height)] text-left outline-none transition-colors",
        ROW_GRID,
        "border-b border-border/60 py-2",
        "hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
      )}
      onClick={() => onOpen(row.cycleId)}
      type="button"
    >
      <span className="grid min-w-0 gap-0.5">
        <span className="truncate font-medium">{row.label}</span>
        {subtitle ? (
          <span className="truncate text-xs text-muted-foreground">{subtitle}</span>
        ) : null}
      </span>
      <span className="flex justify-start">
        {row.attentionCount ? (
          <Badge className={PILL_CLASS} variant="warning">
            {row.attentionCount} prompts
          </Badge>
        ) : null}
      </span>
      <span className="flex justify-start">
        <Badge className={PILL_CLASS} variant={badgeVariant(row.statusTone)}>
          {row.statusLabel}
        </Badge>
      </span>
      <span
        className="font-mono text-xs tabular-nums text-muted-foreground"
        title="Cases finished, of the cases this cycle plans"
      >
        {row.finishedCount} / {row.caseCount}
      </span>
      <span
        className="font-mono text-xs tabular-nums text-muted-foreground"
        title={row.updatedAtIso ? new Date(row.updatedAtIso).toLocaleString() : undefined}
      >
        {relativeUpdatedLabel(row.updatedAtIso, now)}
      </span>
    </button>
  );
}

/* === Private Functions === */

/**
 * How long ago the cycle last moved.
 *
 * "5s" alone reads as a duration; "5s Ago" says which way it points. The
 * absolute instant stays available on the cell's title.
 */
function relativeUpdatedLabel(updatedAtIso: string | null, now: number): string {
  const age = compactAgeLabel(updatedAtIso, now);
  return age === "-" ? age : `${age} Ago`;
}

function badgeVariant(tone: CycleRow["statusTone"]) {
  switch (tone) {
    case "good":
      return "success" as const;
    case "warn":
      return "warning" as const;
    case "bad":
      return "destructive" as const;
    default:
      return "outline" as const;
  }
}
