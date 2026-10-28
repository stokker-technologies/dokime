/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createFileRoute } from "@tanstack/react-router";

import { PluginSettingsSurface } from "@/features/settings/PluginSettingsSurface";

export const Route = createFileRoute("/settings_/plugins/$pluginId")({
  component: PluginSettingsRoute,
});

function PluginSettingsRoute() {
  const { pluginId } = Route.useParams();
  return <PluginSettingsSurface key={pluginId} pluginId={pluginId} />;
}
