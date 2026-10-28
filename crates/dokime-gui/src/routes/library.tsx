/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { LibrarySurface } from "@/features/library/LibrarySurface";

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

export const Route = createFileRoute("/library")({
  component: LibraryRoute,
  validateSearch: (search) => librarySearchSchema.parse(search),
});

function LibraryRoute() {
  const { appendTo } = Route.useSearch();
  return <LibrarySurface appendTo={appendTo} />;
}
