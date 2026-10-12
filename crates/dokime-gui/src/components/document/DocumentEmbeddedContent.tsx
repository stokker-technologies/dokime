/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ComponentPropsWithoutRef, ElementType, ReactNode } from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Types === */

type DocumentEmbeddedTone = "default" | "warning";

type DocumentEmbeddedBlockOwnProps<T extends ElementType> = {
  as?: T;
  children: ReactNode;
  className?: string;
  tone?: DocumentEmbeddedTone;
};

type DocumentEmbeddedBlockProps<T extends ElementType> = DocumentEmbeddedBlockOwnProps<T> &
  Omit<ComponentPropsWithoutRef<T>, keyof DocumentEmbeddedBlockOwnProps<T>>;

type DocumentEmbeddedEmptyStateProps = {
  children: ReactNode;
  className?: string;
  dashed?: boolean;
};

/* === Components === */

export function DocumentEmbeddedBlock<T extends ElementType = "section">({
  as,
  children,
  className,
  tone = "default",
  ...props
}: DocumentEmbeddedBlockProps<T>) {
  const Component = as ?? "section";

  return (
    <Component
      className={cn(
        "grid min-w-0 gap-2 border-l-2",
        tone === "default" && "border-border/70 pl-3",
        tone === "warning" && "border-warning/40 bg-warning/10 px-3 py-2",
        className,
      )}
      {...props}
    >
      {children}
    </Component>
  );
}

export function DocumentEmbeddedEmptyState({
  children,
  className,
  dashed = false,
}: DocumentEmbeddedEmptyStateProps) {
  return (
    <p
      className={cn(
        "text-sm text-muted-foreground",
        dashed && "border-l-2 border-dashed border-border/70 pl-3",
        className,
      )}
    >
      {children}
    </p>
  );
}
