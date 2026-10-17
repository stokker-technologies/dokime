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

import { Card, CardAction, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { documentCardLayoutAttributes, documentCardLayoutStyle } from "@/lib/document-layout";
import type { DocumentCardLayout } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Types === */

type DocumentCardProps = {
  children: ReactNode;
  title: string;
  className?: string;
  contentClassName?: string;
  description?: ReactNode;
  eyebrow?: string;
  icon?: ReactNode;
  layout?: DocumentCardLayout;
};

/* === Components === */

export function DocumentCard({ children, title, className, contentClassName, description, eyebrow, icon, layout }: DocumentCardProps) {
  return (
    <Card
      className={cn("document-card overflow-hidden rounded-lg py-0", className)}
      {...(layout ? documentCardLayoutAttributes(layout) : {})}
      data-density={layout?.density}
      style={layout ? documentCardLayoutStyle(layout) : undefined}
    >
      <CardHeader className="min-h-16 border-b px-4 py-3">
        <div className="min-w-0">
          {eyebrow ? <p className="mb-1 text-xs font-bold uppercase text-muted-foreground">{eyebrow}</p> : null}
          <CardTitle className="truncate text-lg">{title}</CardTitle>
          {description ? <p className="mt-1 text-sm text-muted-foreground">{description}</p> : null}
        </div>
        {icon ? <CardAction>{icon}</CardAction> : null}
      </CardHeader>
      <CardContent className={cn(contentClassName ?? "grid gap-4 p-4")}>{children}</CardContent>
    </Card>
  );
}
