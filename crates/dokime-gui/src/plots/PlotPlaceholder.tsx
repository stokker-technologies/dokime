/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import type { ReactNode } from "react";

/* === Local Imports === */

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
 * failed plot delegates its normalized record to ErrorCard.
 */
export function PlotPlaceholder({ actions, description, detail, label, title, tone }: PlotPlaceholderProps) {
  if (tone === "error")
    return (
      <div className="grid gap-2">
        {actions}
        <ErrorCard failure={failureOf(detail, title ?? label, [title, label, detail].join(":"))} />
      </div>
    );
  return (
    <div
      className="plot-empty relative grid min-h-44 place-content-center justify-items-center gap-2 rounded-lg bg-muted/20 p-4 text-center text-muted-foreground"
      data-testid="plot-placeholder"
      data-tone={tone}
    >
      {actions ? <div className="absolute top-2 right-2">{actions}</div> : null}
      {title ? <strong>{title}</strong> : null}
      {description ? <span className="text-xs">{description}</span> : null}
      <span>{label}</span>
      <span className="max-w-prose text-xs [overflow-wrap:anywhere]">{detail}</span>
    </div>
  );
}
