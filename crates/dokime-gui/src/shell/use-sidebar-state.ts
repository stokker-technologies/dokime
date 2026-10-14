/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Whether the one sidebar is open, and how wide it therefore is.
 *
 * The sidebar and the status bar's first cell are the same column of the shell
 * grid, so they must agree on one width and one toggle. Both read it here.
 */

/* === External Imports === */

import { useCallback, useSyncExternalStore } from "react";

/* === Local Imports === */

import { useFocusMode } from "@/shell/focus-mode";
import { useShellDensity } from "@/shell/use-shell-density";
import { sidebarStartsCollapsed, useUiStore } from "@/stores/ui-store";

/* === Constants === */

/* Below this the open sidebar would take more than a third of the window, so
   a window this narrow starts on the icon rail unless the operator opened it. */
const NARROW_WINDOW = "(max-width: 56rem)";

/* === Types === */

export type SidebarState = {
  collapsed: boolean;
  toggle: () => void;
  /** The CSS width of the sidebar column, in a form both bars can apply. */
  width: string;
};

/* === Private Functions === */

function readNarrow(): boolean {
  return typeof window !== "undefined" && window.matchMedia(NARROW_WINDOW).matches;
}

function observeNarrow(onChange: () => void): () => void {
  if (typeof window === "undefined") return () => undefined;
  const query = window.matchMedia(NARROW_WINDOW);
  query.addEventListener("change", onChange);
  return () => query.removeEventListener("change", onChange);
}

/* === Public Functions === */

export function useSidebarState(): SidebarState {
  const density = useShellDensity();
  const narrow = useSyncExternalStore(observeNarrow, readNarrow, () => false);
  const collapsedByDensity = useUiStore((state) => state.sidebarCollapsed);
  const setUiState = useUiStore((state) => state.setUiState);
  const { focused } = useFocusMode();

  // Focus borrows a collapsed sidebar for as long as it lasts; the operator's
  // own choice is untouched and comes back when they leave.
  const chosen = sidebarStartsCollapsed(collapsedByDensity, density, narrow);
  const collapsed = chosen || focused;

  const toggle = useCallback(() => {
    setUiState({ sidebarCollapsed: { ...collapsedByDensity, [density]: !chosen } });
  }, [chosen, collapsedByDensity, density, setUiState]);

  return {
    collapsed,
    toggle,
    width: collapsed ? "var(--sidebar-width-collapsed)" : "var(--sidebar-width)",
  };
}
