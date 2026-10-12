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

import { PluginSettingsSurface } from "@/features/settings/PluginSettingsSurface";

/* === Route === */

export const Route = createFileRoute("/settings_/plugins/$pluginId")({
  component: PluginSettingsRoute,
});

/* === Components === */

function PluginSettingsRoute() {
  const { pluginId } = Route.useParams();
  return <PluginSettingsSurface key={pluginId} pluginId={pluginId} />;
}
