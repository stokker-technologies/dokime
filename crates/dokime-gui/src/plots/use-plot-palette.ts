/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useTheme } from "next-themes";
import { useMemo } from "react";

import { useAppearance } from "@/lib/appearance";
import type { ResolvedTheme } from "@/lib/theme";
import { materializeThemeFamilyVariables, resolveThemeFamily } from "@/lib/theme-family";
import { plotPaletteFromVariables, resolvePlotThemeColors } from "@/plots/colors";
import { useUiStore } from "@/stores/ui-store";

export function usePlotPalette(): string[] {
  const appearance = useAppearance();
  const { resolvedTheme } = useTheme();
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
