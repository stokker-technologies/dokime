/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The active cycle: shell state the Library adds cases to.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { Search } from "lucide-react";
import { useState } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { CycleStatusPill } from "@/features/cycles-home/CycleStatusPill";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { caseCountLabel, filterOpenCycles, type OpenCycle } from "@/features/library/active-cycle-model";
import { NO_ACTIVE_CYCLE, NO_CHOSEN_CYCLE } from "@/features/library/template-dialog-model";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/ui-store";

/* === Re-Exports === */

/* === Constants === */

/* The picker's columns: name, state, created, identifier, members. Every row
   is a subgrid of the list, so each column keeps one vertical down the list. */
const PICKER_COLUMNS = "grid grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] gap-x-4";

const PICKER_ROW = "col-span-full grid grid-cols-subgrid items-center px-3";

/* === Types === */

type PickerListProps = {
  activeId: string | null;
  cycles: readonly OpenCycle[];
  loading: boolean;
  onChoose: (cycleId: string) => void;
  onNewCycle: () => void;
  onSearch: (text: string) => void;
  search: string;
};

/* === Components === */

/** The active cycle in one line: its name, its short identifier and its state pill. */
export function ActiveCycleSummary({ cycle }: { cycle: OpenCycle }) {
  return (
    <span className="grid min-w-0 grid-cols-[minmax(0,auto)_auto_auto] items-center gap-2">
      <span className="truncate font-medium text-foreground">{cycle.label}</span>
      <span className="font-mono text-muted-foreground tabular-nums">{cycle.shortId}</span>
      <CycleStatusPill label={cycle.statusLabel} tone={cycle.statusTone} />
    </span>
  );
}

/**
 * The template dialog's statement of where its cases go, with Change.
 *
 * Only an empty set of open cycles blocks; open cycles without a choice ask
 * for one through the same picker the shell opens.
 */
export function ActiveCycleInline({
  cycle,
  hasOpenCycles,
  onChange,
  onNewCycle,
}: {
  cycle: OpenCycle | null;
  hasOpenCycles: boolean;
  onChange: () => void;
  onNewCycle: () => void;
}) {
  const action = cycle ? "Change" : hasOpenCycles ? "Choose" : "New Cycle";
  return (
    <div
      className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-lg border bg-muted/50 px-3 py-2 text-sm"
      data-testid="template-active-cycle"
      role="status"
    >
      <span className="text-muted-foreground">Active Cycle</span>
      {cycle ? (
        <ActiveCycleSummary cycle={cycle} />
      ) : (
        <span className="text-muted-foreground">{hasOpenCycles ? NO_CHOSEN_CYCLE : NO_ACTIVE_CYCLE}</span>
      )}
      <Button onClick={cycle || hasOpenCycles ? onChange : onNewCycle} size="sm" type="button" variant="outline">
        {action}
      </Button>
    </div>
  );
}

/** The picker's body: search, the open cycles on one grid, and New Cycle. */
export function ActiveCyclePickerList({ activeId, cycles, loading, onChoose, onNewCycle, onSearch, search }: PickerListProps) {
  const shown = filterOpenCycles(cycles, search);
  return (
    <div className="grid min-h-0 gap-3">
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--bar-gap)]">
        <span className="relative">
          <Search aria-hidden className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            aria-label="Search Open Cycles"
            className="h-[var(--bar-control-height)] pl-8"
            onChange={(event) => onSearch(event.target.value)}
            placeholder="Search By Name Or Identifier"
            value={search}
          />
        </span>
        <Button className="h-[var(--bar-control-height)]" onClick={onNewCycle} type="button">
          New Cycle
        </Button>
      </div>
      {shown.length ? (
        <div className={cn(PICKER_COLUMNS, "max-h-[60vh] overflow-y-auto text-sm")} data-testid="open-cycles">
          <div aria-hidden className={cn(PICKER_ROW, "pb-1 text-xs text-muted-foreground")}>
            <span>Cycle</span>
            <span>State</span>
            <span>Created</span>
            <span>Identifier</span>
            <span className="text-right">Members</span>
          </div>
          {shown.map((cycle) => (
            <button
              aria-pressed={cycle.cycleId === activeId}
              className={cn(
                PICKER_ROW,
                "min-h-[var(--control-height)] rounded-lg py-2 text-left outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
                cycle.cycleId === activeId ? "bg-muted" : null,
              )}
              key={cycle.cycleId}
              onClick={() => onChoose(cycle.cycleId)}
              type="button"
            >
              <span className="truncate font-medium">{cycle.label}</span>
              <CycleStatusPill label={cycle.statusLabel} tone={cycle.statusTone} />
              <span className="font-mono text-xs text-muted-foreground tabular-nums">{cycle.createdLabel}</span>
              <span className="font-mono text-xs text-muted-foreground tabular-nums">{cycle.shortId}</span>
              <span className="text-right font-mono text-xs text-muted-foreground tabular-nums">{caseCountLabel(cycle.caseCount)}</span>
            </button>
          ))}
        </div>
      ) : (
        <p className="text-sm text-muted-foreground">
          {loading ? "Loading Open Cycles" : cycles.length ? "No Open Cycle Matches This Search" : NO_ACTIVE_CYCLE}
        </p>
      )}
    </div>
  );
}

/**
 * The one way the active cycle changes, opened from the shell or the template dialog.
 *
 * Choosing a cycle makes it active and closes the picker; New Cycle creates
 * one and makes it active at once.
 */
export function ActiveCyclePicker() {
  const open = useUiStore((state) => state.activeCyclePickerOpen);
  const setUiState = useUiStore((state) => state.setUiState);
  const active = useActiveCycle();
  const [search, setSearch] = useState("");
  const [creating, setCreating] = useState(false);
  const close = () => {
    setSearch("");
    setUiState({ activeCyclePickerOpen: false });
  };

  return (
    <>
      <Dialog onOpenChange={(next) => (next ? setUiState({ activeCyclePickerOpen: true }) : close())} open={open && !creating}>
        <DialogContent className="sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>Active Cycle</DialogTitle>
            <DialogDescription>Cases added from the Library join the active cycle.</DialogDescription>
          </DialogHeader>
          <ActiveCyclePickerList
            activeId={active.cycle?.cycleId ?? null}
            cycles={active.cycles}
            loading={active.loading}
            onChoose={(cycleId) => {
              active.choose(cycleId);
              close();
            }}
            onNewCycle={() => setCreating(true)}
            onSearch={setSearch}
            search={search}
          />
        </DialogContent>
      </Dialog>
      <NewCycleDialog
        onCreated={(cycleId) => {
          active.choose(cycleId);
          setCreating(false);
          close();
        }}
        onOpenChange={setCreating}
        open={creating}
      />
    </>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
