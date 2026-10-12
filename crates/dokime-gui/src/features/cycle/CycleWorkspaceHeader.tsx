/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle workspace's fixed header: what this cycle is, how far it has got,
 * and every command that acts on the cycle as a whole.
 *
 * One grid owns the whole bar — title block, progress, Cases, run controls,
 * overflow — so the pieces keep their columns whatever the data says. The bar
 * stays above the evidence scroll and outside the Cases panel, so Stop remains
 * reachable while that panel or a prompt form is open.
 *
 * With `live={false}` the bar keeps the identity, the status and the progress
 * — everything a reader needs to know what they are looking at — and drops the
 * run controls and the overflow, which a file cannot honour.
 */

/* === External Imports === */

import {
  Copy,
  Download,
  Ellipsis,
  FileText,
  FolderOpen,
  ListChecks,
  Pause,
  Play,
  Plus,
  Settings2,
  Square,
  Trash2,
} from "lucide-react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { useSidebarState } from "@/shell/use-sidebar-state";
import { CycleSidebarCases } from "./CycleSidebarCases";
import {
  type CycleLifecycle,
  cycleStatusLabel,
  cycleStatusVariant,
  type RunControlAvailability,
  type RunControlVisibility,
  STATUS_PILL_CLASS,
} from "./cycle-workspace-model";

/* === Constants === */

/** One height and one measure for every control in the bar. */
const CONTROL_CLASS = "h-9 w-full [[data-density=touch]_&]:h-14";

/**
 * One slot per run control, whether or not the lifecycle fills it.
 *
 * The slots keep their width so the title, the progress and the overflow menu
 * do not shift sideways the moment a cycle starts or finishes.
 */
const SLOT_CLASS = "w-32 [[data-density=touch]_&]:w-44";

/* === Types === */

export type CycleWorkspaceHeaderProps = {
  availability: RunControlAvailability;
  /** False in the frozen report: no run controls, no overflow menu. */
  live?: boolean;
  /** The case and step the progress stream reports while the cycle runs. */
  activity: string | null;
  caseTotal: number;
  /** The selected case, so the Cases sheet opens on the operator's place. */
  caseId?: string | undefined;
  /** The open cycle. Required for the Cases sheet a collapsed sidebar needs. */
  cycleId?: string;
  finishedCases: number;
  /** When on, a new attempt of the selected case becomes the shown evidence. */
  followActive: boolean;
  label: string;
  /** Each control waits only on its own command, so Stop never waits on another. */
  pending: RunControlAvailability;
  /** Why the runtime refuses every queued case, when it does. */
  runRefusal?: string | null;
  /** Hidden for a remote backend, which has no folder this machine can open. */
  showOpenFolder: boolean;
  status: CycleLifecycle;
  /** Which of the three run-control slots the lifecycle fills. */
  visibility: RunControlVisibility;
  onAddCases: () => void;
  onClearCycle: () => void;
  onCopyCycle: () => void;
  onDownloadBundle: () => void;
  onOpenFolder: () => void;
  onPause: () => void;
  onResume: () => void;
  onRunQueued: () => void;
  onRuntimePolicy: () => void;
  onStop: () => void;
  onToggleFollowActive: () => void;
  onWriteReport: () => void;
};

/* === Components === */

