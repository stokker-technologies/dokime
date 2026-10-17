/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one card every section of the execution page is drawn in.
 *
 * Same border, padding, header and empty state everywhere on the page, so the
 * sections read as one column of equals. The header is one row on a fixed
 * height whether or not it carries an action, so titles share a baseline from
 * card to card. A section with nothing to show yet states it in the same
 * dashed well, below whatever controls it keeps live (Add a note, capture).
 */

/* === External Imports === */

import { type ReactNode, useId } from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Types === */

export type ExecutionSectionCardProps = {
  title: string;
  /** A control on the header row, right-aligned: the plot window, a download. */
  action?: ReactNode;
  children?: ReactNode;
  className?: string;
  description?: ReactNode;
  /** What the section says while it has nothing to show; `null` when it does. */
  empty?: string | null;
};

/* === Components === */

export function ExecutionSectionCard({ title, action, children, className, description, empty = null }: ExecutionSectionCardProps) {
  const titleId = useId();

  return (
    <section
      aria-labelledby={titleId}
      className={cn("grid min-w-0 gap-3 rounded-lg border bg-card p-4", className)}
      data-execution-section={title}
    >
      <header className="grid min-h-8 min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1">
        <h2 className="truncate font-heading text-base leading-6 font-semibold" id={titleId}>
          {title}
        </h2>
        {action ? <div className="grid auto-cols-max grid-flow-col items-center">{action}</div> : <span />}
        {description ? <div className="col-span-2 text-sm text-muted-foreground [overflow-wrap:anywhere]">{description}</div> : null}
      </header>
      {children}
      {empty ? <p className="rounded-md border border-dashed px-3 py-4 text-center text-sm text-muted-foreground">{empty}</p> : null}
    </section>
  );
}
