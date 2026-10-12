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
  className?: string;
  description?: ReactNode;
  footer?: ReactNode;
  title?: ReactNode;
};

/* === Components === */

/** A grouped settings card: an optional header, a divided body of rows, and an optional footer. */
export function SettingsCard({
  children,
  className,
  description,
  footer,
  title,
}: SettingsCardProps) {
  return (
    <section className={cn("overflow-hidden rounded-lg border bg-card", className)}>
      {title || description ? (
        <header className="border-b px-4 py-3">
          {title ? <h2 className="text-sm font-semibold">{title}</h2> : null}
          {description ? (
            <p className="mt-0.5 text-xs text-muted-foreground">{description}</p>
          ) : null}
        </header>
      ) : null}
      <div className="divide-y">{children}</div>
      {footer ? <footer className="border-t bg-muted/20 px-4 py-3">{footer}</footer> : null}
    </section>
  );
}
