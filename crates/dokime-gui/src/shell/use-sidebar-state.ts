/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Breakpoint Navigation With Viewer-Owned Desktop Geometry. */

import { useCallback, useSyncExternalStore } from "react";

import { useFocusMode } from "@/shell/focus-mode";
import { useShellDensity } from "@/shell/use-shell-density";
import { useUiStore } from "@/stores/ui-store";

import { type NavigationMode, navigationMode, sidebarWidth } from "./navigation-model";

export function useNavigationMode() {
  return useSyncExternalStore(observeViewport, readViewport, () => "desktop" as const);
}
export function useSidebarState(): SidebarState {
  const mode = useNavigationMode();
  const density = useShellDensity();
  const choices = useUiStore((state) => state.sidebarCollapsed);
  const savedWidth = useUiStore((state) => state.sidebarWidth);
  const setUiState = useUiStore((state) => state.setUiState);
  const { focused } = useFocusMode();
  const collapsed = mode !== "desktop" || (choices[density] ?? false) || focused;
  const toggle = useCallback(() => {
    setUiState({ sidebarCollapsed: { ...choices, [density]: !collapsed } });
  }, [choices, collapsed, density, setUiState]);
  return {
    collapsed,
    toggle,
    mode,
    width: mode === "phone" ? "0px" : mode === "tablet" ? "4.5rem" : collapsed ? "3.5rem" : `${sidebarWidth(savedWidth)}px`,
  };
}

export type SidebarState = {
  collapsed: boolean;
  toggle: () => void;
  width: string;
  mode: NavigationMode;
};

function readViewport(): NavigationMode {
  return navigationMode(typeof window === "undefined" ? 1024 : window.innerWidth);
}
function observeViewport(onChange: () => void): () => void {
  const queries = [window.matchMedia("(min-width: 600px)"), window.matchMedia("(min-width: 1024px)")];
  for (const query of queries) query.addEventListener("change", onChange);
  return () => {
    for (const query of queries) query.removeEventListener("change", onChange);
  };
}
