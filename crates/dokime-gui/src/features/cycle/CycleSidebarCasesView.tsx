/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The open cycle as a compact explorer tree: the cycle node and, under it,
 * its executions in cycle order.
 *
 * Every row is one line on one grid — chevron, status glyph, label, Run
 * Again, ordinal or progress, menu — with the same column widths on every
 * row, so glyphs, ordinals and actions line up whatever the labels say. A row
 * is `--tree-row-height` tall (28px on a desk, 44px at touch). The label is
 * cut with an ellipsis and its full text is the row's tooltip and accessible
 * name; the status is a coloured glyph whose word is its tooltip and is also
 * read as text. The row menu appears on hover or focus, and on a long press at
 * touch.
 *
 * It is a WAI-ARIA tree: arrow keys move between rows, left and right collapse
 * and expand the cycle node, Enter selects. Selecting the cycle node selects
 * the cycle itself (`view: "cycle"`); a row selects its execution, or its case
 * while the runtime has none. The rows read the same query the workspace
 * reads, so the tree and the document never disagree. The running execution
 * carries a marker while Follow Active is following it, and an execution whose
 * prompt waits for an answer shows the input-needed glyph: the prompt itself
 * is answered on that execution's page.
 *
 * With `live={false}` the same tree is the frozen report's outline: the same
 * grid and glyphs, no menus and no commands, and selection is handed back to
 * the report rather than written to a URL it does not own.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import { useNavigate } from "@tanstack/react-router";
