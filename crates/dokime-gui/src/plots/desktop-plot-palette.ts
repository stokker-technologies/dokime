/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { useAppearance } from "@dokime/document/lib/appearance";
import { useApplicationTheme } from "@dokime/document/lib/application-theme";
import type { ResolvedTheme } from "@dokime/document/lib/theme";
import { materializeThemeFamilyVariables, resolveThemeFamily } from "@dokime/document/lib/theme-family";
import { plotPaletteFromVariables, resolvePlotThemeColors } from "@dokime/plots/colors";
import { useUiStore } from "@dokime/document/stores/ui-store";

/* === Public Functions === */

export function usePlotPalette(): string[] {
  const appearance = useAppearance();
  const { resolvedTheme } = useApplicationTheme();
  const themeFamily = useUiStore((state) => state.themeFamily);
  const resolvedMode: ResolvedTheme = resolvedTheme === "dark" ? "dark" : "light";
  const selectedTheme = resolveThemeFamily(appearance.themes, {
    activeThemeId: appearance.active_theme,
    storedThemeId: themeFamily,
  });

  return useMemo(() => {
    return plotPaletteFromVariables(materializeThemeFamilyVariables(selectedTheme, resolvedMode)) ?? resolvePlotThemeColors().palette;
  }, [resolvedMode, selectedTheme]);
}
