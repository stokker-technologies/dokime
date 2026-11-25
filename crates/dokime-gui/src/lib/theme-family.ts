/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { RuntimeThemeFamily } from "./types/appearance";
import { type ResolvedTheme, type ThemeVariableMap, themeCornerRadiusVariables } from "./theme";

/* === Types === */

type ThemeFamilyPreference = {
  activeThemeId?: string | null;
  storedThemeId?: string | null;
};

/* === Public Functions === */

export function resolveThemeFamily(
  themes: readonly RuntimeThemeFamily[],
  { activeThemeId, storedThemeId }: ThemeFamilyPreference,
): RuntimeThemeFamily | null {
  return (
    themes.find((theme) => theme.theme_id === storedThemeId) ??
    themes.find((theme) => theme.theme_id === activeThemeId) ??
    themes[0] ??
    null
  );
}

export function materializeThemeFamilyVariables(
  theme: RuntimeThemeFamily | null | undefined,
  mode: ResolvedTheme,
): ThemeVariableMap | null {
  if (!theme) {
    return null;
  }

  const variant = theme[mode];
  const palette: ThemeVariableMap = { ...(theme.variables ?? {}), ...variant.variables };
  const variables: ThemeVariableMap = { ...palette, ...themeCornerRadiusVariables(theme.corner_radii) };
  // Keyboard Focus Is Independent Of A Variable Palette's Brand And Destructive Colors; A Stylesheet Owns Its Own.
  if (Object.keys(palette).length) {
    variables["focus-ring"] ||= mode === "dark" ? "#93c5fd" : "#001b4a";
  }
  variant.plot_palette?.forEach((color, index) => {
    if (color.trim()) {
      variables[`chart-${index + 1}`] = color;
    }
  });

  return variables;
}
