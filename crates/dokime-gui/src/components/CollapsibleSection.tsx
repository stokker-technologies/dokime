/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ChevronDown } from "lucide-react";
import { type ReactNode, useState } from "react";

/* === Local Imports === */

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { documentCardLayoutStyle } from "@/lib/document-layout";
import type { DocumentCardLayout } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Types === */

type CollapsibleSectionProps = {
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
  defaultOpen?: boolean;
  eyebrow?: string;
  icon?: ReactNode;
  layout?: DocumentCardLayout;
  summary?: ReactNode;
  title: string;
};

/* === Components === */

export function CollapsibleSection({
  actions,
  children,
  className,
  contentClassName,
  defaultOpen = false,
  eyebrow,
  icon,
  layout,
  summary,
  title,
}: CollapsibleSectionProps) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <Collapsible
      className={cn("document-card overflow-hidden rounded-lg border bg-card", className)}
      data-density={layout?.density}
      open={open}
      style={layout ? documentCardLayoutStyle(layout) : undefined}
      onOpenChange={setOpen}
    >
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 px-4 py-3">
        <CollapsibleTrigger
          aria-label={title}
          className="grid min-w-0 gap-0.5 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          type="button"
        >
          <span className="flex min-w-0 items-center gap-2">
            {icon ? <span className="shrink-0 text-muted-foreground">{icon}</span> : null}
            <span className="min-w-0">
              {eyebrow ? (
                <span className="block truncate text-xs font-bold uppercase text-muted-foreground">
                  {eyebrow}
                </span>
              ) : null}
              <span className="block truncate text-lg font-semibold">{title}</span>
            </span>
          </span>
          {summary ? (
            <span className="block truncate text-xs text-muted-foreground">{summary}</span>
          ) : null}
        </CollapsibleTrigger>
        <div className="flex items-center gap-1">
          {actions}
          <CollapsibleTrigger
            aria-label={open ? `Collapse ${title}` : `Expand ${title}`}
            className="grid size-8 place-items-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
            type="button"
          >
            <ChevronDown className={cn("transition-transform", open ? "rotate-180" : null)} />
          </CollapsibleTrigger>
        </div>
      </div>
      <CollapsibleContent>
        <div className={cn("border-t p-3", contentClassName)}>{children}</div>
      </CollapsibleContent>
    </Collapsible>
  );
}
