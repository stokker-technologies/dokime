/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The header both viewers open with: where the node sits, what it is called
 * and where it stands.
 *
 * Three rows on one grid. The breadcrumb names the node's parents and leads
 * back to them. The title carries its rename pencil inline, then the status
 * pill and the progress text on the title's baseline row; the title gives way
 * first, so the pill and the progress always read whole. An optional
 * description spans the header under them. The cycle and the execution viewer
 * compose this same header; with `live={false}` the pencil is gone and the
 * breadcrumb still selects, as the frozen report's sidebar does.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { ChevronRight } from "lucide-react";
import type { ComponentProps, ReactNode } from "react";

// Local Imports
import { Badge } from "@/components/ui/badge";
import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { EditableLabel } from "./EditableLabel";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** One parent in the breadcrumb; without `onSelect` it is the current page. */
export type NodeCrumb = { label: string; onSelect?: () => void };

export type NodeHeaderProps = {
  breadcrumb: NodeCrumb[];
  title: string;
  /** Text before the title that names the node, "Execution 1 · ". */
  prefix?: string;
  subject: "Cycle" | "Execution";
  /** Renames the node; `null` keeps the title plain. */
  onRename: ((label: string) => Promise<unknown>) | null;
  status: { label: string; variant: ComponentProps<typeof Badge>["variant"] };
  /** The progress text beside the pill. */
  progress: string | null;
  description?: ReactNode;
  /** False in the frozen report: no pencil. */
  live: boolean;
};

/* === Components === */

export function NodeHeader({ breadcrumb, description, live, onRename, prefix, progress, status, subject, title }: NodeHeaderProps) {
  return (
    <header className="grid min-w-0 gap-y-1" data-node-header={subject}>
      <nav aria-label="Breadcrumb" className="min-w-0">
        <ol className="flex min-w-0 flex-wrap items-center gap-x-1 text-xs leading-5 text-muted-foreground">
          {breadcrumb.map((crumb, index) => (
            <li className="flex min-w-0 items-center gap-x-1 [overflow-wrap:anywhere]" key={crumb.label}>
              {index > 0 ? <ChevronRight aria-hidden className="size-3 shrink-0" /> : null}
              {crumb.onSelect ? (
                <button className="hover:text-foreground hover:underline" onClick={crumb.onSelect} type="button">
                  {crumb.label}
                </button>
              ) : (
                <span aria-current="page">{crumb.label}</span>
              )}
            </li>
          ))}
        </ol>
      </nav>
      <div className="grid min-h-[var(--bar-control-height)] min-w-0 grid-cols-[minmax(0,auto)_auto_minmax(0,1fr)] items-center gap-x-3">
        <EditableLabel label={title} onRename={live ? onRename : null} prefix={prefix} subject={subject} />
        <Badge className={STATUS_PILL_CLASS} variant={status.variant}>
          {status.label}
        </Badge>
        <p className="min-w-0 text-xs leading-6 text-muted-foreground tabular-nums [overflow-wrap:anywhere]">{progress}</p>
      </div>
      {description ? <div className="text-sm text-muted-foreground">{description}</div> : null}
    </header>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