import {
  BellRing,
  Check,
  ChevronDown,
  ChevronRight,
  Circle,
  Ellipsis,
  LoaderCircle,
  type LucideIcon,
  Minus,
  Play,
  RotateCcw,
  Square,
  Trash2,
  TriangleAlert,
  Unplug,
  X,
} from "lucide-react";
import { type KeyboardEvent, type RefObject, useCallback, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

// Local Imports

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { TestCyclePlan, TestExecution } from "@/lib/types";
import { useRowWindow } from "@/lib/use-row-window";
import { cn } from "@/lib/utils";
import { useSelectionStore } from "@/stores/selection-store";
import {
  buildCycleCases,
  buildCycleExecutionRows,
  type CycleExecutionRow,
  type CycleLifecycle,
  type CycleSelection,
  caseActionTooltip,
  caseStatusLabel,
  cycleRowSelection,
  cycleState,
  executionStatusLabel,
  finishedCaseCount,
  isFollowing,
  isPlannedExecution,
  rowOffersRunAgain,
  selectedCycleRow,
} from "./cycle-workspace-model";

/* === Re-Exports === */

/* === Constants === */

/**
 * The one row grid: chevron, glyph, label, Run Again, ordinal, menu. Every
 * row uses it, so every trailing column sits on the same x.
 */
const ROW_GRID =
  "grid h-[var(--tree-row-height)] min-w-0 grid-cols-[1.25rem_1.25rem_minmax(0,1fr)_var(--tree-action-width)_2.75rem_var(--tree-action-width)] items-center gap-x-1 rounded-md pr-1";

const TONE_CLASS: Record<GlyphTone, string> = {
  danger: "text-destructive",
  muted: "text-muted-foreground",
  running: "text-primary",
  success: "text-success",
  warning: "text-warning",
};

/** How long a touch must rest on a row before its menu opens. */
const LONG_PRESS_MS = 500;

/* === Types === */

export type CycleSidebarState = {
  data?: { plan: TestCyclePlan | null; executions: TestExecution[] };
  isError: boolean;
  error: unknown;
  refetch?: () => unknown;
};

export type CycleSidebarActions = {
  rerunCase: (caseId: string) => Promise<string | null | undefined>;
  startExecution: (executionId: string) => Promise<unknown>;
  deleteCase: (caseId: string) => Promise<unknown>;
};

export type CycleSidebarCasesViewProps = {
  detailQuery: CycleSidebarState;
  actions?: CycleSidebarActions;
  cycleId: string;
  selection: CycleSelection;
  /** The executions a pending prompt waits on. */
  inputNeeded?: ReadonlySet<string>;
} & ({ live?: true; onSelect?: undefined } | { live: false; onSelect: (selection: CycleSelection) => void });

type GlyphTone = "running" | "success" | "danger" | "warning" | "muted";

type Glyph = { icon: LucideIcon; tone: GlyphTone; spin?: boolean; label: string };

/* === Components === */

export function CycleSidebarCasesView({
  cycleId,
  detailQuery,
  actions,
  inputNeeded,
  live = true,
  onSelect,
  selection,
}: CycleSidebarCasesViewProps) {
  const navigate = useNavigate();
  const followActive = useSelectionStore((state) => state.followActive);
  const [pendingDelete, setPendingDelete] = useState<CycleExecutionRow | null>(null);
  const [expanded, setExpanded] = useState(true);
  const treeRef = useRef<HTMLUListElement>(null);

  const plan = detailQuery.data?.plan ?? null;
  const executions = useMemo(() => detailQuery.data?.executions ?? [], [detailQuery.data]);
  const cases = useMemo(() => buildCycleCases(plan), [plan]);
  const rows = useMemo(() => buildCycleExecutionRows(cases, executions), [cases, executions]);
  // A long cycle renders only the rows in view (`useRowWindow`); every row is one height.
  const rowWindow = useRowWindow(rows.length);
  /* A stale selection selects the cycle, so exactly one row is ever selected. */
  const selectedKey = selectedCycleRow(rows, selection)?.key ?? null;
  const activeExecutionId =
    plan?.test_cycle.current_test_execution_id ?? executions.find((execution) => execution.status === "running")?.test_execution_id ?? null;
  const following = live && followActive && isFollowing(selection, activeExecutionId);

  const report = (outcome: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(outcome.done))
      .catch((error: unknown) =>
        toast.error(outcome.failed, {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

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
        view: next.view,
      }),
      to: "/cycles/$cycleId",
    });
  };

  /* Arrow keys walk the visible rows; left and right fold the cycle node and
     step between it and its first execution; Enter and Space select. */
  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLUListElement>) => {
      const tree = treeRef.current;
      const current = event.target instanceof HTMLElement ? event.target : null;
      if (!tree || current?.getAttribute("role") !== "treeitem") return;
      const items = [...tree.querySelectorAll<HTMLElement>("[role=treeitem]")];
      const index = items.indexOf(current);
      const isCycle = current.dataset.level === "overview";
      const move = (target: HTMLElement | undefined) => {
        event.preventDefault();
        target?.focus();
      };
      switch (event.key) {
        case "ArrowDown":
          move(items[index + 1]);
          break;
        case "ArrowUp":
          move(items[index - 1]);
          break;
        case "Home":
          move(items[0]);
          break;
        case "End":
          /* The last row may sit outside the rendered window; bring it in first. */
          event.preventDefault();
          if (rows.length) rowWindow.focusIndex(rows.length - 1, (row) => row.focus());
          else move(items.at(-1));
          break;
        case "ArrowRight":
          if (isCycle && !expanded) {
            event.preventDefault();
            setExpanded(true);
          } else if (isCycle) move(items[index + 1]);
          break;
        case "ArrowLeft":
          if (isCycle) {
            event.preventDefault();
            setExpanded(false);
          } else move(items[0]);
          break;
        case "Enter":
        case " ":
          event.preventDefault();
          current.querySelector<HTMLButtonElement>(":scope > div button[data-tree-select]")?.click();
          break;
        default:
      }
    },
    [expanded, rowWindow.focusIndex, rows.length],
  );

  const total = cases.length;
  const finished = finishedCaseCount(cases);
  const cycleSelected = selectedKey === null;

  return (
    <li className="grid min-w-0" role="none">
      <ul
        aria-label="Cycle Executions"
        className="grid min-w-0 content-start gap-px"
        onKeyDown={onKeyDown}
        ref={treeRef}
        // biome-ignore lint/a11y/noNoninteractiveElementToInteractiveRole: an ARIA tree of list items keeps its rows `li`s, which the desktop journeys address.
        role="tree"
      >
        {plan ? (
          <li
            aria-expanded={expanded}
            aria-label={`${plan.test_cycle.title}, ${cycleState(plan.test_cycle.phase, plan.test_cycle.status, false).label}`}
            aria-level={1}
            aria-selected={cycleSelected}
            className="grid min-w-0 gap-px outline-none [&:focus-visible>div]:ring-2 [&:focus-visible>div]:ring-ring"
            data-level="overview"
            role="treeitem"
            tabIndex={cycleSelected ? 0 : -1}
          >
            <div className={cn(ROW_GRID, rowTone(cycleSelected))}>
              <button
                aria-label={expanded ? "Collapse Executions" : "Expand Executions"}
                className="grid size-5 place-items-center rounded-sm text-muted-foreground hover:text-foreground"
                onClick={() => setExpanded((open) => !open)}
                tabIndex={-1}
                type="button"
              >
                {expanded ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
              </button>
              <StatusGlyph
                glyph={cycleGlyph(plan.test_cycle.phase, plan.test_cycle.status, inputNeeded !== undefined && inputNeeded.size > 0)}
              />
              <button
                className="min-w-0 truncate text-left text-sm font-medium"
                data-tree-select
                onClick={() => select({ view: "cycle" })}
                tabIndex={-1}
                title={plan.test_cycle.title}
                type="button"
              >
                {plan.test_cycle.title}
              </button>
              <span aria-hidden />
              <span className="text-right text-xs text-muted-foreground tabular-nums" title={`${finished} of ${total} cases finished`}>
                {finished}/{total}
              </span>
              <span aria-hidden />
            </div>

            {expanded ? (
              // biome-ignore lint/a11y/useSemanticElements: a tree node's children are an ARIA group, which no native element expresses.
              <ul className="grid min-w-0 gap-px" ref={rowWindow.anchor as RefObject<HTMLUListElement | null>} role="group">
                {detailQuery.isError ? (
                  <TreeNotice
                    action={{ label: "Try Again", onClick: () => void detailQuery.refetch?.() }}
                    message={detailQuery.error instanceof Error ? detailQuery.error.message : "The cycle's cases could not be read."}
                    title="Cases Unavailable"
                  />
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
                  <>
                    {rowWindow.window.before ? <li aria-hidden role="none" style={{ height: `${rowWindow.window.before}px` }} /> : null}
                    {rows.slice(rowWindow.window.first, rowWindow.window.last).map((row, offset) => (
                      <ExecutionTreeRow
                        active={plan?.test_cycle.phase === "active"}
                        following={following && row.execution !== null && row.execution.test_execution_id === activeExecutionId}
                        inputNeeded={row.execution !== null && inputNeeded?.has(row.execution.test_execution_id) === true}
                        key={row.key}
                        live={live}
                        onDelete={setPendingDelete}
                        onRerun={() =>
                          report({ done: "New Attempt Ready To Start", failed: "Case Re-Run Failed" }, async () => {
                            const created = await liveActions(actions).rerunCase(row.case.caseId);
                            if (created) select({ executionId: created });
                          })
                        }
                        onRun={() =>
                          report({ done: "Execution Started", failed: "Execution Could Not Start" }, () => {
                            const execution = row.execution;
                            if (!execution || !isPlannedExecution(execution.status)) {
                              throw new Error("No unstarted execution is available.");
                            }
                            return liveActions(actions).startExecution(execution.test_execution_id);
                          })
                        }
                        onSelect={() => select(cycleRowSelection(row))}
                        row={row}
                        selected={row.key === selectedKey}
                        windowIndex={rowWindow.window.first + offset}
                      />
                    ))}
                    {rowWindow.window.after ? <li aria-hidden role="none" style={{ height: `${rowWindow.window.after}px` }} /> : null}
                  </>
                )}
              </ul>
            ) : null}
          </li>
        ) : detailQuery.isError ? (
          <TreeNotice
            action={{ label: "Try Again", onClick: () => void detailQuery.refetch?.() }}
            message={detailQuery.error instanceof Error ? detailQuery.error.message : "The cycle could not be read."}
            title="Cycle Unavailable"
          />
        ) : (
          <TreeNotice message="Reading this cycle's cases." title="Loading" />
        )}
      </ul>

      <Dialog onOpenChange={(open) => !open && setPendingDelete(null)} open={live && pendingDelete !== null}>
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
                if (selected?.case.caseId === row.case.caseId) select({ view: "cycle" });
                report({ done: "Case deleted", failed: "The case could not be deleted." }, () =>
                  liveActions(actions).deleteCase(row.case.caseId),
                );
              }}
              variant="danger"
            >
              Delete Case
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </li>
  );
}

