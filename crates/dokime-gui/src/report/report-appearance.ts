/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { type AppearanceSnapshot, appearanceStylesheet, type Brand, brandFromAppearance, defaultBrand } from "@/lib/brand";
import { applyBrandFavicon } from "@/lib/favicon";
import { applyThemeStylesheet, type ResolvedTheme } from "@/lib/theme";

/* === Report appearance ===
   The generated report carries the captured host appearance. Applying it here
   keeps the frozen report under the theme contract instead of reverting
   to default organization. */

/* === Constants === */

// Reports embed their checkpoint's image bytes, including when opened from disk.
const EMBEDDED_IMAGE = /^data:image\/(?:png|jpeg|gif|webp|svg\+xml|x-icon|vnd\.microsoft\.icon);base64,[A-Za-z0-9+/]+=*$/;

/* === Public Functions === */

/**
 * Apply the report payload's appearance: the theme stylesheet in the viewer's
 * color scheme, the document title and favicon, and the brand for the shell.
 * Falls back to the default brand when no appearance was embedded.
 */
export function applyReportAppearance(appearance: AppearanceSnapshot | null): Brand {
  if (!appearance) {
    return defaultBrand;
  }

  const stylesheet = appearanceStylesheet(appearance);
  if (stylesheet) {
    applyThemeStylesheet(stylesheet);
    applySystemColorMode();
  }

  if (appearance.appName.trim()) {
    document.title = `${appearance.appName} Report`;
  }

  const brand = brandFromAppearance(appearance, (uri) => EMBEDDED_IMAGE.test(uri));
  applyBrandFavicon(brand);
  return brand;
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
}

function systemColorMode(): ResolvedTheme {
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
