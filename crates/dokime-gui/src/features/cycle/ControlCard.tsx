/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The Control card both viewers lead with.
 *
 * One state line saying where the node stands, the step progress beside it
 * while a member runs, and the node's lifecycle actions in a fixed slot at the
 * right: the execution's Start, Cancel, Pause or Resume and Stop, the cycle's
 * Seal Cycle. The slot keeps its column whether it holds one action, several
 * or none, so the state line never moves when the actions change. Anything
 * else the node needs said — the executions waiting on input — follows across
 * the whole card. Prompts are not here: the execution's prompt is its own card
 * directly below.
 */

import type { ReactNode } from "react";

import { Progress } from "@/components/ui/progress";
import { ExecutionSectionCard } from "./ExecutionSectionCard";

export type ControlCardProps = {
  /** What the actions fieldset is called, "Execution #2 Controls". */
  label: string;
  /** The state line, `Finalizing · saving evidence` or the running step's label. */
  state: string;
  /** Secondary text after the state, muted: the running step's id. */
  detail?: string | null;
  /** The live step progress, as a bar with its count to the right. */
  progress?: { completed: number; total: number } | null;
  /** The lifecycle actions, in card order; `null` leaves the slot empty. */
  actions?: ReactNode;
  /** Detail across the whole card, under the state row. */
  children?: ReactNode;
};

/**
 * One row on one vertical centre line: the state, the bar with rounded ends,
 * its count to its right, and the actions. The bar takes the free width; the
 * row keeps its tracks with or without progress, so nothing moves when a run
 * starts or ends.
 */
export function ControlCard({ actions, children, detail, label, progress, state }: ControlCardProps) {
  return (
    <ExecutionSectionCard title="Control">
      <div className="grid min-h-[var(--bar-control-height)] min-w-0 grid-cols-[minmax(0,auto)_minmax(0,1fr)_auto_auto] items-center gap-x-4 gap-y-2 px-1">
        <p aria-live="polite" className="min-w-0 text-sm leading-5 [overflow-wrap:anywhere]" data-control-state>
          {state}
          {detail ? <span className="ml-2 font-mono text-xs text-muted-foreground">{detail}</span> : null}
        </p>
        {progress ? (
          <>
            <Progress
              aria-label="Execution Progress"
              className="h-1.5 min-w-16 rounded-full [&>[data-slot=progress-indicator]]:rounded-full"
              value={progress.total > 0 ? (progress.completed / progress.total) * 100 : 0}
            />
            <span className="text-xs text-muted-foreground tabular-nums">
              {progress.completed}/{progress.total}
            </span>
          </>
        ) : (
          <>
            <span />
            <span />
          </>
        )}
        <fieldset aria-label={label} className="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]">
          {actions}
        </fieldset>
        {children ? <div className="col-span-4 min-w-0">{children}</div> : null}
      </div>
    </ExecutionSectionCard>
  );
}
