/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

/* === Local Imports === */

import { LibrarySurface } from "@/features/library/LibrarySurface";

/* === Constants === */

const librarySearchSchema = z.object({
  appendTo: z
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

/* === Route === */

export const Route = createFileRoute("/library")({
  component: LibraryRoute,
  validateSearch: (search) => librarySearchSchema.parse(search),
});

/* === Components === */

function LibraryRoute() {
  const { appendTo } = Route.useSearch();
  return <LibrarySurface appendTo={appendTo} />;
}
