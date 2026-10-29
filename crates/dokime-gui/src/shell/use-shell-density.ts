/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The density the shell is actually rendering at.
 *
 * `data-density` on the document root is what every density rule in
 * `styles.css` reads, so it — not a store field — is the rendered truth.
 * `ThemeController` writes the store's preference there, but the attribute is
 * also the surface the desktop appearance pass and the screenshot harness
 * drive directly. Layout that must agree with the CSS reads it here.
 */

import { useSyncExternalStore } from "react";

import type { DensityPreference } from "@/lib/theme";

const DENSITIES: ReadonlySet<string> = new Set(["comfortable", "compact", "touch"]);

function readDensity(): DensityPreference {
  if (typeof document === "undefined") return "comfortable";
  const value = document.documentElement.dataset.density ?? "";
  return DENSITIES.has(value) ? (value as DensityPreference) : "comfortable";
}

function observeDensity(onChange: () => void): () => void {
  if (typeof MutationObserver === "undefined") return () => undefined;
  const observer = new MutationObserver(onChange);
  observer.observe(document.documentElement, {
    attributeFilter: ["data-density"],
    attributes: true,
  });
  return () => observer.disconnect();
}

/** The density `styles.css` is applying right now. */
export function useShellDensity(): DensityPreference {
  return useSyncExternalStore(observeDensity, readDensity, () => "comfortable" as const);
}
