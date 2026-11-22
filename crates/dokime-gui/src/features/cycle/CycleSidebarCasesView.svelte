<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import {
    BellRing,
    Check,
    ChevronDown,
    ChevronRight,
    Circle,
    LoaderCircle,
    Minus,
    RotateCcw,
    Square,
    TriangleAlert,
    Unplug,
    X,
  } from "lucide-svelte";
  import type { TestCyclePlan, TestExecution } from "../../lib/types";
  import {
    buildCycleCases,
    buildCycleExecutionRows,
    caseStatusLabel,
    cycleRowSelection,
    cycleState,
    executionStatusLabel,
    isFollowing,
    rowOffersRunAgain,
    sealedCaseCount,
    selectedCycleRow,
    type CycleExecutionRow,
    type CycleLifecycle,
    type CycleSelection,
  } from "./cycle-workspace-model";

  type IconComponent = typeof Circle;

  export type CycleSidebarState = {
    data?: { plan: TestCyclePlan | null; executions: TestExecution[] };
    isError: boolean;
    error: unknown;
    refetch?: () => unknown;
  };

  export type CycleSidebarActions = {
    addCases: () => void;
    rerunCase: (caseId: string) => Promise<string | null | undefined>;
  };

  export type CycleSidebarCasesViewProps = {
    detailQuery: CycleSidebarState;
    selection: CycleSelection;
    onSelect: (selection: CycleSelection) => void;
    /** The executions a pending prompt waits on. */
    inputNeeded?: ReadonlySet<string>;
  } & (
    | { live: true; actions: CycleSidebarActions; followActive: boolean }
    | { live: false; actions?: undefined; followActive?: undefined }
  );

  type GlyphTone = "running" | "success" | "danger" | "warning" | "muted";

  type Glyph = { icon: IconComponent; tone: GlyphTone; spin?: boolean; label: string };

  /**
   * The one row grid: chevron, glyph, label, Run Again, ordinal. Every
   * row uses it, so every trailing column sits on the same x.
   */
  const ROW_GRID =
    "grid h-[var(--tree-row-height)] min-w-0 grid-cols-[--spacing(5)_--spacing(5)_minmax(0,1fr)_var(--tree-action-width)_--spacing(11)] items-center gap-x-1 rounded-lg pr-1";

  const TONE_CLASS: Record<GlyphTone, string> = {
    danger: "text-destructive",
    muted: "text-muted-foreground",
    running: "text-primary",
    success: "text-success",
    warning: "text-warning",
  };

  /** The selected row's ground, full width; otherwise a hover ground only. */
  function rowTone(selected: boolean): string {
    return selected ? "bg-primary/10 text-primary" : "text-foreground/90 hover:bg-muted hover:text-foreground";
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
      case "Seal Failed":
        return { icon: TriangleAlert, label, tone: "danger" };
      case "Sealed":
        return { icon: Check, label, tone: "muted" };
      default:
        return { icon: Circle, label, tone: "muted" };
    }
  }
</script>

