/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The open cycle's cases, as the Cycles destination's children.
 *
 * This is the only list of the cycle's cases: the workspace gives the whole
 * main region to evidence. The rows read the same query the workspace reads, so
 * the tree and the evidence never disagree about an attempt, and selection
 * leaves through the URL — `caseId` and the attempt the operator picked.
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
import { useCycleCommands, useCycleDetailQuery } from "./cycle-queries";
import {
  buildCycleCaseRows,
  type CycleCaseRow,
  caseActionTooltip,
  caseStatusLabel,
  caseStatusVariant,
  cycleStatusLabel,
  cycleStatusVariant,
  cycleTreeSelection,
  STATUS_PILL_CLASS,
} from "./cycle-workspace-model";

/* === Types === */

export type CycleSidebarCasesProps = {
  caseId: string | undefined;
  cycleId: string;
} & (
  | { live?: true; onSelectCase?: undefined }
  | { live: false; onSelectCase: (caseId: string | undefined) => void }
);

/* === Constants === */

/** The attempt count and the row menu, as two fixed trailing columns. */
const ATTEMPT_COLUMN_CLASS = "w-3 text-right text-xs tabular-nums";
const MENU_COLUMN_CLASS = "flex w-6 shrink-0 justify-center";

/* === Components === */

export function CycleSidebarCases({
  caseId,
  cycleId,
  live = true,
  onSelectCase,
}: CycleSidebarCasesProps) {
  const navigate = useNavigate();
  const detailQuery = useCycleDetailQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const [pendingDelete, setPendingDelete] = useState<CycleCaseRow | null>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const rows = useMemo(() => buildCycleCaseRows(plan, executions), [executions, plan]);
  const selection = cycleTreeSelection(caseId, rows);

  const report = (outcome: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(outcome.done))
      .catch((error: unknown) =>
        toast.error(outcome.failed, {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  /* Overview clears the case and execution selection. The evidence pane
     asks for an execution before resolving a document. */
  const selectOverview = () => {
    if (!live) {
      onSelectCase?.(undefined);
      return;
    }
    void navigate({
      params: { cycleId },
      replace: true,
      search: (current: Record<string, unknown>) => ({
        ...current,
        caseId: undefined,
        executionId: undefined,
      }),
      to: "/cycles/$cycleId",
    });
  };

  const select = (row: CycleCaseRow) => {
    if (!live) {
      onSelectCase?.(row.caseId);
      return;
    }
    void navigate({
      params: { cycleId },
      replace: true,
      search: (current: Record<string, unknown>) => ({
        ...current,
        caseId: row.caseId,
        executionId: row.browsable.at(-1)?.test_execution_id,
      }),
      to: "/cycles/$cycleId",
    });
  };

  return (
    <>
      {plan ? (
        <SidebarRow
          active={selection.kind === "overview"}
          label={plan.test_cycle.title}
          level="overview"
          onSelect={selectOverview}
          title={plan.test_cycle.title}
        >
          <Badge className={STATUS_PILL_CLASS} variant={cycleStatusVariant(plan.test_cycle.status)}>
            {cycleStatusLabel(plan.test_cycle.status)}
          </Badge>
          <span aria-hidden className={ATTEMPT_COLUMN_CLASS} />
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
          <CaseTreeRow
            key={row.caseId}
            onDelete={setPendingDelete}
            onRun={() =>
              report({ done: "Execution Started", failed: "Execution Could Not Start" }, () => {
                const execution = row.attempts.find((attempt) => attempt.status === "planned");
                if (!execution) throw new Error("No unstarted execution is available.");
                return commands.runCase.mutateAsync(execution.test_execution_id);
              })
            }
            onScheduleRerun={() =>
              report({ done: "Rerun scheduled", failed: "The rerun could not be scheduled." }, () =>
                commands.scheduleRerun.mutateAsync(row.caseId),
              )
            }
            live={live}
            onSelect={() => select(row)}
            row={row}
            selected={selection.kind === "case" && selection.caseId === row.caseId}
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
                ? `Deleting removes the case "${pendingDelete.label}" and its ${pendingDelete.attemptCount} attempts from this cycle. This cannot be undone.`
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
                if (row.caseId === caseId) {
                  void navigate({
                    params: { cycleId },
                    replace: true,
                    search: (current: Record<string, unknown>) => ({
                      ...current,
                      caseId: undefined,
                      executionId: undefined,
                    }),
                    to: "/cycles/$cycleId",
                  });
                }
                report({ done: "Case deleted", failed: "The case could not be deleted." }, () =>
                  commands.deleteCase.mutateAsync(row.caseId),
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

function CaseTreeRow({
  live,
  onDelete,
  onRun,
  onScheduleRerun,
  onSelect,
  row,
  selected,
}: {
  live: boolean;
  onDelete: (row: CycleCaseRow) => void;
  onRun: () => void;
  onScheduleRerun: () => void;
  onSelect: () => void;
  row: CycleCaseRow;
  selected: boolean;
}) {
  /* Eligibility is the runtime's, not a guess from the row's own status: the
     projection computes it from the checks each mutation enforces, and its
     reasons are the only honest thing to put on a disabled item. A projection
     that states no eligibility at all offers every item, and the mutation
     itself refuses. */
  const actions = row.actions;
  const reason = caseActionTooltip(row);
  const allowed = (key: "delete" | "run" | "schedule_rerun") => actions === null || actions[key];

  return (
    <SidebarRow
      active={selected}
      label={row.label}
      level="case"
      onSelect={onSelect}
      title={row.label}
    >
      <Badge
        className={STATUS_PILL_CLASS}
        title={row.rerunScheduled ? "Rerun Scheduled" : undefined}
        variant={caseStatusVariant(row.status)}
      >
        {caseStatusLabel(row.status)}
      </Badge>
      {/* What this column offers is somewhere to go, so it counts the
          attempts there is something to read. The runtime counts a
          materialized but undispatched attempt in `attempt_count` — a queued
          case holds one — and printing that here would draw a 1 that leads
          nowhere; the tooltip states it instead, where it explains rather than
          invites. */}
      <span
        className={ATTEMPT_COLUMN_CLASS}
        title={
          row.browsable.length
            ? `${row.browsable.length} ${row.browsable.length === 1 ? "Attempt" : "Attempts"}`
            : row.attemptCount
              ? "One attempt is materialized and has not been dispatched"
              : "No Attempts Yet"
        }
      >
        {row.browsable.length || ""}
      </span>
      {/* The menu's column stays, so a frozen outline and a live tree put the
          label, the pill and the attempt count on exactly the same x. */}
      {live ? (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              aria-label={`Actions For ${row.label}`}
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
              onSelect={onScheduleRerun}
              title={allowed("schedule_rerun") ? undefined : reason}
            >
              <RotateCcw />
              Schedule Rerun
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