function ExecutionTreeRow({
  active,
  following,
  inputNeeded,
  windowIndex,
  live,
  onDelete,
  onRerun,
  onRun,
  onSelect,
  row,
  selected,
}: {
  active: boolean;
  following: boolean;
  inputNeeded: boolean;
  live: boolean;
  onDelete: (row: CycleExecutionRow) => void;
  onRerun: () => void;
  onRun: () => void;
  onSelect: () => void;
  row: CycleExecutionRow;
  selected: boolean;
  /** The row's position in the whole list, which windowing reads back. */
  windowIndex: number;
}) {
  /* Eligibility is the runtime's, not a guess from the row's own status: the
     projection computes it from the checks each mutation enforces, and its
     reasons are the only honest thing to put on a disabled item. A projection
     that states no eligibility at all offers every item, and the mutation
     itself refuses. The case's actions live on its newest row: an earlier
     execution is history, and acting on the case from it would be ambiguous. */
  const actions = row.case.actions;
  const reason = caseActionTooltip(row.case);
  const deleteAllowed = actions === null || actions.delete;
  const startAllowed = active && row.execution !== null && isPlannedExecution(row.execution.status);
  const glyph: Glyph = inputNeeded ? { icon: BellRing, label: "Waiting For Input", tone: "warning" } : rowGlyph(row);
  const label = row.rerunOf === null ? row.case.label : `${row.case.label} · Re-Run Of #${row.rerunOf}`;
  const [menuOpen, setMenuOpen] = useState(false);
  const press = useLongPress(() => setMenuOpen(true));
  const menu = live && row.newest;

  return (
    <li
      aria-label={`${label}, ${glyph.label}`}
      aria-level={2}
      aria-selected={selected}
      className="group/row grid min-w-0 outline-none [&:focus-visible>div]:ring-2 [&:focus-visible>div]:ring-ring"
      data-execution-id={row.execution?.test_execution_id}
      data-following={following || undefined}
      data-level="case"
      data-rerun-of={row.rerunOf ?? undefined}
      data-window-row={windowIndex}
      role="treeitem"
      tabIndex={selected ? 0 : -1}
      {...(menu ? press : {})}
    >
      <div
        className={cn(ROW_GRID, "pl-[var(--tree-indent)]", rowTone(selected), following && "shadow-[inset_2px_0_0_var(--color-primary)]")}
      >
        <span aria-hidden />
        <StatusGlyph glyph={glyph} />
        <button
          className="min-w-0 truncate text-left text-sm"
          data-tree-select
          onClick={onSelect}
          tabIndex={-1}
          title={label}
          type="button"
        >
          {row.case.label}
          {row.rerunOf === null ? null : <span className="text-xs text-muted-foreground"> Re-Run Of #{row.rerunOf}</span>}
        </button>
        {/* Run Again is the one row command an operator reaches for after a
            case ends, so it is a visible target, not a menu item. */}
        {live && rowOffersRunAgain(row) ? (
          <Button
            aria-label={`Run Again ${row.case.label}`}
            className="size-[var(--tree-action-width)]"
            disabled={!active}
            onClick={onRerun}
            size="icon-sm"
            tabIndex={-1}
            title={active ? "Append New Attempt" : "Only Active Cycles Accept New Attempts"}
            variant="ghost"
          >
            <RotateCcw />
          </Button>
        ) : (
          <span aria-hidden />
        )}
        <span
          className="text-right text-xs text-muted-foreground tabular-nums"
          title={row.execution ? `Execution ${row.ordinal}` : "No Execution Yet"}
        >
          {row.execution ? `#${row.ordinal}` : ""}
        </span>
        {menu ? (
          <DropdownMenu onOpenChange={setMenuOpen} open={menuOpen}>
            <DropdownMenuTrigger asChild>
              <Button
                aria-label={`Actions For ${row.case.label}`}
                className="size-[var(--tree-action-width)] opacity-0 group-hover/row:opacity-100 group-focus-within/row:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100 [[data-density=touch]_&]:pointer-events-none"
                size="icon-sm"
                tabIndex={-1}
                variant="ghost"
              >
                <Ellipsis />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-48" side="right">
              <DropdownMenuItem
                disabled={!startAllowed}
                onSelect={onRun}
                title={startAllowed ? undefined : "Select An Unstarted Execution In An Active Cycle"}
              >
                <Play />
                Start Execution
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                disabled={!deleteAllowed}
                onSelect={() => onDelete(row)}
                title={deleteAllowed ? undefined : reason}
                variant="destructive"
              >
                <Trash2 />
                Delete Case
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ) : (
          <span aria-hidden />
        )}
      </div>
    </li>
  );
}

/** A status as a small coloured glyph, its word as tooltip and as read text. */
function StatusGlyph({ glyph }: { glyph: Glyph }) {
  const Icon = glyph.icon;
  const dot = glyph.icon === Circle;
  return (
    <span className="grid place-items-center" title={glyph.label}>
      <Icon
        aria-hidden
        className={cn(
          TONE_CLASS[glyph.tone],
          dot ? "size-2 fill-current [[data-density=touch]_&]:size-2.5" : "size-3.5 [[data-density=touch]_&]:size-4.5",
          glyph.spin && "animate-spin",
        )}
      />
      <span className="sr-only">{glyph.label}</span>
    </span>
  );
}

function TreeNotice({ action, message, title }: { action?: { label: string; onClick: () => void }; message: string; title: string }) {
  return (
    <li className="grid gap-1.5 py-3 pr-2 pl-[calc(2.5rem+var(--tree-indent))]" role="none">
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

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

/** The selected row's ground, full width; otherwise a hover ground only. */
function rowTone(selected: boolean): string {
  return selected ? "bg-primary/10 text-primary" : "text-foreground/90 hover:bg-muted hover:text-foreground";
}

/** Open a menu after a touch rests on the row; any movement cancels it. */
function useLongPress(onLongPress: () => void) {
  const timer = useRef<number | null>(null);
  const cancel = () => {
    if (timer.current !== null) window.clearTimeout(timer.current);
    timer.current = null;
  };
  return {
    onContextMenu: (event: { preventDefault: () => void }) => {
      event.preventDefault();
      onLongPress();
    },
    onPointerCancel: cancel,
    onPointerDown: (event: { pointerType: string }) => {
      if (event.pointerType !== "touch") return;
      cancel();
      timer.current = window.setTimeout(onLongPress, LONG_PRESS_MS);
    },
    onPointerLeave: cancel,
    onPointerMove: cancel,
    onPointerUp: cancel,
  };
}

function rowGlyph(row: CycleExecutionRow): Glyph {
  if (row.execution) {
    const label = executionStatusLabel(row.execution.status);
    switch (row.execution.status) {
      case "running":
      case "finalizing":
        return { icon: LoaderCircle, label, spin: true, tone: "running" };
      case "waiting_for_prompt":
        return { icon: BellRing, label, tone: "warning" };
      case "passed":
        return { icon: Check, label, tone: "success" };
      case "failed":
        return { icon: X, label, tone: "danger" };
      case "stopped":
        return { icon: Square, label, tone: "muted" };
      case "disconnected":
      case "invalidated":
        return {
          icon: row.execution.status === "disconnected" ? Unplug : TriangleAlert,
          label,
          tone: "warning",
        };
      case "planned":
      case "created":
        return { icon: Circle, label, tone: "muted" };
      default:
        return { icon: Minus, label, tone: "muted" };
    }
  }
  const label = caseStatusLabel(row.case.status);
  switch (row.case.status) {
    case "planned":
    case "queued":
      return { icon: Circle, label, tone: "muted" };
    case "running":
      return { icon: LoaderCircle, label, spin: true, tone: "running" };
    case "passed":
      return { icon: Check, label, tone: "success" };
    case "failed":
      return { icon: X, label, tone: "danger" };
    case "blocked":
    case "invalidated":
      return { icon: TriangleAlert, label, tone: "warning" };
    case "stopped":
      return { icon: Square, label, tone: "muted" };
    default:
      return { icon: Minus, label, tone: "muted" };
  }
}

/** The cycle's own state as a glyph: never an execution's outcome. */
function cycleGlyph(phase: TestCyclePlan["test_cycle"]["phase"], status: CycleLifecycle, waiting: boolean): Glyph {
  const { label } = cycleState(phase, status, waiting);
  switch (label) {
    case "Active · Running":
      return { icon: LoaderCircle, label, spin: true, tone: "running" };
    case "Active · Waiting":
      return { icon: BellRing, label, tone: "warning" };
    case "Sealing":
      return { icon: LoaderCircle, label, spin: true, tone: "warning" };
    case "Sealed":
      return { icon: Check, label, tone: "muted" };
    default:
      return { icon: Circle, label, tone: "muted" };
  }
}

/** How many executions of `caseId` the list holds. */
function caseExecutionCount(rows: readonly CycleExecutionRow[], caseId: string): number {
  return rows.filter((row) => row.case.caseId === caseId && row.execution !== null).length;
}

function liveActions(actions: CycleSidebarActions | undefined): CycleSidebarActions {
  if (!actions) throw new Error("Cycle commands require a live workspace.");
  return actions;
}
