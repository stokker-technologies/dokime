/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Types === */

type SettingsCardProps = {
  children: ReactNode;
  actions?: ReactNode;
  className?: string;
  description?: ReactNode;
  footer?: ReactNode;
  title?: ReactNode;
};

/* === Components === */

/** A grouped settings card: an optional header, a divided body of rows, and an optional footer. */
export function SettingsCard({ actions, children, className, description, footer, title }: SettingsCardProps) {
  return (
    /* A column, so a card stretched to its grid row grows its body and keeps
       the footer on the row's bottom edge. */
    <section className={cn("flex min-w-0 flex-col overflow-hidden rounded-lg border bg-card", className)}>
      {title || description ? (
        <header className="flex items-center justify-between gap-4 border-b px-4 py-3">
          <div>
            {title ? <h2 className="text-sm font-semibold">{title}</h2> : null}
            {description ? <p className="mt-1 text-xs text-muted-foreground">{description}</p> : null}
          </div>
          {actions ? <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div> : null}
        </header>
      ) : null}
      <div className="flex-1 divide-y">{children}</div>
      {footer ? <footer className="border-t bg-muted/20 px-4 py-3">{footer}</footer> : null}
    </section>
  );
}
