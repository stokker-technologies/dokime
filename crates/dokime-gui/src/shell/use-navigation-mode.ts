/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { useSyncExternalStore } from "react";
import { type NavigationMode, navigationMode } from "./navigation-model";

/* === Public Functions === */

export function useNavigationMode() {
  return useSyncExternalStore(observeViewport, readViewport, () => "desktop" as const);
}

/* === Private Functions === */
function readViewport(): NavigationMode {
  return navigationMode(typeof window === "undefined" ? 1024 : window.innerWidth);
}
function observeViewport(onChange: () => void): () => void {
  const queries = [window.matchMedia("(min-width: 600px)"), window.matchMedia("(min-width: 1024px)")];
  for (const query of queries) query.addEventListener("change", onChange);
  return () => {
    for (const query of queries) query.removeEventListener("change", onChange);
  };
}
