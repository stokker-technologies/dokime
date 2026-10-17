/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one grid every settings page lays its cards in.
 *
 * One column up to the `wide` breakpoint, two from there. Cards must be direct
 * children: the grid stretches each row, so two cards side by side start on
 * one top edge and end on one bottom edge, and `SettingsCard` keeps its footer
 * on that bottom edge.
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Types === */

type SettingsGridProps = {
  children: ReactNode;
};

/* === Components === */

export function SettingsGrid({ children }: SettingsGridProps) {
  return <div className="grid min-w-0 content-start gap-[var(--section-gap)] wide:grid-cols-2">{children}</div>;
}
