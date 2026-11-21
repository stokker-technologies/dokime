/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useEffect } from "react";

/* === Local Imports === */

import { useAppearance } from "@dokime/document/lib/appearance";
import { useApplicationTheme } from "@dokime/document/lib/application-theme";
import { applyDensity, applyThemeStylesheet, applyThemeVariables, type ResolvedTheme } from "@dokime/document/lib/theme";
import { materializeThemeFamilyVariables, resolveThemeFamily } from "@dokime/document/lib/theme-family";
import { useUiStore } from "@dokime/document/stores/ui-store";

/* === Components === */

/** Applies runtime theme variables and density to the document root. */
export function ThemeController() {
  const appearance = useAppearance();
  const { resolvedTheme } = useApplicationTheme();
  const themeFamily = useUiStore((state) => state.themeFamily);
  const density = useUiStore((state) => state.density);
  const setUiState = useUiStore((state) => state.setUiState);
  const selectedTheme = resolveThemeFamily(appearance.themes, {
    activeThemeId: appearance.active_theme,
    storedThemeId: themeFamily,
  });
  const resolvedMode: ResolvedTheme = resolvedTheme === "dark" ? "dark" : "light";

  useEffect(() => {
    if (appearance.theme_family_locked && appearance.active_theme && themeFamily !== appearance.active_theme) {
      setUiState({ themeFamily: appearance.active_theme });
    }
  }, [appearance.active_theme, appearance.theme_family_locked, setUiState, themeFamily]);

  useEffect(() => {
    applyThemeStylesheet(selectedTheme?.stylesheet);
    return () => applyThemeStylesheet(null);
  }, [selectedTheme?.stylesheet]);

  useEffect(() => {
    applyThemeVariables(materializeThemeFamilyVariables(selectedTheme, resolvedMode));
  }, [resolvedMode, selectedTheme]);

  useEffect(() => {
    applyDensity(density);
  }, [density]);

  return null;
}
