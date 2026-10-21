/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { RuntimeAppearanceFrame } from "@/lib/appearance";
import { type Brand, brandFromLogos, defaultBrand } from "@/lib/brand";
import { applyThemeStylesheet, applyThemeVariables, type ResolvedTheme } from "@/lib/theme";
import { materializeThemeFamilyVariables, resolveThemeFamily } from "@/lib/theme-family";

/* === Report appearance ===
   The generated report carries the captured runtime appearance frame. Applying it here
   keeps the frozen report under the theme contract instead of reverting
   to default branding. */

/* === Constants === */

/* The family the payload named, kept so a mode change can re-materialize its
   variables for the other scheme. */
let reportThemeFamily: ReturnType<typeof resolveThemeFamily> = null;

/* === Public Functions === */

/**
 * Apply the report payload's appearance frame: theme variables for the
 * viewer's color scheme, the document title, and the brand for the shell.
 * Falls back to the default brand when no frame was embedded.
 */
export function applyReportAppearance(appearance: unknown): Brand {
  const frame = appearanceFrame(appearance);
  if (!frame) {
    return defaultBrand;
  }

  reportThemeFamily = resolveThemeFamily(frame.themes ?? [], { activeThemeId: frame.active_theme });
  if (reportThemeFamily) {
    applyThemeStylesheet(reportThemeFamily.stylesheet);
    applySystemColorMode();
  }

  if (frame.app_name?.trim()) {
    document.title = `${frame.app_name} Report`;
  }

  return brandFromFrame(frame);
}

/* === Private Functions === */

/**
 * Render the report in the viewer's color scheme at the moment of the call;
 * the report holds no preference store and no control to choose one.
 */
function applySystemColorMode(): void {
  const mode = systemColorMode();
  const root = document.documentElement;
  root.classList.toggle("dark", mode === "dark");
  root.classList.toggle("light", mode === "light");
  root.style.colorScheme = mode;
  if (reportThemeFamily) {
    applyThemeVariables(materializeThemeFamilyVariables(reportThemeFamily, mode));
  }
}

function systemColorMode(): ResolvedTheme {
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function appearanceFrame(value: unknown): RuntimeAppearanceFrame | null {
  if (typeof value !== "object" || value === null || !("themes" in value) || !Array.isArray((value as { themes: unknown }).themes)) {
    return null;
  }
  return value as RuntimeAppearanceFrame;
}

function brandFromFrame(frame: RuntimeAppearanceFrame): Brand {
  const label = frame.app_name?.trim() || defaultBrand.label;
  // Reports embed their checkpoint's image bytes, including when opened from disk.
  const logos = (frame.logos ?? []).filter((logo) =>
    /^data:image\/(?:png|jpeg|gif|webp|svg\+xml|x-icon|vnd\.microsoft\.icon);base64,[A-Za-z0-9+/]+=*$/.test(logo.url),
  );
  return brandFromLogos(label, logos);
}
