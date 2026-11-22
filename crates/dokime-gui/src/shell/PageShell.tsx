/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * One Page Shell For Content Pages
 */

/* === Imports === */

// Standard Library Imports

import { MoreHorizontal } from "lucide-react";
// Third Party Imports
import { type ComponentProps, type ReactNode } from "react";
// Local Imports
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { SvelteMount, useSvelteSlots } from "../components/SvelteMount";
import PageHeader from "./PanelHeader.svelte";
import { usePageChrome } from "./page-chrome";
import { pageContentClass } from "./page-layout";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type PageShellProps = Omit<ComponentProps<"div">, "children" | "className" | "title"> & {
  /** The breadcrumb, outermost first; the page bar shows it joined with chevrons. */
  trail: string[];
  /** The page bar's single action slot. Memoize it; see `usePageChrome`. */
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
};

type PageNoticeProps = {
  children: ReactNode;
};

/* === Components === */

/**
 * The content page: its breadcrumb and actions in the page bar, the shell inset
 * as gutters and one section gap between everything it holds.
 */
export function PageShell({ actions, children, className, trail, ...props }: PageShellProps) {
  usePageChrome({ actions, title: pageTitle(trail) });
  return (
    <div className={cn(pageContentClass, className)} {...props}>
      {children}
    </div>
  );
}

/** One line about the page's data, at the page's own text inset. */
export function PageNotice({ children }: PageNoticeProps) {
  return (
    <p role="status" className="text-sm text-muted-foreground">
      {children}
    </p>
  );
}

/* === Hooks === */

/* === Public Functions === */

export function pageTitle(trail: string[]): string {
  return trail.join(" \u203a ");
}

/* === Private Functions === */

export function PanelHeader({
  title,
  actions,
  overflow,
  onBack,
}: {
  title: string;
  actions?: ReactNode;
  overflow?: ReactNode;
  onBack?: () => void;
}) {
  const menu = overflow ? (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button aria-label="Page Actions" size="icon" variant="ghost">
          <MoreHorizontal />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">{overflow}</DropdownMenuContent>
    </DropdownMenu>
  ) : null;
  const { rendered, portals } = useSvelteSlots({
    actions: actions || menu ? <>{actions}{menu}</> : null,
  });
  return (
    <>
      <SvelteMount component={PageHeader} props={{ title, onBack, ...rendered }} />
      {portals}
    </>
  );
}
