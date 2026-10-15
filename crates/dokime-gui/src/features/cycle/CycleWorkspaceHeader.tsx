/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle cockpit: the full-width band above the cycle document that says
 * what the cycle is and asks what its run needs.
 *
 * 1. Identity — one row on `--page-bar-height`: the title with its rename
 *    pencil, the status pill and case progress, and the cycle's own "…" menu.
 * 2. Prompts — the questions the run is waiting on, full width, answered in
 *    place.
 *
 * The execution page has no cockpit: its title is the plain header of its
 * sections and its prompts lead its Control section (`ExecutionSections`). It
 * borrows only the node menu from here, so both pages offer one menu of one
 * size. The run controls are not here: the workspace publishes the cycle's
 * into the page bar's action slot, beside Focus. With `live={false}` the
 * cockpit keeps the identity and drops the menu and the prompts, which a file
 * cannot honour.
 */

/* === External Imports === */

import {
  Archive,
  ArchiveRestore,
  Copy,
  Ellipsis,
  FileText,
  Plus,
  RotateCcw,
  Settings2,
} from "lucide-react";
import { type ComponentProps, Fragment, type ReactNode } from "react";

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
import { type NodeMenuItem, nodeMenuGroups, STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { EditableLabel } from "./EditableLabel";

/* === Types === */

/** One node's identity: what the cockpit row states about it. */
export type CockpitIdentity = {
  title: string;
  /** The cycle's lifecycle pill and case progress. The execution row carries
   *  neither: the sidebar glyph states its status and the Steps card its
   *  progress. */
  status?: {
    label: string;
    variant: ComponentProps<typeof Badge>["variant"];
    progress: string;
    /** The case and step the progress stream reports while the node runs. */
    activity?: string | null;
  };
};

/** Archive or Unarchive, as the node's menu offers it. */
export type ArchiveAction = {
  archived: boolean;
  onToggle: () => void;
  /** Why the node cannot be archived yet; archiving waits for a terminal run. */
  refusal?: string;
};

/** The cycle page's "…" menu. */
export type CycleMenu = {
  kind: "cycle";
  archive: ArchiveAction;
  followActive: boolean;
  onAddCases: () => void;
  onCopyCycle: () => void;
  /** Opens the report the cycle has now: current, sealed or regenerated. */
  onOpenReport: () => void;
  onRuntimePolicy: () => void;
  onToggleFollowActive: () => void;
};

/** The execution page's "…" menu. */
export type ExecutionMenu = {
  kind: "execution";
  /** `null` for a case that has not run, which has no execution to archive. */
  archive: ArchiveAction | null;
  /** Opens the report the execution has now: current, sealed or regenerated;
   *  `null` for a case that has not run, which has no report of its own. */
  onOpenReport: (() => void) | null;
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
  /** Renames the node; absent, or with `live={false}`, the title is plain. */
  onRename?: (label: string) => Promise<unknown>;
  /** The prompt band, full width under the identity row. */
  prompts?: ReactNode;
};

/* === Components === */

export function CycleWorkspaceHeader({
  identity,
  live = true,
  menu,
  onRename,
  prompts,
}: CycleWorkspaceHeaderProps) {
  return (
    <section aria-label="Execution Cockpit" className="grid min-w-0 border-b bg-background">
      {/* One row on the shell's bar token: its bottom edge lines up with the
          sidebar brand bar and the page bar at every density, and like them
          it carries no rule of its own. The title gives way first, so the
          progress always reads whole. */}
      <div className="grid min-h-[var(--page-bar-height)] min-w-0 grid-cols-[minmax(0,auto)_auto_minmax(0,1fr)_auto] items-center gap-x-3 px-[var(--shell-inset)] py-[var(--bar-pad-y)]">
        <EditableLabel
          label={identity.title}
          onRename={live && onRename ? onRename : null}
          subject={menu.kind === "cycle" ? "Cycle" : "Execution"}
        />
        {identity.status ? (
          <>
            <Badge className={STATUS_PILL_CLASS} variant={identity.status.variant}>
              {identity.status.label}
            </Badge>
            <p className="min-w-0 truncate text-xs leading-6 text-muted-foreground">
              <span className="whitespace-nowrap tabular-nums">{identity.status.progress}</span>
              {identity.status.activity ? (
                <span aria-live="polite">
                  {" · "}
                  {identity.status.activity}
                </span>
              ) : null}
            </p>
          </>
        ) : (
          <>
            <span />
            <span />
          </>
        )}
        {live ? <NodeMenu menu={menu} /> : <span />}
      </div>

      {live && prompts ? (
        <div className="min-w-0 px-[var(--shell-inset)] pb-3">{prompts}</div>
      ) : null}
    </section>
  );
}

/** The selected node's own menu, behind one trigger of one size on both pages. */
export function NodeMenu({ menu }: { menu: CycleMenu | ExecutionMenu }) {
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
      {/* The groups and their order are the model's, so a test can read
          what each page's menu offers without rendering it. */}
      <DropdownMenuContent align="end" className="w-52">
        {nodeMenuGroups(menu.kind, menu.archive?.archived ?? false).map((group, index) => (
          <Fragment key={group[0]}>
            {index > 0 ? <DropdownMenuSeparator /> : null}
            {group.map((item) => (
              <NodeMenuEntry item={item} key={item} menu={menu} />
            ))}
          </Fragment>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/* === Private Components === */

/** One menu entry, wired to the handler its node supplies. */
function NodeMenuEntry({ item, menu }: { item: NodeMenuItem; menu: CycleMenu | ExecutionMenu }) {
  switch (item) {
    case "Archive":
    case "Unarchive":
      return (
        <DropdownMenuItem
          disabled={!menu.archive || Boolean(menu.archive.refusal)}
          onSelect={() => menu.archive?.onToggle()}
          title={menu.archive?.refusal}
        >
          {item === "Archive" ? <Archive /> : <ArchiveRestore />}
          {item}
        </DropdownMenuItem>
      );
    case "Open Report":
      return (
        <DropdownMenuItem disabled={!menu.onOpenReport} onSelect={() => menu.onOpenReport?.()}>
          <FileText />
          Open Report
        </DropdownMenuItem>
      );
    case "Re-Run":
      if (menu.kind !== "execution") return null;
      return (
        <DropdownMenuItem
          disabled={!menu.onRerun || Boolean(menu.rerunRefusal)}
          onSelect={() => menu.onRerun?.()}
          title={menu.rerunRefusal}
        >
          <RotateCcw />
          Re-Run
        </DropdownMenuItem>
      );
  }
  if (menu.kind !== "cycle") return null;
  switch (item) {
    case "Copy Cycle":
      return (
        <DropdownMenuItem onSelect={menu.onCopyCycle}>
          <Copy />
          Copy Cycle
        </DropdownMenuItem>
      );
    case "Add Cases":
      return (
        <DropdownMenuItem onSelect={menu.onAddCases}>
          <Plus />
          Add Cases
        </DropdownMenuItem>
      );
    case "Runtime Policy":
      return (
        <DropdownMenuItem onSelect={menu.onRuntimePolicy}>
          <Settings2 />
          Runtime Policy
        </DropdownMenuItem>
      );
    case "Follow Active":
      return (
        <DropdownMenuCheckboxItem
          checked={menu.followActive}
          onSelect={(event) => {
            event.preventDefault();
            menu.onToggleFollowActive();
          }}
        >
          Follow Active
        </DropdownMenuCheckboxItem>
      );
  }
}
