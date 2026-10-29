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

export const Route = createFileRoute("/settings")({
  component: SettingsRoute,
});

function SettingsRoute() {
  return <SettingsSurface />;
}
