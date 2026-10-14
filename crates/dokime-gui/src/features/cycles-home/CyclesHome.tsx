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

import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
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
import { usePageChrome } from "@/shell/page-chrome";

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
  const columns = useMemo(() => cycleColumns(now), [now]);

  // Every group heads with one row — title and count at the start, the empty
  // statement at the end — so the three headings align whatever they hold.
  const heading = (
    <div className="grid grid-cols-[auto_minmax(0,1fr)] items-baseline gap-x-[var(--bar-gap)]">
      <p className="flex items-baseline gap-2">
        <span
          className="font-heading text-lg font-semibold tracking-wide"
          id={`cycle-group-${group.id}`}
        >
          {group.title}
        </span>
        <span className="font-mono text-sm tabular-nums text-muted-foreground">
          {group.rows.length}
        </span>
      </p>
      {group.rows.length ? null : (
        <span className="truncate text-right text-sm text-muted-foreground">
          {loading ? "Loading Cycles" : group.emptyText}
        </span>
      )}
    </div>
  );

  if (!group.rows.length) {
    return (
      <section aria-labelledby={`cycle-group-${group.id}`} className="grid">
        {heading}
      </section>
    );
  }

  // The table bleeds to the page edges so a row's hover spans the page while
  // its text, carried by the table inset, stays on the heading's line.
  return (
    <section aria-labelledby={`cycle-group-${group.id}`} className="grid gap-1">
      {heading}
      <DataTable
        className="-mx-[var(--shell-inset)]"
        columns={columns}
        data={group.rows}
        emptyText={group.emptyText}
        getRowId={rowId}
        onRowClick={(row) => onOpen(row.cycleId)}
        stickyHeader={false}
      />
    </section>
  );
}

/** One line: the title, then the suite it came from when that adds
    information. When the cycle was created lives on the Updated tooltip. */
function CycleTitle({ row }: { row: CycleRow }) {
  const suite = row.suiteLabel && row.suiteLabel !== row.label ? row.suiteLabel : null;

  return (
    <span className="flex min-w-0 items-baseline gap-2">
      <span className="truncate">{row.label}</span>
      {suite ? (
        <span className="min-w-0 truncate text-xs text-muted-foreground">{suite}</span>
      ) : null}
    </span>
  );
}

/* === Private Functions === */

function rowId(row: CycleRow): string {
  return row.cycleId;
}

/** The five columns every group shares; `now` ages the Updated column. */
function cycleColumns(now: number): DataTableColumns<CycleRow> {
  const column = dataTableColumns<CycleRow>();
  return column.columns([
    column.accessor("label", {
      cell: ({ row }) => <CycleTitle row={row.original} />,
      header: "Cycle",
      meta: { filter: "text", label: "Cycle", primary: true },
    }),
    column.accessor("attentionCount", {
      cell: ({ row }) =>
        row.original.attentionCount ? (
          <Badge className={LIST_STATUS_PILL_CLASS} variant="warning">
            {row.original.attentionCount} prompts
          </Badge>
        ) : null,
      header: "Attention",
      meta: { label: "Attention", width: "w-32" },
    }),
    column.accessor("statusLabel", {
      cell: ({ row }) => (
        <Badge className={LIST_STATUS_PILL_CLASS} variant={badgeVariant(row.original.statusTone)}>
          {row.original.statusLabel}
        </Badge>
      ),
      header: "Status",
      meta: { filter: "facet", label: "Status", width: "w-36" },
    }),
    column.accessor("finishedCount", {
      cell: ({ row }) => (
        <span
          className="font-mono text-xs tabular-nums text-muted-foreground"
          title="Cases finished, of the cases this cycle plans"
        >
          {row.original.finishedCount} / {row.original.caseCount}
        </span>
      ),
      header: "Cases",
      meta: { align: "end", label: "Cases", width: "w-28" },
    }),
    column.accessor((row) => row.updatedAtMs ?? undefined, {
      cell: ({ row }) => (
        <span
          className="font-mono text-xs tabular-nums text-muted-foreground"
          title={instantsLabel(row.original)}
        >
          {relativeUpdatedLabel(row.original.updatedAtIso, now)}
        </span>
      ),
      header: "Updated",
      id: "updated",
      meta: { align: "end", label: "Updated", width: "w-28" },
    }),
  ]);
}

/**
 * How long ago the cycle last moved.
 *
 * "5s" alone reads as a duration; "5s Ago" says which way it points. The
 * absolute instant stays available on the cell's title.
 */
/** The absolute instants behind the relative age: updated, then created. */
function instantsLabel(row: CycleRow): string | undefined {
  const instants = [
    row.updatedAtIso ? `Updated ${new Date(row.updatedAtIso).toLocaleString()}` : null,
    row.createdAtIso ? `Created ${new Date(row.createdAtIso).toLocaleString()}` : null,
  ].filter(Boolean);
  return instants.length ? instants.join("\n") : undefined;
}

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
