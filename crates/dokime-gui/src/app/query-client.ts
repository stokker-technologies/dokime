/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { QueryClient } from "@tanstack/react-query";

const GUI_QUERY_FRESH_MS = 1_000;
const GUI_QUERY_RETRIES = 1;

/* === Public Functions === */

export function createDokimeQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        retry: GUI_QUERY_RETRIES,
        staleTime: GUI_QUERY_FRESH_MS,
      },
    },
  });
}
