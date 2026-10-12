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

/* === Local Imports === */

import { CyclesHome } from "@/features/cycles-home/CyclesHome";

/* === Route === */

export const Route = createFileRoute("/cycles")({
  component: CyclesHome,
});