export function CycleWorkspaceHeader({
  availability,
  activity,
  live = true,
  caseTotal,
  caseId,
  cycleId,
  finishedCases,
  followActive,
  label,
  pending,
  runRefusal,
  showOpenFolder,
  status,
  visibility,
  onAddCases,
  onClearCycle,
  onCopyCycle,
  onDownloadBundle,
  onOpenFolder,
  onPause,
  onResume,
  onRunQueued,
  onRuntimePolicy,
  onStop,
  onToggleFollowActive,
  onWriteReport,
}: CycleWorkspaceHeaderProps) {
  return (
    <header className="grid min-h-16 min-w-0 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] items-center gap-x-4 border-b bg-background px-[var(--shell-inset)] py-2 [[data-density=touch]_&]:min-h-20">
      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-3">
        <h1 className="truncate font-heading text-lg leading-6 font-semibold" title={label}>
          {label}
        </h1>
        <Badge className={STATUS_PILL_CLASS} variant={cycleStatusVariant(status)}>
          {cycleStatusLabel(status)}
        </Badge>
      </div>

      {/* Collapsed, the sidebar's case rows are behind a rail the operator has
          to restore to reach. At touch that is the only route to a case, so
          the header offers the same rows in a sheet instead. */}
      {live && cycleId ? <CasesSheet caseId={caseId} cycleId={cycleId} /> : null}

      <div className="hidden grid-flow-col items-center gap-3 text-xs leading-6 text-muted-foreground sm:grid">
        <span className="tabular-nums">
          {finishedCases} of {caseTotal} cases finished
        </span>
        {activity ? (
          <span aria-live="polite" className="hidden max-w-80 truncate lg:inline">
            {activity}
          </span>
        ) : null}
      </div>

      {/* A lifecycle that offers nothing shows nothing: the slots exist to
          keep the group stable while a cycle runs, not to reserve room for
          buttons a finished cycle can never regain. */}
      {live && (visibility.run || visibility.pauseResume || visibility.stop) ? (
        <fieldset className="grid grid-flow-col items-center gap-2">
          <legend className="sr-only">Run Controls</legend>
          <div className={SLOT_CLASS}>
            {visibility.run ? (
              <Button
                className={CONTROL_CLASS}
                disabled={pending.run || !availability.run}
                onClick={onRunQueued}
                size="lg"
                title={availability.run ? undefined : (runRefusal ?? undefined)}
                variant="default"
              >
                <Play data-icon="inline-start" />
                Run Queued
              </Button>
            ) : null}
          </div>
          <div className={SLOT_CLASS}>
            {visibility.pauseResume === "pause" ? (
              <Button
                className={CONTROL_CLASS}
                disabled={pending.pause || !availability.pause}
                onClick={onPause}
                size="lg"
                variant="outline"
              >
                <Pause data-icon="inline-start" />
                Pause
              </Button>
            ) : visibility.pauseResume === "resume" ? (
              <Button
                className={CONTROL_CLASS}
                disabled={pending.resume || !availability.resume}
                onClick={onResume}
                size="lg"
                variant="outline"
              >
                <Play data-icon="inline-start" />
                Resume
              </Button>
            ) : null}
          </div>
          <div className={SLOT_CLASS}>
            {visibility.stop ? (
              <Button
                className={CONTROL_CLASS}
                disabled={pending.stop || !availability.stop}
                onClick={onStop}
                size="lg"
                variant="danger"
              >
                <Square data-icon="inline-start" />
                {status === "stopping" ? "Stopping" : "Stop"}
              </Button>
            ) : null}
          </div>
        </fieldset>
      ) : (
        <span />
      )}

      {live ? (
        <div className="grid grid-flow-col items-center gap-2">
          <Separator className="h-6" orientation="vertical" />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                aria-label="Cycle Actions"
                className="size-9 [[data-density=touch]_&]:size-14"
                size="icon-lg"
                variant="outline"
              >
                <Ellipsis />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-52">
              <DropdownMenuItem onSelect={onCopyCycle}>
                <Copy />
                Copy Cycle
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={onAddCases}>
                <Plus />
                Add Cases
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={onRuntimePolicy}>
                <Settings2 />
                Runtime Policy
              </DropdownMenuItem>
              <DropdownMenuCheckboxItem
                checked={followActive}
                onSelect={(event) => {
                  event.preventDefault();
                  onToggleFollowActive();
                }}
              >
                Follow Active
              </DropdownMenuCheckboxItem>
              <DropdownMenuSeparator />
              {showOpenFolder ? (
                <DropdownMenuItem onSelect={onOpenFolder}>
                  <FolderOpen />
                  Open Folder
                </DropdownMenuItem>
              ) : null}
              <DropdownMenuItem onSelect={onDownloadBundle}>
                <Download />
                Download Bundle
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={onWriteReport}>
                <FileText />
                Write Report
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onSelect={onClearCycle} variant="destructive">
                <Trash2 />
                Clear Cycle
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      ) : (
        <span />
      )}
    </header>
  );
}

/* === Private Components === */

/**
 * The cycle's cases, reachable while the sidebar is a rail.
 *
 * The sheet holds the sidebar's own rows — the same component, the same
 * selection, the same pills — so there is one list of a cycle's cases and this
 * is a second door to it, not a second copy of it.
 */
function CasesSheet({ caseId, cycleId }: { caseId: string | undefined; cycleId: string }) {
  const { collapsed } = useSidebarState();
  if (!collapsed) return null;

  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button className={CONTROL_CLASS} size="lg" variant="outline">
          <ListChecks data-icon="inline-start" />
          Cases
        </Button>
      </SheetTrigger>
      <SheetContent className="w-[22rem] max-w-full" side="left">
        <SheetHeader>
          <SheetTitle>Cases</SheetTitle>
          <SheetDescription>Select a case to show its evidence.</SheetDescription>
        </SheetHeader>
        <ul className="grid min-h-0 content-start gap-0.5 overflow-y-auto px-2 pb-4">
          <CycleSidebarCases caseId={caseId} cycleId={cycleId} />
        </ul>
      </SheetContent>
    </Sheet>
  );
}
