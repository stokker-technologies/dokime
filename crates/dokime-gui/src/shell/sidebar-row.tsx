/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One row shape for the whole sidebar tree.
 *
 * A destination, a settings section, a cycle overview and a case are the same
 * three columns — lead, label, trailing — at the same height, and the only
 * thing that changes with depth is where the label starts: 44px for a
 * destination and for the cycle overview, 60px for a case. The inset is a lead
 * column width on a full-width row, never a margin, so no level gets a second
 * background panel around it and every trailing pill lands on one column.
 *
 * A long name is given two lines before anything is cut; the row grows.
 */

import type { LucideIcon } from "lucide-react";
import type { ComponentProps, ReactNode, Ref } from "react";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

/** Where the row's label starts. The tree has exactly these three lines. */
export type SidebarRowLevel = "destination" | "overview" | "case";

export type SidebarRowProps = Omit<ComponentProps<"li">, "onSelect"> & {
  /** Marks the row as the operator's current place. */
  active?: boolean;
  /** Rendered after the label, on the row's shared trailing columns. */
  children?: ReactNode;
  /** Icon only, centred: the collapsed sidebar. */
  collapsed?: boolean;
  railLabel?: boolean;
  icon?: LucideIcon;
  label: ReactNode;
  level?: SidebarRowLevel;
  /** Omitted by a row that states rather than navigates. */
  onSelect?: () => void;
  ref?: Ref<HTMLLIElement>;
  title?: string;
};

/**
 * The lead column of each level, measured from the row's own 20px inset.
 *
 * 20px + 24px puts a destination's and the cycle overview's label on 44px;
 * 20px + 40px puts a case's label on 60px. A destination's 20px icon fills the
 * first 20px of its own lead column, so icon and label never trade places.
 */
const LEVEL_LEAD: Record<SidebarRowLevel, string> = {
  case: "grid-cols-(--layout-sidebar-case)",
  destination: "grid-cols-(--layout-sidebar-destination)",
  overview: "grid-cols-(--layout-sidebar-destination)",
};

const LEVEL_TEXT: Record<SidebarRowLevel, string> = {
  case: "text-sm font-normal",
  destination: "text-sm font-medium",
  overview: "text-sm font-medium",
};

export function SidebarRow({
  active,
  children,
  collapsed,
  railLabel,
  icon: Icon,
  label,
  level = "destination",
  onSelect,
  ref,
  title,
  ...rest
}: SidebarRowProps) {
  const tone = active ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-muted hover:text-foreground";

  if (collapsed) {
    return (
      <li
        className={cn("grid min-h-[var(--sidebar-row-height)] place-items-center rounded-lg", tone)}
        data-active={active || undefined}
        ref={ref}
        {...rest}
      >
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              aria-current={active ? "page" : undefined}
              aria-label={title}
              className="grid min-h-[var(--control-height)] w-full place-items-center content-center gap-1 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-ring"
              onClick={onSelect}
              type="button"
            >
              {Icon ? <Icon aria-hidden className="size-5" /> : null}
              {railLabel ? <span className="text-xs">{label}</span> : null}
            </button>
          </TooltipTrigger>
          {!railLabel ? <TooltipContent side="right">{title}</TooltipContent> : null}
        </Tooltip>
      </li>
    );
  }

  const labelNode = <span className={cn("line-clamp-2 min-w-0 text-left leading-tight", LEVEL_TEXT[level])}>{label}</span>;

  return (
    <li
      className={cn("grid min-h-[var(--sidebar-row-height)] items-center gap-1 rounded-lg py-1 pr-2 pl-4", LEVEL_LEAD[level], tone)}
      data-active={active || undefined}
      data-level={level}
      ref={ref}
      {...rest}
    >
      <span aria-hidden className="flex shrink-0 items-center justify-start">
        {Icon ? <Icon className="size-5" /> : null}
      </span>
      {onSelect ? (
        <button
          aria-current={active ? "page" : undefined}
          className="grid min-h-[var(--control-height)] min-w-0 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
          onClick={onSelect}
          type="button"
        >
          {labelNode}
        </button>
      ) : (
        <span className="grid min-w-0" title={title}>
          {labelNode}
        </span>
      )}
      <span className="flex shrink-0 items-center gap-1">{children}</span>
    </li>
  );
}
