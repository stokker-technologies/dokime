/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createFileRoute } from "@tanstack/react-router";

import { CyclesHome } from "@/features/cycles-home/CyclesHome";

export const Route = createFileRoute("/cycles")({
  component: CyclesHome,
});
