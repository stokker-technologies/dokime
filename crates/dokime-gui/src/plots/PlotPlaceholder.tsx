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

import type { PlotPlaceholderText } from "@/plots/plot-placeholder";

/* === Types === */

type PlotPlaceholderProps = PlotPlaceholderText & {
  title?: string;
  description?: string | null;
  actions?: ReactNode;
};

/* === Components === */

/** The centred box every plot shows in place of a chart it cannot draw yet. */
export function PlotPlaceholder({ actions, description, detail, label, title }: PlotPlaceholderProps) {
  return (
    <div
      className="plot-empty relative grid min-h-44 place-content-center gap-2 rounded-md bg-muted/20 p-4 text-center text-muted-foreground"
      data-testid="plot-placeholder"
    >
      {actions ? <div className="absolute top-2 right-2">{actions}</div> : null}
      {title ? <strong>{title}</strong> : null}
      {description ? <span className="text-xs">{description}</span> : null}
      <span>{label}</span>
      <span className="text-xs">{detail}</span>
    </div>
  );
}