<script lang="ts">
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { focusWindowRow, observeRowWindow, rowWindow, ROW_WINDOW_THRESHOLD, type RowWindowMeasurements } from "../../lib/row-window";
  import { cn } from "../../lib/utils";

  let { detailQuery, followActive, actions, inputNeeded, live, onSelect, selection }: CycleSidebarCasesViewProps = $props();
  let expanded = $state(true);
  let list = $state<HTMLUListElement>();
  let measurements = $state.raw<RowWindowMeasurements>({ pitch: null, offset: 0, height: 0 });
  const emptyExecutions: TestExecution[] = [];
  const plan = $derived(detailQuery.data?.plan ?? null);
  const executions = $derived(detailQuery.data?.executions ?? emptyExecutions);
  const cases = $derived(buildCycleCases(plan));
  const rows = $derived(buildCycleExecutionRows(cases, executions));
  const selectedKey = $derived(selectedCycleRow(rows, selection)?.key ?? null);
  const activeExecutionId = $derived(
    plan?.test_cycle.current_test_execution_id ?? executions.find((execution) => execution.status === "running")?.test_execution_id ?? null,
  );
  const following = $derived(live && followActive && isFollowing(selection, activeExecutionId));
  const windowed = $derived(rows.length >= ROW_WINDOW_THRESHOLD);
  const slice = $derived(rowWindow(rows.length, measurements.pitch, measurements.offset, measurements.height));
  const sealed = $derived(sealedCaseCount(cases));

  $effect(() => {
    if (windowed && list) {
      return observeRowWindow(list, (next) => {
        measurements = next;
      });
    }
  });

  async function rerun(row: CycleExecutionRow) {
    if (!actions) {
      return;
    }
    try {
      const created = await actions.rerunCase(row.case.caseId);
      if (created) {
        onSelect({ executionId: created });
      }
      toast.success("New Attempt Ready to Start");
    } catch (error: unknown) {
      toast.error(failureOf(error, "Case Re-Run Failed"));
    }
  }

  function onKeyDown(event: KeyboardEvent & { currentTarget: HTMLUListElement }) {
    const current = event.target;
    if (!(current instanceof HTMLElement) || current.getAttribute("role") !== "treeitem") {
      return;
    }
    const items = [...event.currentTarget.querySelectorAll<HTMLElement>("[role=treeitem]")];
    const index = items.indexOf(current);
    const isCycle = current.dataset.level === "overview";
    function move(target: HTMLElement | undefined) {
      event.preventDefault();
      target?.focus();
    }

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
        if (list && rows.length) {
          event.preventDefault();
          focusWindowRow(list, rows.length - 1, windowed ? measurements.pitch : null);
        } else {
          move(items.at(-1));
        }
        break;
      case "ArrowRight":
        if (isCycle && !expanded) {
          event.preventDefault();
          expanded = true;
        } else if (isCycle) {
          move(items[index + 1]);
        }
        break;
      case "ArrowLeft":
        if (isCycle) {
          event.preventDefault();
          expanded = false;
        } else {
          move(items[0]);
        }
        break;
      case "Enter":
      case " ":
        event.preventDefault();
        current.querySelector<HTMLButtonElement>(":scope > div button[data-tree-select]")?.click();
        break;
      default:
    }
  }
</script>

{#snippet statusGlyph(
  glyph: Glyph,
)}
  {@const Icon = glyph.icon}
  {@const dot = Icon === Circle}
  <span class="grid place-items-center" title={glyph.label}>
    <Icon
      aria-hidden="true"
      class={cn(
        TONE_CLASS[glyph.tone],
        dot ? "size-2 fill-current [[data-density=touch]_&]:size-2.5" : "size-4 [[data-density=touch]_&]:size-4.5",
        glyph.spin && "animate-spin",
      )}
    />
    <span class="sr-only">{glyph.label}</span>
  </span>
{/snippet}

