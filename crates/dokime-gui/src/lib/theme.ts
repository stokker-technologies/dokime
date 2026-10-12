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
export type ThemeVariableMap = Record<string, string>;
export type ThemeCornerRadii = {
  base?: string | null;
  sm?: string | null;
  md?: string | null;
  lg?: string | null;
  xl?: string | null;
  "2xl"?: string | null;
  "3xl"?: string | null;
  "4xl"?: string | null;
};

/* === Constants === */

let managedThemeVariables = new Set<string>();
const runtimeThemeStyleId = "dokime-runtime-theme";

const cornerRadiusVariables = [
  ["base", "brand-radius"],
  ["sm", "brand-radius-sm"],
  ["md", "brand-radius-md"],
  ["lg", "brand-radius-lg"],
  ["xl", "brand-radius-xl"],
  ["2xl", "brand-radius-2xl"],
  ["3xl", "brand-radius-3xl"],
  ["4xl", "brand-radius-4xl"],
] as const satisfies ReadonlyArray<readonly [keyof ThemeCornerRadii, string]>;

/* === Public Functions === */

export function applyDensity(density: DensityPreference): void {
  if (typeof document === "undefined") {
    return;
  }
  document.documentElement.dataset.density = density;
}

export function themeCornerRadiusVariables(
  cornerRadii: ThemeCornerRadii | null | undefined,
): ThemeVariableMap {
  const variables: ThemeVariableMap = {};
  for (const [key, variable] of cornerRadiusVariables) {
    const value = cornerRadii?.[key];
    if (value?.trim()) {
      variables[variable] = value;
    }
  }
  return variables;
}

export function applyThemeVariables(variables: ThemeVariableMap | null | undefined): void {
  if (typeof document === "undefined") {
    return;
  }
  const root = document.documentElement;
  for (const name of managedThemeVariables) {
    root.style.removeProperty(name);
  }
  managedThemeVariables = new Set();

  for (const [rawName, value] of Object.entries(variables ?? {})) {
    const name = rawName.startsWith("--") ? rawName : `--${rawName}`;
    root.style.setProperty(name, value);
    managedThemeVariables.add(name);
  }
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
