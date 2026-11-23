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
import { type ComponentProps, type ReactNode } from "react";
// Local Imports
import { cn } from "@/lib/utils";
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

export function PageNotice({ children }: { children: ReactNode }) {
  return (
    <p role="status" className="text-sm text-muted-foreground">
      {children}
    </p>
  );
}

export function pageTitle(trail: string[]): string {
  return trail.join(" \u203a ");
}
