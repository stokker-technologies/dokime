// Dokime — SPDX-License-Identifier: Apache-2.0
import { createContext, useContext, useMemo } from "react";
import { resolvePlotThemeColors } from "@dokime/plots/colors";

export const PlotPaletteContext = createContext<string[] | null>(null);

export function usePlotPalette(): string[] {
  const palette = useContext(PlotPaletteContext);
  return useMemo(() => palette ?? resolvePlotThemeColors().palette, [palette]);
}
