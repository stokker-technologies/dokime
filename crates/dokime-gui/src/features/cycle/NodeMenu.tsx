/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The node's "…" menu: one trigger of one size, the last control of the
 * action cluster on the cycle and the execution viewer. The groups and their
 * order are the model's (`nodeMenuGroups`), so a test can read what each
 * viewer's menu offers without rendering it.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { Archive, ArchiveRestore, Copy, Ellipsis, FileText, Plus, RotateCcw, Settings2 } from "lucide-react";
import { Fragment, type ReactNode } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { type NodeMenuItem, nodeMenuGroups } from "./cycle-workspace-model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

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
  /** Opens the report the cycle has now: current or sealed. */
  onOpenReport: () => void;
  /** Opens the cycle's Settings: title, description, Continue Automatically. */
  onSettings: () => void;
  onToggleFollowActive: () => void;
};

/** The execution page's "…" menu. */
export type ExecutionMenu = {
  kind: "execution";
  /** `null` for a case that has not run, which has no execution to archive. */
  archive: ArchiveAction | null;
  /** Opens the report the execution has now: current or sealed;
   *  `null` for a case that has not run, which has no report of its own. */
  onOpenReport: (() => void) | null;
  /** Re-Run the execution's case; `null` while the attempt has not ended. */
  onRerun: (() => void) | null;
  /** Why the runtime refuses the re-run, when it does. */
  rerunRefusal?: string;
  /** Opens the execution's Settings; `null` for a case that has not run. */
  onSettings: (() => void) | null;
};

/* === Components === */

/** The selected node's own menu, behind one trigger of one size on both pages. */
export function NodeMenu({ menu, children }: { menu: CycleMenu | ExecutionMenu; children?: ReactNode }) {
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
        {children}
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

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

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
    case "Settings":
      return (
        <DropdownMenuItem disabled={!menu.onSettings} onSelect={() => menu.onSettings?.()}>
          <Settings2 />
          Settings
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
