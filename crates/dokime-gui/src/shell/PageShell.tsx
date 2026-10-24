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

// Third Party Imports
import type { ComponentProps, ReactNode } from "react";

// Local Imports
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
    <div className={cn("grid min-w-0 content-start gap-[var(--section-gap)] p-[var(--shell-inset)]", className)} {...props}>
      {children}
    </div>
  );
}

/** A filter and controls row with the page bar's own air and gap. */
export function PageToolbar({ children }: { children: ReactNode }) {
  return <div className="flex min-w-0 flex-wrap items-center justify-between gap-[var(--bar-gap)] py-[var(--bar-pad-y)]">{children}</div>;
}

/** Edge-to-edge content, such as a table whose outer cells carry the shell inset themselves. */
export function PageBleed({ children }: { children: ReactNode }) {
  return <div className="-mx-[var(--shell-inset)] min-w-0">{children}</div>;
}

/** One line about the page's data, at the page's own text inset. */
export function PageNotice({ children, tone = "status" }: PageNoticeProps) {
  return (
    <div className={cn("grid gap-1 text-sm", tone === "alert" ? "text-destructive" : "text-muted-foreground")} role={tone}>
      {children}
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

export function pageTitle(trail: string[]): string {
  return trail.join(" \u203a ");
}

/* === Private Functions === */
