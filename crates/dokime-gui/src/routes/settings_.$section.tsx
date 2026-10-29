/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createFileRoute } from "@tanstack/react-router";

import { SettingsSurface } from "@/features/settings/SettingsSurface";

export const Route = createFileRoute("/settings_/$section")({
  component: SettingsSectionRoute,
});

function SettingsSectionRoute() {
  const { section } = Route.useParams();
  return <SettingsSurface section={section} />;
}
