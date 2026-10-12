/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute, redirect } from "@tanstack/react-router";

/* === Route === */

export const Route = createFileRoute("/$")({
  beforeLoad: () => {
    throw redirect({ replace: true, to: "/cycles" });
  },
});
