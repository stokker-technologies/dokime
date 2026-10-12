/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { z } from "zod";

/* === Explore URL state ===
   Compare selection lives in the URL (`?selected=a,b`) so it survives
   navigation between the library, the compare view, and cycle detail,
   and deep-links stay shareable. */

/* === Schema === */

export const exploreSearchSchema = z.object({
  selected: z
    .preprocess((value) => {
      const candidate = Array.isArray(value) ? value.at(-1) : value;
      if (typeof candidate !== "string") {
        return undefined;
      }
      const trimmed = candidate.trim();
      return trimmed.length > 0 ? trimmed : undefined;
    }, z.string().optional())
    .catch(undefined),
});

/* === Types === */

export type ExploreSearch = z.infer<typeof exploreSearchSchema>;

/* === Public Functions === */

export function selectedIdsFromSearch(search: ExploreSearch): Set<string> {
  if (!search.selected) {
    return new Set();
  }
  return new Set(
    search.selected
      .split(",")
      .map((id) => id.trim())
      .filter(Boolean),
  );
}

export function searchFromSelectedIds(selectedIds: Set<string>): ExploreSearch {
  return selectedIds.size > 0 ? { selected: [...selectedIds].join(",") } : {};
}

export function toggleSelectedId(selectedIds: Set<string>, cycleId: string): Set<string> {
  const next = new Set(selectedIds);
  if (next.has(cycleId)) {
    next.delete(cycleId);
  } else {
    next.add(cycleId);
  }
  return next;
}
