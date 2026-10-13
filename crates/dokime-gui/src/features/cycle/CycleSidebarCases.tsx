/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The open cycle's executions, as the Cycles destination's children.
 *
 * This is the only list of the cycle's work: the workspace gives the whole
 * main region to evidence. One row is one execution, in cycle order, so a
 * re-run is a new row at the end marked with the row it re-runs. The rows read
 * the same query the workspace reads, so the tree and the evidence never
 * disagree, and selection leaves through the URL as `executionId` — or as
 * `caseId` for a case the runtime has no execution for yet.
 *
 * With `live={false}` the same rows become the frozen report's outline: the
 * same grid, the same pills, no row menus and no commands, and selection is
 * handed back to the report rather than written to a URL it does not own.
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { Ellipsis, Play, RotateCcw, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SidebarRow } from "@/shell/sidebar-row";
import { useCycleCommands, useCycleDetailQuery, useRerunCase } from "./cycle-queries";
import {
  buildCycleCases,
  buildCycleExecutionRows,
  type CycleExecutionRow,
  type CycleSelection,
  caseActionTooltip,
  cycleRowSelection,
  cycleRowStatus,
  cycleStatusLabel,
  cycleStatusVariant,
  isPlannedExecution,
  STATUS_PILL_CLASS,
  selectedCycleRow,
} from "./cycle-workspace-model";

/* === Types === */

export type CycleSidebarCasesProps = {
  cycleId: string;
  selection: CycleSelection;
} & (
  | { live?: true; onSelect?: undefined }
  | { live: false; onSelect: (selection: CycleSelection) => void }
);

/* === Constants === */

/** The execution ordinal and the row menu, as two fixed trailing columns. */
const ORDINAL_COLUMN_CLASS = "w-7 text-right text-xs tabular-nums";
const MENU_COLUMN_CLASS = "flex w-6 shrink-0 justify-center";

/* === Components === */

