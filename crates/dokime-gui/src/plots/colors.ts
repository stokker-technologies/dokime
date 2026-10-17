/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type ResolvedPlotThemeColors = {
  background: string;
  border: string;
  foreground: string;
  mutedForeground: string;
  palette: string[];
};

/* === Constants === */

const PLOT_PALETTE_SIZE = 8;
const CSS_COLOR_FALLBACK = "currentColor";
const CSS_VARIABLE_PATTERN = /^var\(\s*(--[a-zA-Z0-9_-]+)\s*\)$/;
const PLOT_PALETTE_VARIABLES = Array.from({ length: PLOT_PALETTE_SIZE }, (_, index) => `--chart-${index + 1}`);
const PLOT_PALETTE_REFERENCES = PLOT_PALETTE_VARIABLES.map((name) => `var(${name})`);

/* === Public Functions === */

export function plotColor(topic: string, index: number, palette?: readonly string[] | null): string {
  const colors = normalizePlotPalette(palette) ?? PLOT_PALETTE_REFERENCES;
  const offset = hashString(topic) % colors.length;
  return colors[(offset + index) % colors.length] ?? colors[0] ?? CSS_COLOR_FALLBACK;
}

export function normalizePlotPalette(palette: readonly string[] | null | undefined): string[] | null {
  const colors = (palette ?? []).map((color) => color.trim()).filter((color) => color.length > 0);

  return colors.length ? colors : null;
}

export function plotPaletteFromVariables(variables: Record<string, string> | null | undefined): string[] | null {
  return normalizePlotPalette(PLOT_PALETTE_VARIABLES.map((name) => variables?.[name.slice(2)] ?? variables?.[name] ?? ""));
}

/** Resolves the CSS custom properties needed by canvas-backed plots. */
export function resolvePlotThemeColors(element?: Element | null): ResolvedPlotThemeColors {
  const styles = computedStyles(element);
  const foreground = resolvedVariable(styles, "--foreground") ?? CSS_COLOR_FALLBACK;

  return {
    background: resolvedVariable(styles, "--background") ?? foreground,
    border: resolvedVariable(styles, "--border") ?? foreground,
    foreground,
    mutedForeground: resolvedVariable(styles, "--muted-foreground") ?? foreground,
    palette: normalizePlotPalette(
      PLOT_PALETTE_VARIABLES.map((name) => resolvedVariable(styles, name)).filter((color): color is string => color !== null),
    ) ?? [foreground],
  };
}

/** Resolves a semantic `var(--token)` before passing a color to canvas. */
export function resolvePlotCssColor(color: string | null | undefined, element?: Element | null, fallback = CSS_COLOR_FALLBACK): string {
  const value = color?.trim();
  if (!value) {
    return fallback;
  }
  const variable = value.match(CSS_VARIABLE_PATTERN)?.[1];
  if (!variable) {
    return value;
  }
  return resolvedVariable(computedStyles(element), variable) ?? fallback;
}

/* === Private Functions === */

function computedStyles(element?: Element | null): CSSStyleDeclaration | null {
  if (typeof window === "undefined" || typeof window.getComputedStyle !== "function") {
    return null;
  }
  const target = element ?? (typeof document === "undefined" ? null : document.documentElement);
  return target ? window.getComputedStyle(target) : null;
}

function resolvedVariable(styles: CSSStyleDeclaration | null, name: string): string | null {
  const value = styles?.getPropertyValue(name).trim() ?? "";
  return value || null;
}

function hashString(value: string): number {
  return [...value].reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) >>> 0, 0);
}
