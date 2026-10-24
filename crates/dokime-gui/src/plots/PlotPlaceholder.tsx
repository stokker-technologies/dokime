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
import type { PlotPlaceholderText } from "@/plots/plot-placeholder";

/* === Types === */

type PlotPlaceholderProps = PlotPlaceholderText & {
  title?: string;
  description?: string | null;
  actions?: ReactNode;
};

/* === Components === */

/**
 * The centred box every plot shows in place of a chart it cannot draw yet. A
 * failed plot keeps the chart's full height under a red tint, with its
 * diagnostic centred both ways and wrapped to the plot's width.
 */
export function PlotPlaceholder({ actions, description, detail, label, title, tone }: PlotPlaceholderProps) {
  const failed = tone === "error";
  return (
    <div
      className={cn(
        "plot-empty relative grid place-content-center justify-items-center gap-2 rounded-md p-4 text-center",
        failed
          ? "min-h-[230px] border border-destructive/30 bg-destructive/10 text-destructive"
          : "min-h-44 bg-muted/20 text-muted-foreground",
      )}
      data-testid="plot-placeholder"
      data-tone={tone}
      role={failed ? "alert" : undefined}
    >
      {actions ? <div className="absolute top-2 right-2">{actions}</div> : null}
      {title ? <strong className={failed ? "text-foreground" : undefined}>{title}</strong> : null}
      {description ? <span className="text-xs">{description}</span> : null}
      <span className={failed ? "font-semibold" : undefined}>{label}</span>
      <span className="max-w-prose text-xs [overflow-wrap:anywhere]">{detail}</span>
    </div>
  );
}
