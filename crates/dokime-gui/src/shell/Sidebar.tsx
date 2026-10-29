/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared Destinations For Desktop Sidebar, Tablet Rail And Phone Tabs. */

import { useNavigate, useRouterState } from "@tanstack/react-router";
import { Bell, History as HistoryIcon, Library, ListChecks, type LucideIcon, Settings } from "lucide-react";
import { type ReactNode, type RefObject, useLayoutEffect, useMemo, useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTitle } from "@/components/ui/sheet";
import { CycleSidebarCases } from "@/features/cycle/CycleSidebarCases";
import { STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import { highestPromptLevel, pendingPromptCountLabel } from "@/features/cycle/prompt-banner-model";
import { cycleRows } from "@/features/cycles-home/cycles-model";
import { useCyclesQuery, usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS } from "@/features/settings/settings-sections";
import { useNotificationSummary } from "@/lib/notifications";
import { recordLevelVariant } from "@/lib/record-display";
import { cycleIdOfPathname } from "@/shell/route-scope";
import { sidebarCycles } from "@/shell/sidebar-cycles";
import { SidebarRow } from "@/shell/sidebar-row";
import { useSidebarState } from "@/shell/use-sidebar-state";
import { SidebarFrame } from "./SidebarFrame";

const DESTINATIONS: Destination[] = [
  { icon: ListChecks, label: "Cycles", prefix: "/cycles", to: "/cycles" },
  { icon: Library, label: "Library", prefix: "/library", to: "/library" },
  { icon: HistoryIcon, label: "History", prefix: "/history", to: "/history" },
  { icon: Bell, label: "Notifications", prefix: "/notifications", to: "/notifications" },
  { icon: Settings, label: "Settings", prefix: "/settings", to: "/settings" },
];

const SETTINGS_SECTION_ROUTE = /^\/settings\/([^/]+)/;

/** The sidebar lists cycles, not their prompts; the Cycles entry counts those. */
const NO_ATTENTION: ReadonlyMap<string, number> = new Map();

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
  to: "/cycles" | "/history" | "/library" | "/notifications" | "/settings";
};

export function Sidebar(props: SidebarProps) {
  const { mode } = useSidebarState();
  const [drawer, setDrawer] = useState(false);
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  if (props.live !== false && mode === "phone")
    return (
      <nav aria-label="Navigation" className="fixed inset-x-0 bottom-0 z-30 grid grid-cols-5 border-t bg-background">
        {DESTINATIONS.map(({ icon: Icon, ...destination }) => (
          <button
            key={destination.to}
            type="button"
            aria-current={isCurrent(pathname, destination.prefix) ? "page" : undefined}
            className="grid min-h-16 justify-items-center content-center gap-1 text-xs aria-[current=page]:text-primary"
            onClick={() => void navigate({ to: destination.to })}
          >
            <Icon className="size-5" />
            {destination.label}
            {destination.to === "/notifications" && <UnreadNotifications />}
          </button>
        ))}
      </nav>
    );
  return (
    <>
      <SidebarFrame>
        {props.live === false ? (
          <ul className="grid min-h-0 content-start gap-1 overflow-y-auto px-[var(--shell-gap)] pt-2 pb-2">
            <CycleSidebarCases selection={props.selection} cycleId={props.cycleId} live={false} onSelect={props.onSelect} />
          </ul>
        ) : (
          <>
            <DestinationTree />
            {mode === "tablet" ? (
              <Button variant="ghost" className="min-h-11" onClick={() => setDrawer(true)}>
                More
              </Button>
            ) : null}
          </>
        )}
      </SidebarFrame>
      {props.live !== false && mode === "tablet" ? (
        <Sheet open={drawer} onOpenChange={setDrawer}>
          <SheetContent
            side="left"
            className="w-80"
            onClick={(event) => {
              if ((event.target as Element).closest("button")) setDrawer(false);
            }}
          >
            <SheetTitle>Navigation</SheetTitle>
            <DestinationTree expanded />
          </SheetContent>
        </Sheet>
      ) : null}
    </>
  );
}

/**
 * The four destinations, the unsealed cycles under Cycles on every route, and
 * the settings sections in the drawer. Live application only.
 */
