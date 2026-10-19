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

type DocumentProps = {
  children: ReactNode;
  className?: string;
  description?: ReactNode;
  title?: ReactNode;
};

type DocumentSectionProps = {
  children: ReactNode;
  className?: string;
  description?: ReactNode;
  title: ReactNode;
};

type DocumentGridProps = {
  children: ReactNode;
  className?: string;
};

/* === Components === */

export function Document({ children, className, description, title }: DocumentProps) {
  return (
    <article className={cn("document grid min-w-0 gap-5", className)}>
      {title || description ? (
        <header className="grid min-w-0 gap-1">
          {title ? <h2 className="truncate text-xl font-semibold">{title}</h2> : null}
          {description ? <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">{description}</p> : null}
        </header>
      ) : null}
      {children}
    </article>
  );
}

export function DocumentSection({ children, className, description, title }: DocumentSectionProps) {
  return (
    <section className={cn("document-section grid min-w-0 gap-3", className)}>
      <header className="grid min-w-0 gap-0.5">
        <h3 className="truncate text-base font-semibold">{title}</h3>
        {description ? <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">{description}</p> : null}
      </header>
      {children}
    </section>
  );
}

export function DocumentGrid({ children, className }: DocumentGridProps) {
  return <div className={cn("document-grid grid min-w-0 grid-cols-1 gap-3 md:grid-cols-12", className)}>{children}</div>;
}
