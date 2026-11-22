/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Breakpoint Navigation With Viewer-Owned Desktop Geometry. */

/* === External Imports === */

import { useCallback } from "react";

/* === Local Imports === */

import { useFocusMode } from "./focus-mode";
import { useShellDensity } from "./use-shell-density";
import { useUiStore } from "../stores/ui-store";

import { type NavigationMode, sidebarFrameWidth, sidebarWidth } from "./navigation-model";
import { useNavigationMode } from "./use-navigation-mode";

/* === Public Functions === */

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
    width: sidebarFrameWidth(mode, collapsed, savedWidth),
    resize: (width) => setUiState({ sidebarWidth: sidebarWidth(width) }),
  };
}

/* === Types === */
export type SidebarState = {
  collapsed: boolean;
  toggle: () => void;
  resize: (width: number) => void;
  width: string;
  mode: NavigationMode;
};
