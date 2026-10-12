/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createRouter } from "@tanstack/react-router";

/* === Local Imports === */

import { routeTree } from "@/routeTree.gen";

/* === Constants === */

export const router = createRouter({
  routeTree,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