export function CycleSidebarCases({
  cycleId,
  live = true,
  onSelect,
  selection,
}: CycleSidebarCasesProps) {
  const navigate = useNavigate();
  const detailQuery = useCycleDetailQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const rerunCase = useRerunCase(cycleId);
  const [pendingDelete, setPendingDelete] = useState<CycleExecutionRow | null>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const rows = useMemo(
    () => buildCycleExecutionRows(buildCycleCases(plan), executions),
    [executions, plan],
  );
  /* A stale selection selects the overview, so exactly one row is ever selected. */
  const selectedKey = selectedCycleRow(rows, selection)?.key ?? null;

  const report = (outcome: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(outcome.done))
      .catch((error: unknown) =>
        toast.error(outcome.failed, {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  /* The overview is the empty selection. A row selects its execution, or its
     case while the runtime has no execution for it. */
  const select = (next: CycleSelection) => {
    if (!live) {
      onSelect?.(next);
      return;
    }
    void navigate({
      params: { cycleId },
      replace: true,
      search: (current: Record<string, unknown>) => ({
        ...current,
        caseId: next.caseId,
        executionId: next.executionId,
      }),
      to: "/cycles/$cycleId",
    });
  };

  return (
    <>
      {plan ? (
        <SidebarRow
          active={selectedKey === null}
          label={plan.test_cycle.title}
          level="overview"
          onSelect={() => select({})}
          title={plan.test_cycle.title}
        >
          <Badge className={STATUS_PILL_CLASS} variant={cycleStatusVariant(plan.test_cycle.status)}>
            {cycleStatusLabel(plan.test_cycle.status)}
          </Badge>
          <span aria-hidden className={ORDINAL_COLUMN_CLASS} />
          <span aria-hidden className={MENU_COLUMN_CLASS} />
        </SidebarRow>
      ) : null}

      {detailQuery.isError ? (
        <TreeNotice
          action={{ label: "Try Again", onClick: () => void detailQuery.refetch() }}
          message={
            detailQuery.error instanceof Error
              ? detailQuery.error.message
              : "The cycle's cases could not be read."
          }
          title="Cases Unavailable"
        />
      ) : detailQuery.isPending && rows.length === 0 ? (
        <TreeNotice message="Reading this cycle's cases." title="Loading" />
      ) : rows.length === 0 ? (
        <TreeNotice
          action={
            live
              ? {
                  label: "Add Cases",
                  onClick: () => void navigate({ search: { appendTo: cycleId }, to: "/library" }),
                }
              : undefined
          }
          message="This cycle has no cases yet."
          title="No Cases"
        />
      ) : (
        rows.map((row) => (
          <ExecutionTreeRow
            key={row.key}
            onDelete={setPendingDelete}
            onRun={() =>
              report({ done: "Execution Started", failed: "Execution Could Not Start" }, () => {
                const execution = row.execution;
                if (!execution || !isPlannedExecution(execution.status)) {
                  throw new Error("No unstarted execution is available.");
                }
                return commands.runCase.mutateAsync(execution.test_execution_id);
              })
            }
            onRerun={() =>
              report({ done: "Case Re-Run Started", failed: "Case Re-Run Failed" }, async () => {
                const created = await rerunCase.rerun(row.case.caseId);
                if (created) select({ executionId: created });
              })
            }
            live={live}
            onSelect={() => select(cycleRowSelection(row))}
            row={row}
            selected={row.key === selectedKey}
          />
        ))
      )}

      <Dialog
        onOpenChange={(open) => !open && setPendingDelete(null)}
        open={live && pendingDelete !== null}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Case</DialogTitle>
            <DialogDescription>
              {pendingDelete
                ? `Deleting removes the case "${pendingDelete.case.label}" and its ${caseExecutionCount(rows, pendingDelete.case.caseId)} executions from this cycle. This cannot be undone.`
                : ""}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setPendingDelete(null)} variant="outline">
              Cancel
            </Button>
            <Button
              onClick={() => {
                const row = pendingDelete;
                setPendingDelete(null);
                if (!row) return;
                const selected = rows.find((candidate) => candidate.key === selectedKey);
                if (selected?.case.caseId === row.case.caseId) select({});
                report({ done: "Case deleted", failed: "The case could not be deleted." }, () =>
                  commands.deleteCase.mutateAsync(row.case.caseId),
                );
              }}
              variant="danger"
            >
              Delete Case
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

/* === Private Components === */

function ExecutionTreeRow({
  live,
  onDelete,
  onRerun,
  onRun,
  onSelect,
  row,
  selected,
}: {
  live: boolean;
  onDelete: (row: CycleExecutionRow) => void;
  onRerun: () => void;
  onRun: () => void;
  onSelect: () => void;
  row: CycleExecutionRow;
  selected: boolean;
}) {
  /* Eligibility is the runtime's, not a guess from the row's own status: the
     projection computes it from the checks each mutation enforces, and its
     reasons are the only honest thing to put on a disabled item. A projection
     that states no eligibility at all offers every item, and the mutation
     itself refuses. The case's actions live on its newest row: an earlier
     execution is history, and acting on the case from it would be ambiguous. */
  const actions = row.case.actions;
  const reason = caseActionTooltip(row.case);
  const allowed = (key: "delete" | "run" | "schedule_rerun") => actions === null || actions[key];
  const status = cycleRowStatus(row);
  const label = row.case.label;

  return (
    <SidebarRow
      active={selected}
      data-execution-id={row.execution?.test_execution_id}
      data-rerun-of={row.rerunOf ?? undefined}
      label={
        row.rerunOf === null ? (
          label
        ) : (
          <>
            {label}{" "}
            <span className="text-xs whitespace-nowrap text-muted-foreground">
              Re-Run Of #{row.rerunOf}
            </span>
          </>
        )
      }
      level="case"
      onSelect={onSelect}
      title={row.rerunOf === null ? label : `${label} · Re-Run Of #${row.rerunOf}`}
    >
      <Badge
        className={STATUS_PILL_CLASS}
        title={row.newest && row.case.rerunScheduled ? "Rerun Scheduled" : undefined}
        variant={status.variant}
      >
        {status.label}
      </Badge>
      <span
        className={ORDINAL_COLUMN_CLASS}
        title={row.execution ? `Execution ${row.ordinal}` : "No Execution Yet"}
      >
        {row.execution ? `#${row.ordinal}` : ""}
      </span>
      {/* The menu's column stays, so a frozen outline, a live tree and an
          earlier execution put the label, the pill and the ordinal on exactly
          the same x. */}
      {live && row.newest ? (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              aria-label={`Actions For ${label}`}
              className="size-6"
              size="icon-sm"
              variant="ghost"
            >
              <Ellipsis />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-48" side="right">
            <DropdownMenuItem
              disabled={!allowed("run")}
              onSelect={onRun}
              title={allowed("run") ? undefined : reason}
            >
              <Play />
              Run Case
            </DropdownMenuItem>
            <DropdownMenuItem
              disabled={!allowed("schedule_rerun")}
              onSelect={onRerun}
              title={allowed("schedule_rerun") ? undefined : reason}
            >
              <RotateCcw />
              Re-Run Case
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              disabled={!allowed("delete")}
              onSelect={() => onDelete(row)}
              title={allowed("delete") ? undefined : reason}
              variant="destructive"
            >
              <Trash2 />
              Delete Case
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      ) : (
        <span aria-hidden className={MENU_COLUMN_CLASS} />
      )}
    </SidebarRow>
  );
}

function TreeNotice({
  action,
  message,
  title,
}: {
  action?: { label: string; onClick: () => void };
  message: string;
  title: string;
}) {
  return (
    <li className="grid gap-1.5 py-3 pr-2 pl-[3.75rem]">
      <p className="font-heading text-xs font-semibold">{title}</p>
      <p className="text-xs leading-snug text-muted-foreground">{message}</p>
      {action ? (
        <div className="grid justify-start pt-1">
          <Button onClick={action.onClick} size="sm" variant="outline">
            {action.label}
          </Button>
        </div>
      ) : null}
    </li>
  );
}

/* === Private Functions === */

/** How many executions of `caseId` the list holds. */
function caseExecutionCount(rows: readonly CycleExecutionRow[], caseId: string): number {
  return rows.filter((row) => row.case.caseId === caseId && row.execution !== null).length;
}
