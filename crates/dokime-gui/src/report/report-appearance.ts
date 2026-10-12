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
import { type Brand, defaultBrand } from "@/lib/brand";
import { applyThemeStylesheet, applyThemeVariables } from "@/lib/theme";
import { materializeThemeFamilyVariables, resolveThemeFamily } from "@/lib/theme-family";

/* === Report appearance ===
   The generated report carries the captured runtime appearance frame. Applying it here
   keeps the frozen report under the theme contract instead of reverting
   to default branding. */

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

  const active = resolveThemeFamily(frame.themes ?? [], { activeThemeId: frame.active_theme });
  if (active) {
    applyThemeStylesheet(active.stylesheet);
    const mode =
      typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    if (mode === "dark") {
      document.documentElement.classList.add("dark");
    }
    applyThemeVariables(materializeThemeFamilyVariables(active, mode));
  }

  if (frame.app_name?.trim()) {
    document.title = `${frame.app_name} Report`;
  }

  return brandFromFrame(frame);
}

/* === Private Functions === */

function appearanceFrame(value: unknown): RuntimeAppearanceFrame | null {
  if (
    typeof value !== "object" ||
    value === null ||
    !("themes" in value) ||
    !Array.isArray((value as { themes: unknown }).themes)
  ) {
    return null;
  }
  return value as RuntimeAppearanceFrame;
}

function brandFromFrame(frame: RuntimeAppearanceFrame): Brand {
  const label = frame.app_name?.trim() || defaultBrand.label;
  // Reports embed their checkpoint's image bytes, including when opened from disk.
  const embeddedLogo =
    frame.logo &&
    /^data:image\/(?:png|jpeg|gif|webp|svg\+xml|x-icon|vnd\.microsoft\.icon);base64,[A-Za-z0-9+/]+=*$/.test(
      frame.logo.url,
    );
  if (frame.logo && embeddedLogo) {
    return {
      label,
      logo: {
        kind: "image",
        src: frame.logo.url,
        alt: frame.logo.alt ?? label,
      },
    };
  }
  return { ...defaultBrand, label };
}
