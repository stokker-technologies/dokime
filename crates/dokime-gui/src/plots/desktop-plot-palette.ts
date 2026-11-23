// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ThemeVariableMap } from "../lib/theme";
import { plotPaletteFromVariables, resolvePlotThemeColors } from "./colors";

export function desktopPlotPalette(variables: ThemeVariableMap | null | undefined, fallback = resolvePlotThemeColors().palette): string[] {
  return plotPaletteFromVariables(variables) ?? fallback;
}
