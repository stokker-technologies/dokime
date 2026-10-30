/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Apply Brand Logos To The Document Favicon
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { type Brand, selectLogo } from "@/lib/brand";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/** Replace tab icons with the brand's artwork, preserving the default when no logos apply. */
export function applyBrandFavicon(brand: Brand): void {
  const dark = typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches;
  const surface = dark ? "dark" : "light";
  const alternative = dark ? "light" : "dark";
  const logo = selectLogo(brand, "contained", surface) ?? selectLogo(brand, "contained", alternative);
  if (!logo || typeof document === "undefined") return;

  // Replace The Entire Link So Stale Type, Sizes And Media Hints Cannot Override The Brand.
  const icons = document.head.querySelectorAll('link[rel~="icon"]');
  const icon = document.createElement("link");
  icon.rel = "icon";
  icon.href = logo.src;
  for (const existing of icons) existing.remove();
  document.head.append(icon);
}

/* === Private Functions === */

/* === Entry Point === */
