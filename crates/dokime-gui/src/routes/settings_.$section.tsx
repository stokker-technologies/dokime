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

import { SettingsSurface } from "@/features/settings/SettingsSurface";

/* === Route === */

export const Route = createFileRoute("/settings_/$section")({
  component: SettingsSectionRoute,
});

/* === Components === */

function SettingsSectionRoute() {
  const { section } = Route.useParams();
  return <SettingsSurface section={section} />;
}
