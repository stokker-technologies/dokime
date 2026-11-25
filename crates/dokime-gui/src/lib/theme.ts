/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type ThemeModePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";
export type DensityPreference = "comfortable" | "compact" | "touch";

/* === Constants === */

const runtimeThemeStyleId = "dokime-host-theme";

/* === Public Functions === */

export function applyDensity(density: DensityPreference): void {
  if (typeof document === "undefined") {
    return;
  }
  document.documentElement.dataset.density = density;
}

export function applyThemeStylesheet(stylesheet: string | null | undefined): void {
  if (typeof document === "undefined") {
    return;
  }
  const existing = document.getElementById(runtimeThemeStyleId);
  if (!stylesheet?.trim()) {
    existing?.remove();
    return;
  }
  const style = existing ?? document.createElement("style");
  style.id = runtimeThemeStyleId;
  style.textContent = stylesheet;
  if (!existing) {
    document.head.append(style);
  }
}
