/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The shared tag column of the Library's suite and template rows.
 *
 * Tags are a shared column, so the column has a fixed width and a single line:
 * a suite with twenty tags must not push the row's other columns around or
 * grow the row's height. What does not fit collapses into one "+N" chip whose
 * tooltip carries the rest. */

/* === Local Imports === */

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { formatTagLabel } from "./model";

/* === Constants === */

/** Character budget of the column, measured off the 11px chip face. */
const TAG_COLUMN_BUDGET = 44;

/** What a "+N" chip costs of that budget. */
const OVERFLOW_BUDGET = 5;

/** Fixed column width so suite and template rows share one tag column. */
export const TAG_COLUMN_WIDTH = "w-[240px] xl:w-[320px]";

/* === Components === */

export function SuiteTags({ className, tags }: { className?: string; tags: string[] }) {
  const { hidden, visible } = splitTags(tags);

  return (
    <div className={cn("flex min-w-0 flex-nowrap items-center gap-1 overflow-hidden", TAG_COLUMN_WIDTH, className)}>
      {visible.map((tag) => (
        <TagChip key={tag} label={formatTagLabel(tag)} />
      ))}
      {hidden.length ? (
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              aria-label={`${hidden.length} more tags: ${hidden.map(formatTagLabel).join(", ")}`}
              className="shrink-0 rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
              type="button"
            >
              <TagChip label={`+${hidden.length}`} />
            </button>
          </TooltipTrigger>
          <TooltipContent>{hidden.map(formatTagLabel).join(", ")}</TooltipContent>
        </Tooltip>
      ) : null}
    </div>
  );
}

/* === Subcomponents === */

/** One chip face, shared by the tag column and the Library's provenance pills. */
export function TagChip({ className, label, title }: { className?: string; label: string; title?: string }) {
  return (
    <span
      className={cn(
        "inline-flex h-5 shrink-0 items-center rounded-sm border px-1.5 text-[11px] leading-none whitespace-nowrap text-muted-foreground [[data-density=touch]_&]:h-6",
        className,
      )}
      title={title}
    >
      {label}
    </span>
  );
}

/* === Private Functions === */

/** Greedy fit against the character budget, reserving room for the "+N" chip. */
export function splitTags(tags: string[]): { hidden: string[]; visible: string[] } {
  const visible: string[] = [];
  let used = 0;
  for (const [index, tag] of tags.entries()) {
    const remaining = tags.length - index;
    const budget = remaining > 1 ? TAG_COLUMN_BUDGET - OVERFLOW_BUDGET : TAG_COLUMN_BUDGET;
    const cost = formatTagLabel(tag).length + 3;
    if (used + cost > budget) {
      return { hidden: tags.slice(index), visible };
    }
    used += cost;
    visible.push(tag);
  }
  return { hidden: [], visible };
}
