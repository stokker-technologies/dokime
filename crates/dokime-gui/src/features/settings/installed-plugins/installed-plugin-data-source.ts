/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin reads that compose more than one native call. */

/* === Local Imports === */

import type { InstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { listNativeAdmittedPlugins } from "@/lib/api/native-plugins";
import { getNativePluginSettings } from "@/lib/api/native-settings";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Public Functions === */

export async function getInstalledPluginDetail(
  session: SessionIdentity,
  pluginId: string,
): Promise<InstalledPluginDetail | null> {
  const plugins = await listNativeAdmittedPlugins(session);
  const plugin = plugins.find((candidate) => candidate.plugin_id === pluginId);
  if (!plugin) return null;
  const config = await getNativePluginSettings(session, pluginId);
  return { ...plugin, ...config };
}
