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

import { ChevronLeft, MoreHorizontal } from "lucide-react";
// Third Party Imports
import { type ComponentProps, type ReactNode, useEffect } from "react";
// Local Imports
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { usePageChrome } from "@/shell/page-chrome";

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
  /** `alert` for failures the page cannot hide, `status` for quiet notes. */
  tone?: "alert" | "status";
};

/* === Components === */

/**
 * The content page: its breadcrumb and actions in the page bar, the shell inset
 * as gutters and one section gap between everything it holds.
 */
export function PageShell({ actions, children, className, trail, ...props }: PageShellProps) {
  usePageChrome({ actions, title: pageTitle(trail) });
  return (
    <div className={cn("grid grid-cols-12 min-w-0 content-start gap-4 p-4 [&>*]:col-span-12", className)} {...props}>
      {children}
    </div>
  );
}

/** One line about the page's data, at the page's own text inset. */
export function PageNotice({ children, tone = "status" }: PageNoticeProps) {
  return <NotificationMessage tone={tone === "alert" ? "error" : "info"}>{children}</NotificationMessage>;
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
  title: ReactNode;
  actions?: ReactNode;
  overflow?: ReactNode;
  onBack?: () => void;
}) {
  useEffect(() => {
    if (!onBack) return;
    const back = (event: KeyboardEvent) => {
      const target = event.target as Element | null;
      if (
        event.defaultPrevented ||
        target?.closest("[role=dialog], [role=menu], [role=listbox], [data-state=open][data-slot=popover-content]")
      )
        return;
      if (event.key === "Escape" || (event.altKey && event.key === "ArrowLeft")) {
        event.preventDefault();
        onBack();
      }
    };
    window.addEventListener("keydown", back);
    return () => window.removeEventListener("keydown", back);
  }, [onBack]);
  return (
    <header className="flex h-[var(--page-bar-height)] shrink-0 items-center justify-between gap-[var(--bar-gap)] border-b px-[var(--shell-inset)] py-[var(--bar-pad-y)]">
      <div className="flex min-w-0 items-center gap-[var(--shell-gap)]">
        {onBack ? (
          <Button aria-label="Back" size="icon" variant="ghost" onClick={onBack}>
            <ChevronLeft />
          </Button>
        ) : null}
        <h1 className="min-w-0 truncate font-heading font-semibold tracking-wide">{title}</h1>
      </div>
      <div className="flex shrink-0 items-center gap-[var(--bar-gap)]">
        {actions}
        {overflow ? (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button aria-label="Page Actions" size="icon" variant="ghost">
                <MoreHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">{overflow}</DropdownMenuContent>
          </DropdownMenu>
        ) : null}
      </div>
    </header>
  );
}
