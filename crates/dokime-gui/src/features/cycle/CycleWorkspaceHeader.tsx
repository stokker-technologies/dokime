/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The execution cockpit: the full-width band above the document that says
 * what the selected node is and asks what its run needs.
 *
 * 1. Identity — one row on `--page-bar-height` with one anatomy on both
 *    pages: title, status pill, progress, and the node's own "…" menu. The
 *    cycle page names the cycle, its case progress and the cycle menu; the
 *    execution page names the execution, its step progress and the execution
 *    menu. Nothing of one node appears on the other's page.
 * 2. Prompts — the questions the run is waiting on, full width, answered in
 *    place.
 *
 * The run controls are not here: the workspace publishes the cycle's or the
 * selected execution's into the page bar's action slot, beside Focus. With
 * `live={false}` the cockpit keeps the identity and drops the menu and the
 * prompts, which a file cannot honour.
 */

/* === External Imports === */

import {
  Copy,
  Download,
  Ellipsis,
  FileText,
  Plus,
  RotateCcw,
  Settings2,
  Trash2,
} from "lucide-react";
import { type ComponentProps, type ReactNode, useId } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SEALED_EVIDENCE_PENDING, STATUS_PILL_CLASS } from "./cycle-workspace-model";

/* === Types === */

/** One node's identity: what the cockpit row states about it. */
export type CockpitIdentity = {
  title: string;
  statusLabel: string;
  statusVariant: ComponentProps<typeof Badge>["variant"];
  progress: string;
  /** The case and step the progress stream reports while the node runs. */
  activity?: string | null;
};

/** The cycle page's "…" menu. */
export type CycleMenu = {
  kind: "cycle";
  followActive: boolean;
  /** Bundle and report are cut from the sealed cycle; in flight they wait. */
  evidencePending: boolean;
  onAddCases: () => void;
  onClearCycle: () => void;
  onCopyCycle: () => void;
  onDownloadBundle: () => void;
  onRuntimePolicy: () => void;
  onToggleFollowActive: () => void;
  onWriteReport: () => void;
};

/** The execution page's "…" menu. */
export type ExecutionMenu = {
  kind: "execution";
  /** Re-Run the execution's case; `null` while the attempt has not ended. */
  onRerun: (() => void) | null;
  /** Why the runtime refuses the re-run, when it does. */
  rerunRefusal?: string;
};

export type CycleWorkspaceHeaderProps = {
  identity: CockpitIdentity;
  /** False in the frozen report: no menu, no prompts. */
  live?: boolean;
  menu: CycleMenu | ExecutionMenu;
  /** The prompt band, full width under the identity row. */
  prompts?: ReactNode;
};

/* === Components === */

export function CycleWorkspaceHeader({
  identity,
  live = true,
  menu,
  prompts,
}: CycleWorkspaceHeaderProps) {
  return (
    <section aria-label="Execution Cockpit" className="grid min-w-0 border-b bg-background">
      {/* One row on the shell's bar token: its bottom edge lines up with the
          sidebar brand bar and the page bar at every density, and like them
          it carries no rule of its own. The title gives way first, so the
          progress always reads whole. */}
      <div className="grid min-h-[var(--page-bar-height)] min-w-0 grid-cols-[minmax(0,auto)_auto_minmax(0,1fr)_auto] items-center gap-x-3 px-[var(--shell-inset)] py-[var(--bar-pad-y)]">
        <h1
          className="min-w-0 truncate font-heading text-lg leading-7 font-semibold"
          title={identity.title}
        >
          {identity.title}
        </h1>
        <Badge className={STATUS_PILL_CLASS} variant={identity.statusVariant}>
          {identity.statusLabel}
        </Badge>
        <p className="min-w-0 truncate text-xs leading-6 text-muted-foreground">
          <span className="whitespace-nowrap tabular-nums">{identity.progress}</span>
          {identity.activity ? (
            <span aria-live="polite">
              {" · "}
              {identity.activity}
            </span>
          ) : null}
        </p>
        {live ? <NodeMenu menu={menu} /> : <span />}
      </div>

      {live && prompts ? (
        <div className="min-w-0 px-[var(--shell-inset)] pb-3">{prompts}</div>
      ) : null}
    </section>
  );
}

/* === Private Components === */

/** The selected node's own menu, behind one trigger of one size on both pages. */
function NodeMenu({ menu }: { menu: CycleMenu | ExecutionMenu }) {
  const evidencePendingId = useId();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          aria-label={menu.kind === "cycle" ? "Cycle Actions" : "Execution Actions"}
          className="size-[var(--bar-control-height)]"
          size="icon"
          variant="outline"
        >
          <Ellipsis />
        </Button>
      </DropdownMenuTrigger>
      {menu.kind === "cycle" ? (
        <DropdownMenuContent align="end" className="w-52">
          <DropdownMenuItem onSelect={menu.onCopyCycle}>
            <Copy />
            Copy Cycle
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={menu.onAddCases}>
            <Plus />
            Add Cases
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={menu.onRuntimePolicy}>
            <Settings2 />
            Runtime Policy
          </DropdownMenuItem>
          <DropdownMenuCheckboxItem
            checked={menu.followActive}
            onSelect={(event) => {
              event.preventDefault();
              menu.onToggleFollowActive();
            }}
          >
            Follow Active
          </DropdownMenuCheckboxItem>
          <DropdownMenuSeparator />
          {/* A disabled menu item takes no pointer, so a hover title alone
              would never show; the reason is visible text both items name. */}
          <DropdownMenuItem
            aria-describedby={menu.evidencePending ? evidencePendingId : undefined}
            disabled={menu.evidencePending}
            onSelect={menu.onDownloadBundle}
            title={menu.evidencePending ? SEALED_EVIDENCE_PENDING : undefined}
          >
            <Download />
            Download Bundle
          </DropdownMenuItem>
          <DropdownMenuItem
            aria-describedby={menu.evidencePending ? evidencePendingId : undefined}
            disabled={menu.evidencePending}
            onSelect={menu.onWriteReport}
            title={menu.evidencePending ? SEALED_EVIDENCE_PENDING : undefined}
          >
            <FileText />
            Write Report
          </DropdownMenuItem>
          {menu.evidencePending ? (
            <DropdownMenuLabel className="font-normal" id={evidencePendingId}>
              {SEALED_EVIDENCE_PENDING}
            </DropdownMenuLabel>
          ) : null}
          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={menu.onClearCycle} variant="destructive">
            <Trash2 />
            Clear Cycle
          </DropdownMenuItem>
        </DropdownMenuContent>
      ) : (
        <DropdownMenuContent align="end" className="w-52">
          <DropdownMenuItem
            disabled={!menu.onRerun || Boolean(menu.rerunRefusal)}
            onSelect={() => menu.onRerun?.()}
            title={menu.rerunRefusal}
          >
            <RotateCcw />
            Re-Run
          </DropdownMenuItem>
        </DropdownMenuContent>
      )}
    </DropdownMenu>
  );
}
