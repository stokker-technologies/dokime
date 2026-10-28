/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { useNativeSession } from "@/app/native-session/context";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { LIST_STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import { CycleStatusPill } from "@/features/cycles-home/CycleStatusPill";
import {
  applyFrozenOrder,
  archivedCycleGroups,
  type CycleGroup,
  type CycleRow,
  cycleRows,
  groupCycleRows,
  promptCountsByCycle,
} from "@/features/cycles-home/cycles-model";
import { cyclesHomeQueryKeys, useCyclesQuery, usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { setNativeCycleArchived } from "@/lib/api/native-execution";
import { PageNotice, PageShell } from "@/shell/PageShell";

const GUI_FILTER_DEBOUNCE_MS = 250;

/* The Status track, in the pill's own type size so it follows every density:
   room for the longest state label (16 characters at under 0.7em each), the
   pill's padding and the cell's. */
const STATUS_COLUMN_WIDTH = "w-[calc(var(--pill-text)_*_11_+_2_*_var(--pill-pad-x)_+_--spacing(6))]";

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

  useEffect(() => {
    const handle = window.setTimeout(() => setSelection(filter), GUI_FILTER_DEBOUNCE_MS);
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

  const attentionByCycleId = useMemo(() => promptCountsByCycle(prompts.data ?? []), [prompts.data]);

  const rows = useMemo(() => cycleRows(cycles.data ?? [], attentionByCycleId), [attentionByCycleId, cycles.data]);
  const groups = useMemo(() => {
    const filtered = selection.trim().length > 0;
    const grouped = showArchived ? archivedCycleGroups(rows, filtered) : groupCycleRows(rows, filtered);
    return frozenIds.length ? grouped.map((group) => ({ ...group, rows: applyFrozenOrder(group.rows, frozenIds) })) : grouped;
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
    <PageShell actions={actions} onPointerCancel={release} onPointerDown={freeze} onPointerUp={release} trail={["Cycles"]}>
      {cycles.isError ? (
        <PageNotice tone="alert">The cycle list could not be read. Check the backend connection in the status bar.</PageNotice>
      ) : null}

      {groups.map((group) => (
        <CycleGroupSection
          group={group}
          search={{ label: "Filter Cycles", value: filter, onChange: setFilter }}
          actions={
            <Button aria-pressed={showArchived} variant="outline" onClick={() => setShowArchived((current) => !current)}>
              Archived
            </Button>
          }
          key={group.id}
          loading={cycles.isPending}
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
    </PageShell>
  );
}

function CycleGroupSection({
  group,
  search,
  actions,
  loading,
  onOpen,
  onUnarchive,
}: {
  group: CycleGroup;
  search: import("@/components/data-table/data-table-model").DataTableSearch;
  actions: import("react").ReactNode;
  loading: boolean;
  onOpen: (cycleId: string) => void;
  /** Present in the Archived view: each row offers Unarchive. */
  onUnarchive: ((cycleId: string) => void) | null;
}) {
  const columns = useMemo(() => cycleColumns(onOpen, onUnarchive), [onOpen, onUnarchive]);

  // Section Counts Stay Outside The Table Card.
  const heading = (
    <div className="grid grid-cols-[auto_minmax(0,1fr)] items-baseline gap-x-[var(--bar-gap)]">
      <p className="flex items-baseline gap-2">
        <span className="font-heading text-lg font-semibold tracking-wide" id={`cycle-group-${group.id}`}>
          {group.title}
        </span>
        <span className="font-mono text-sm tabular-nums text-muted-foreground">{group.rows.length}</span>
      </p>
    </div>
  );

  return (
    <section id={group.id} aria-labelledby={`cycle-group-${group.id}`} className="grid gap-1">
      {heading}
      <div className="min-w-0">
        <DataTable
          search={search}
          actions={actions}
          columns={columns}
          data={group.rows}
          emptyText={loading ? "Loading Cycles" : group.emptyText}
          getRowId={rowId}
          onRowClick={(row) => onOpen(row.cycleId)}
        />
      </div>
    </section>
  );
}

/** One line: the cycle's own label. When it was created lives on the Updated
    tooltip. */
function CycleTitle({ row }: { row: CycleRow }) {
  return <span className="block truncate">{row.label}</span>;
}

function rowId(row: CycleRow): string {
  return row.cycleId;
}

/** The five columns every group shares. The Archived view adds Unarchive. */
function cycleColumns(onOpen: (id: string) => void, onUnarchive: ((cycleId: string) => void) | null): DataTableColumns<CycleRow> {
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
      cell: ({ row }) => <CycleStatusPill label={row.original.statusLabel} tone={row.original.statusTone} />,
      header: "Status",
      meta: { filter: "facet", label: "Status", width: STATUS_COLUMN_WIDTH },
    }),
    column.accessor("finishedCount", {
      cell: ({ row }) => (
        <span className="font-mono text-xs tabular-nums text-muted-foreground" title="Cases finished, of the cases this cycle plans">
          {row.original.finishedCount} / {row.original.caseCount}
        </span>
      ),
      header: "Cases",
      meta: { align: "end", label: "Cases", width: "w-28" },
    }),
    column.accessor((row) => row.updatedAtMs ?? undefined, {
      cell: ({ row }) => (
        <span className="font-mono text-xs tabular-nums text-muted-foreground" title={instantsLabel(row.original)}>
          {row.original.updatedAtIso ? new Date(row.original.updatedAtIso).toLocaleString() : "-"}
        </span>
      ),
      header: "Updated",
      id: "updated",
      meta: { align: "end", label: "Updated", width: "w-44" },
    }),
    column.display({
      id: "actions",
      header: "Actions",
      meta: { width: "w-36", align: "end" },
      cell: ({ row }) => (
        <DataTableRowActions label={row.original.label} primary={{ label: "Open", onSelect: () => onOpen(row.original.cycleId) }}>
          {onUnarchive ? <DropdownMenuItem onSelect={() => onUnarchive(row.original.cycleId)}>Unarchive</DropdownMenuItem> : null}
        </DataTableRowActions>
      ),
    }),
  ]);
}

/** The absolute update and creation instants. */
function instantsLabel(row: CycleRow): string | undefined {
  const instants = [
    row.updatedAtIso ? `Updated ${new Date(row.updatedAtIso).toLocaleString()}` : null,
    row.createdAtIso ? `Created ${new Date(row.createdAtIso).toLocaleString()}` : null,
  ].filter(Boolean);
  return instants.length ? instants.join("\n") : undefined;
}
