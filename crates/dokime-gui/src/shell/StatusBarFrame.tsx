/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Live Footer Geometry.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import type { ReactNode } from "react";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */
export function StatusBarFrame({ children }: { children: ReactNode }) {
  return (
    <footer className="flex min-w-0 shrink-0 border-t bg-card text-xs text-muted-foreground" style={{ height: "var(--status-bar-height)" }}>
      {children}
    </footer>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