function DestinationTree({ expanded = false }: { expanded?: boolean }) {
  const navigate = useNavigate();
  const sidebar = useSidebarState();
  const collapsed = sidebar.collapsed && !expanded;
  const railLabels = sidebar.mode === "tablet" && !expanded;
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

  // Every open prompt, whichever cycle asked it: the same read the status bar
  // makes, so the Cycles entry says an operator is needed before they look.
  const pendingPrompts = usePendingPromptsQuery().data ?? [];
  const promptLevel = highestPromptLevel(pendingPrompts);

  // The Cycles home's own read, so the sidebar lists exactly the cycles it does.
  const cycles = useCyclesQuery().data;
  const listedCycles = useMemo(() => sidebarCycles(cycleRows(cycles ?? [], NO_ATTENTION), cycleId), [cycles, cycleId]);

  const activeIndex = DESTINATIONS.findIndex((destination) => isCurrent(pathname, destination.prefix));
  const active = activeIndex >= 0 ? (DESTINATIONS[activeIndex] ?? null) : null;

  const destinationRow = (destination: Destination) => {
    const current = isCurrent(pathname, destination.prefix);
    const row = (
      <SidebarRow
        active={current}
        collapsed={collapsed}
        railLabel={railLabels}
        icon={destination.icon}
        key={destination.to}
        label={destination.label}
        level="destination"
        onSelect={() => void navigate({ to: destination.to })}
        title={destination.label}
      >
        {destination.to === "/notifications" && <UnreadNotifications />}
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
    return row;
  };

  const cycleChildren: ReactNode =
    collapsed || listedCycles.length === 0
      ? null
      : listedCycles.map((cycle) =>
          cycle.current ? (
            <CycleSidebarCases cycleId={cycle.cycleId} key={cycle.cycleId} selection={selection} />
          ) : (
            <SidebarRow
              key={cycle.cycleId}
              label={cycle.label}
              level="overview"
              onSelect={() => void navigate({ params: { cycleId: cycle.cycleId }, to: "/cycles/$cycleId" })}
              title={cycle.label}
            />
          ),
        );

  let settingsChildren: ReactNode = null;
  if (expanded && active?.to === "/settings") {
    settingsChildren = SETTINGS_SECTIONS.map((section) => (
      <SidebarRow
        active={section.id === settingsSection}
        key={section.id}
        label={section.label}
        level="overview"
        onSelect={() => void navigate({ params: { section: section.id }, to: "/settings/$section" })}
        title={section.label}
      />
    ));
  }

  return (
    <ul className="flex min-h-0 flex-col gap-1 px-[var(--shell-gap)] pt-2 pb-2">
      {DESTINATIONS.slice(0, 1).map(destinationRow)}
      {cycleChildren ? (
        <li className="flex min-h-0 flex-col">
          <ChildRegion offsets={scrollOffsets} storageKey={`cycles:${cycleId ?? ""}`}>
            {cycleChildren}
          </ChildRegion>
        </li>
      ) : null}
      {DESTINATIONS.slice(1, 3).map(destinationRow)}
      {settingsChildren ? (
        <li className="flex min-h-0 flex-1 flex-col">
          <ChildRegion offsets={scrollOffsets} storageKey="/settings">
            {settingsChildren}
          </ChildRegion>
        </li>
      ) : (
        <li className="flex-1" />
      )}
      {DESTINATIONS.slice(3).map(destinationRow)}
    </ul>
  );
}

/**
 * A destination's children, and the only part of the tree that scrolls. Its
 * offset is restored when the region comes back so a collapse or a trip to
 * another destination does not send the operator to the top of a long case
 * list.
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
      className="grid min-h-0 content-start gap-1 overflow-y-auto"
      onScroll={(event) => {
        offsets.current[storageKey] = event.currentTarget.scrollTop;
      }}
      ref={ref}
    >
      {children}
    </ul>
  );
}

function isCurrent(pathname: string, prefix: string): boolean {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

function UnreadNotifications() {
  const { unread } = useNotificationSummary();
  return unread ? (
    <Badge aria-label={`${unread} Unread Notifications`} variant="secondary">
      {unread}
    </Badge>
  ) : null;
}
