/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one sidebar.
 *
 * A tree, not a rail beside another rail: the four destinations stay reachable
 * and the one the operator is in opens its children underneath at the same row
 * height and on the same three columns. Only the child region scrolls — the
 * destinations above it and below it are pinned — so a cycle with forty cases
 * never pushes Settings out of the window. Collapsed, it keeps the
 * destinations as icons and each tooltip states how many children are waiting
 * behind them, and the child region's scroll position comes back with them.
 * The collapse toggle is not here: it sits in the status bar's first cell, on
 * the sidebar's own column.
 *
 * With `live={false}` the same tree carries a frozen report. A report is one
 * cycle and nothing else, so the destinations — which would navigate to
 * surfaces the file does not contain — give way to that cycle's outline, and
 * the mark above it stays exactly where it is.
 */

/* === External Imports === */

import { useNavigate, useRouterState } from "@tanstack/react-router";
import { History as HistoryIcon, Library, ListChecks, type LucideIcon, Settings } from "lucide-react";
import { type ReactNode, type RefObject, useLayoutEffect, useRef } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { CycleSidebarCases } from "@/features/cycle/CycleSidebarCases";
import { useCycleDetailQuery } from "@/features/cycle/cycle-queries";
import { STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import { highestPromptLevel, pendingPromptCountLabel } from "@/features/cycle/prompt-banner-model";
import { usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS } from "@/features/settings/settings-sections";
import { recordLevelVariant } from "@/lib/record-display";
import { cn } from "@/lib/utils";
import { cycleIdOfPathname } from "@/shell/route-scope";
import { SidebarRow } from "@/shell/sidebar-row";
import { useSidebarState } from "@/shell/use-sidebar-state";
import { SidebarFrame } from "./SidebarFrame";

/* === Types === */

export type SidebarProps =
  | { live?: true }
  | {
      selection: { caseId?: string; executionId?: string };
      cycleId: string;
      live: false;
      onSelect: (selection: { caseId?: string; executionId?: string }) => void;
    };

type Destination = {
  icon: LucideIcon;
  label: string;
  /** Every path under this prefix marks the destination current. */
  prefix: string;
  to: "/cycles" | "/history" | "/library" | "/settings";
};

/* === Constants === */

const DESTINATIONS: Destination[] = [
  { icon: ListChecks, label: "Cycles", prefix: "/cycles", to: "/cycles" },
  { icon: Library, label: "Library", prefix: "/library", to: "/library" },
  { icon: HistoryIcon, label: "History", prefix: "/history", to: "/history" },
  { icon: Settings, label: "Settings", prefix: "/settings", to: "/settings" },
];

const SETTINGS_SECTION_ROUTE = /^\/settings\/([^/]+)/;

/* === Components === */

export function Sidebar(props: SidebarProps) {
  return (
    <SidebarFrame>
      {props.live === false ? (
        <ul className="grid min-h-0 content-start gap-0.5 overflow-y-auto px-[var(--shell-gap)] pt-2 pb-2">
          <CycleSidebarCases selection={props.selection} cycleId={props.cycleId} live={false} onSelect={props.onSelect} />
        </ul>
      ) : (
        <DestinationTree />
      )}
    </SidebarFrame>
  );
}

/* === Private Components === */

/** The four destinations and the active one's children. Live application only. */
function DestinationTree() {
  const navigate = useNavigate();
  const { collapsed } = useSidebarState();
  const { selection, pathname } = useRouterState({
    select: (state) => ({
      selection: state.location.search as { caseId?: string; executionId?: string },
      pathname: state.location.pathname,
    }),
  });

  // Collapsing unmounts the children, so the browser forgets where they were
  // scrolled to. The offset is kept here, where the sidebar itself survives.
  const scrollOffsets = useRef<Record<string, number>>({});

  const cycleId = cycleIdOfPathname(pathname);
  const onSettings = isCurrent(pathname, "/settings");
  const settingsSection = onSettings ? (SETTINGS_SECTION_ROUTE.exec(pathname)?.[1] ?? DEFAULT_SETTINGS_SECTION) : null;

  // The one read is the shared query of the surface it describes, so the tree
  // adds no traffic beyond the destination the operator is actually in.
  const cycleDetail = useCycleDetailQuery(cycleId ?? "", cycleId !== null);
  const caseCount = cycleDetail.data?.plan?.cases.length ?? 0;

  // Every open prompt, whichever cycle asked it: the same read the status bar
  // makes, so the Cycles entry says an operator is needed before they look.
  const pendingPrompts = usePendingPromptsQuery().data ?? [];
  const promptLevel = highestPromptLevel(pendingPrompts);

  const childCount = (destination: Destination): number => {
    if (destination.to === "/settings") return SETTINGS_SECTIONS.length;
    if (destination.to === "/cycles") return cycleId ? caseCount : 0;
    return 0;
  };

  const activeIndex = DESTINATIONS.findIndex((destination) => isCurrent(pathname, destination.prefix));
  const active = activeIndex >= 0 ? (DESTINATIONS[activeIndex] ?? null) : null;

  const destinationRow = (destination: Destination) => {
    const current = isCurrent(pathname, destination.prefix);
    const row = (
      <SidebarRow
        active={current}
        collapsed={collapsed}
        icon={destination.icon}
        key={destination.to}
        label={destination.label}
        level="destination"
        onSelect={() => void navigate({ to: destination.to })}
        title={destination.label}
      >
        {destination.to === "/cycles" && promptLevel ? (
          <Badge
            aria-label={pendingPromptCountLabel(pendingPrompts.length)}
            className={STATUS_PILL_CLASS}
            data-testid="navigation-prompt-count"
            variant={recordLevelVariant(promptLevel)}
          >
            {pendingPrompts.length}
          </Badge>
        ) : null}
      </SidebarRow>
    );
    if (!collapsed) return row;
    return (
      <Tooltip key={destination.to}>
        <TooltipTrigger asChild>{row}</TooltipTrigger>
        <TooltipContent side="right">
          {childCountLabel(destination, childCount(destination))}
          {destination.to === "/cycles" && pendingPrompts.length > 0 ? ` · ${pendingPromptCountLabel(pendingPrompts.length)}` : null}
        </TooltipContent>
      </Tooltip>
    );
  };

  let children: ReactNode = null;
  if (!collapsed && active?.to === "/settings") {
    children = SETTINGS_SECTIONS.map((section) => (
      <SidebarRow
        active={section.id === settingsSection}
        key={section.id}
        label={section.label}
        level="overview"
        onSelect={() => void navigate({ params: { section: section.id }, to: "/settings/$section" })}
        title={section.label}
      />
    ));
  } else if (!collapsed && active?.to === "/cycles" && cycleId) {
    children = <CycleSidebarCases selection={selection} cycleId={cycleId} />;
  }

  return (
    <ul className="flex min-h-0 flex-col gap-0.5 px-[var(--shell-gap)] pt-2 pb-2">
      {DESTINATIONS.slice(0, Math.max(activeIndex, 0)).map(destinationRow)}

      {active ? (
        <li className={cn("flex flex-col gap-0.5", children ? "min-h-0" : "shrink-0")}>
          <ul className="shrink-0">{destinationRow(active)}</ul>
          {children ? (
            <ChildRegion offsets={scrollOffsets} storageKey={active.to === "/cycles" ? `cycles:${cycleId}` : active.to}>
              {children}
            </ChildRegion>
          ) : null}
        </li>
      ) : null}

      {DESTINATIONS.slice(activeIndex + 1).map(destinationRow)}
    </ul>
  );
}

/**
 * The active destination's children, and the only thing in the tree that
 * scrolls. Its offset is restored when the region comes back so a collapse or
 * a trip to another destination does not send the operator to the top of a
 * long case list.
 */
function ChildRegion({
  children,
  offsets,
  storageKey,
}: {
  children: ReactNode;
  offsets: RefObject<Record<string, number>>;
  storageKey: string;
}) {
  const ref = useRef<HTMLUListElement>(null);

  useLayoutEffect(() => {
    const node = ref.current;
    if (node) node.scrollTop = offsets.current[storageKey] ?? 0;
  }, [offsets, storageKey]);

  return (
    <ul
      className="grid min-h-0 content-start gap-0.5 overflow-y-auto"
      onScroll={(event) => {
        offsets.current[storageKey] = event.currentTarget.scrollTop;
      }}
      ref={ref}
    >
      {children}
    </ul>
  );
}

/* === Private Functions === */

function isCurrent(pathname: string, prefix: string): boolean {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

/** The collapsed tooltip has to say what the hidden children are. */
function childCountLabel(destination: Destination, count: number): string {
  if (count === 0) return destination.label;
  const noun = destination.to === "/settings" ? "section" : "case";
  return `${destination.label} — ${count} ${noun}${count === 1 ? "" : "s"}`;
}
