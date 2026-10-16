/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArchiveRestore } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { LIST_STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import {
  applyFrozenOrder,
  archivedCycleGroups,
  type CycleGroup,
  type CycleRow,
  cycleRows,
  groupCycleRows,
} from "@/features/cycles-home/cycles-model";
import {
  cyclesHomeQueryKeys,
  useCyclesQuery,
  usePendingPromptsQuery,
} from "@/features/cycles-home/cycles-queries";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { setNativeCycleArchived } from "@/lib/api/native-execution";
import { compactAgeLabel } from "@/lib/time-format";
import { usePageChrome } from "@/shell/page-chrome";

/* === Components === */

/**
 * Home. Three fixed groups over every loaded cycle, a filter box, and one way
 * into a cycle: its row. The Archived view lists the archived cycles instead,
 * each with Unarchive.
 */
export function CyclesHome() {
  const navigate = useNavigate();
  const [filter, setFilter] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const [creating, setCreating] = useState(false);
  const session = useNativeSession();
  const queryClient = useQueryClient();
  const unarchive = useMutation({
    mutationFn: (cycleId: string) => setNativeCycleArchived(session.identity, cycleId, false),
    onSettled: () => queryClient.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all }),
  });
  /* The box types faster than a round trip; the request follows the operator
     rather than every keystroke, and the box itself never lags. */
  const [selection, setSelection] = useState("");
  const cycles = useCyclesQuery({ archived: showArchived, text: selection });
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
      <Button className="h-[var(--control-height)]" onClick={() => setCreating(true)} type="button">
        New Cycle
      </Button>
    ),
    [],
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
    const filtered = selection.trim().length > 0;
    const grouped = showArchived
      ? archivedCycleGroups(rows, filtered)
      : groupCycleRows(rows, filtered);
    return frozenIds.length
      ? grouped.map((group) => ({ ...group, rows: applyFrozenOrder(group.rows, frozenIds) }))
      : grouped;
  }, [frozenIds, rows, selection, showArchived]);

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
          aria-pressed={showArchived}
          className="h-[var(--bar-control-height)]"
          onClick={() => setShowArchived((current) => !current)}
          type="button"
          variant={showArchived ? "secondary" : "outline"}
        >
          Archived
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
          onUnarchive={
            showArchived
              ? (cycleId) =>
                  unarchive
                    .mutateAsync(cycleId)
                    .then(() => toast.success("Unarchived"))
                    .catch((error: unknown) =>
                      toast.error("The archive mark could not be removed.", {
                        description: error instanceof Error ? error.message : String(error),
                      }),
                    )
              : null
          }
        />
      ))}

      <NewCycleDialog
        onCreated={(cycleId) => void navigate({ params: { cycleId }, to: "/cycles/$cycleId" })}
        onOpenChange={setCreating}
        open={creating}
      />
    </div>
  );
}

function CycleGroupSection({
  group,
  loading,
  now,
  onOpen,
  onUnarchive,
}: {
  group: CycleGroup;
  loading: boolean;
  now: number;
  onOpen: (cycleId: string) => void;
  /** Present in the Archived view: each row offers Unarchive. */
  onUnarchive: ((cycleId: string) => void) | null;
}) {
  const columns = useMemo(() => cycleColumns(now, onUnarchive), [now, onUnarchive]);

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

/** One line: the cycle's own label. When it was created lives on the Updated
    tooltip. */
function CycleTitle({ row }: { row: CycleRow }) {
  return <span className="block truncate">{row.label}</span>;
}

/* === Private Functions === */

function rowId(row: CycleRow): string {
  return row.cycleId;
}

/** The five columns every group shares; `now` ages the Updated column. The
    Archived view adds Unarchive as the last column. */
function cycleColumns(
  now: number,
  onUnarchive: ((cycleId: string) => void) | null,
): DataTableColumns<CycleRow> {
  const column = dataTableColumns<CycleRow>();
  const unarchive = onUnarchive
    ? [
        column.display({
          cell: ({ row }) => (
            <Button
              className="h-[var(--control-height)]"
              onClick={(event) => {
                event.stopPropagation();
                onUnarchive(row.original.cycleId);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              <ArchiveRestore data-icon="inline-start" />
              Unarchive
            </Button>
          ),
          header: "",
          id: "unarchive",
          meta: { align: "end", label: "Unarchive", width: "w-36" },
        }),
      ]
    : [];
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
    ...unarchive,
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