{#snippet notice(
  title: string,
  message: string,
  addCases?: () => void,
)}
  <li class="grid gap-2 py-3 pr-2 pl-[calc(--spacing(10)+var(--tree-indent))]" role="none">
    <p class="font-sans text-xs font-semibold">{title}</p>
    <p class="text-xs leading-snug text-muted-foreground">{message}</p>
    {#if addCases}
      <div class="grid justify-start pt-1"><Button onclick={addCases} size="sm" variant="outline">Add Cases</Button></div>
    {/if}
  </li>
{/snippet}

{#snippet executionRow(
  row: CycleExecutionRow,
  index: number,
  active: boolean,
)}
  {@const selected = row.key === selectedKey}
  {@const isFollowingRow = following && row.execution?.test_execution_id === activeExecutionId}
  {@const waiting = row.execution && inputNeeded?.has(row.execution.test_execution_id)}
  {@const glyph = waiting ? { icon: BellRing, label: "Waiting for Input", tone: "warning" as const } : rowGlyph(row)}
  {@const label = row.rerunOf === null ? row.case.label : `${row.case.label} · Re-Run of #${row.rerunOf}`}
  <li
    aria-label={`${label}, ${glyph.label}`}
    aria-level={2}
    aria-selected={selected}
    class="group/row grid min-w-0 [&:focus-visible>div]:ring-2 [&:focus-visible>div]:ring-ring"
    data-execution-id={row.execution?.test_execution_id}
    data-execution-status={row.execution?.status}
    data-following={isFollowingRow || undefined}
    data-level="case"
    data-rerun-of={row.rerunOf ?? undefined}
    data-window-row={index}
    role="treeitem"
    tabindex={selected ? 0 : -1}
  >
    <div
      class={cn(ROW_GRID, "pl-[var(--tree-indent)]", rowTone(selected), isFollowingRow && "shadow-[inset_2px_0_0_var(--color-primary)]")}
    >
      <span aria-hidden="true"></span>
      {@render statusGlyph(glyph)}
      <Button
        variant="plain"
        size="inline"
        class="min-w-0 truncate text-left text-sm"
        data-tree-select
        onclick={() => onSelect(cycleRowSelection(row))}
        tabindex={-1}
        title={label}
      >
        {row.case.label}
        {#if row.rerunOf !== null}
          <span class="text-xs text-muted-foreground"> Re-Run of #{row.rerunOf}</span>
        {/if}
      </Button>
      {#if live && rowOffersRunAgain(row)}
        <Button
          aria-label={`Run Again ${row.case.label}`}
          class="size-[var(--tree-action-width)]"
          disabled={!active}
          onclick={() => rerun(row)}
          size="icon-sm"
          tabindex={-1}
          title={active ? "Append New Attempt" : "Only Active Cycles Accept New Attempts"}
          variant="ghost"
        >
          <RotateCcw />
        </Button>
      {:else}
        <span aria-hidden="true"></span>
      {/if}
      <span
        class="text-right text-xs text-muted-foreground tabular-nums"
        title={row.execution ? `Execution ${row.ordinal}` : "No Execution Yet"}
      >
        {row.execution ? `#${row.ordinal}` : ""}
      </span>
    </div>
  </li>
{/snippet}

<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role (Tree Items Are List Rows) -->
<ul aria-label="Cycle Executions" class="grid min-w-0 content-start gap-px" onkeydown={onKeyDown} role="tree">
  {#if plan}
    {@const cycle = plan.test_cycle}
    {@const selected = selectedKey === null}
    <li
      aria-expanded={expanded}
      aria-label={`${cycle.title}, ${cycleState(cycle.phase, cycle.status, false).label}`}
      aria-level={1}
      aria-selected={selected}
      class="grid min-w-0 gap-px [&:focus-visible>div]:ring-2 [&:focus-visible>div]:ring-ring"
      data-level="overview"
      role="treeitem"
      tabindex={selected ? 0 : -1}
    >
      <div class={cn(ROW_GRID, rowTone(selected))}>
        <Button
          variant="plain"
          size="inline"
          aria-label={expanded ? "Collapse Executions" : "Expand Executions"}
          class="grid size-5 place-items-center rounded-lg text-muted-foreground hover:text-foreground"
          onclick={() => {
            expanded = !expanded;
          }}
          tabindex={-1}
        >
          {#if expanded}
            <ChevronDown class="size-4" />
          {:else}
            <ChevronRight class="size-4" />
          {/if}
        </Button>
        {@render statusGlyph(cycleGlyph(cycle.phase, cycle.status, (inputNeeded?.size ?? 0) > 0))}
        <Button
          variant="plain"
          size="inline"
          class="min-w-0 truncate text-left text-sm font-medium"
          data-tree-select
          onclick={() => onSelect({ view: "cycle" })}
          tabindex={-1}
          title={cycle.title}
        >
          {cycle.title}
        </Button>
        <span aria-hidden="true"></span>
        <span class="text-right text-xs text-muted-foreground tabular-nums" title={`${sealed} of ${cases.length} Cases Sealed`}
          >{sealed}/{cases.length}</span
        >
      </div>
      {#if expanded}
        <ul class="grid min-w-0 gap-px" bind:this={list} role="group">
          {#if detailQuery.isError}
            <li role="none">
              <ErrorCard failure={failureOf(detailQuery.error, "Cases Unavailable")} onRetry={() => detailQuery.refetch?.()} />
            </li>
          {:else if !rows.length}
            {@render notice("No Cases", "This cycle has no cases yet.", live ? actions?.addCases : undefined)}
          {:else}
            {#if slice.before}
              <li aria-hidden="true" style:height={`${slice.before}px`}></li>
            {/if}
            {#each rows.slice(slice.first, slice.last) as row, offset (row.key)}
              {@render executionRow(row, slice.first + offset, cycle.phase === "active")}
            {/each}
            {#if slice.after}
              <li aria-hidden="true" style:height={`${slice.after}px`}></li>
            {/if}
          {/if}
        </ul>
      {/if}
    </li>
  {:else if detailQuery.isError}
    <li role="none"><ErrorCard failure={failureOf(detailQuery.error, "Cycle Unavailable")} onRetry={() => detailQuery.refetch?.()} /></li>
  {:else}
    {@render notice("Loading", "Reading this cycle's cases.")}
  {/if}
</ul>
