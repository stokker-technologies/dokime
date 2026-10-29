/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Breakpoint Navigation And Persisted Sidebar Bounds.
 */

export type NavigationMode = "phone" | "tablet" | "desktop";

export function navigationMode(width: number): NavigationMode {
  if (width < 600) return "phone";
  return width < 1024 ? "tablet" : "desktop";
}
export function sidebarWidth(value: number): number {
  return Number.isFinite(value) ? Math.min(360, Math.max(200, Math.round(value / 4) * 4)) : 256;
}
export function settingsParent(pathname: string): string {
  return /^\/settings\/plugins\/.+/.test(pathname) ? "/settings/plugins" : "/settings";
}
