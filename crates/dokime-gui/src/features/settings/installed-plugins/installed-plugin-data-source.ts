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
import { listNativeAdmittedPlugins, listNativePlugins } from "@/lib/api/native-plugins";
import { getNativePluginSettings } from "@/lib/api/native-settings";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Public Functions === */

export async function getInstalledPluginDetail(session: SessionIdentity, pluginId: string): Promise<InstalledPluginDetail | null> {
  const plugins = await listNativeAdmittedPlugins(session);
  let plugin = plugins.find((candidate) => candidate.plugin_id === pluginId);
  if (!plugin) {
    const installations = await listNativePlugins(session);
    const entry = installations.find((candidate) => candidate.state !== "removal_pending" && candidate.metadata?.owners.includes(pluginId));
    if (entry?.metadata) {
      plugin = {
        plugin_id: pluginId,
        owner: { kind: "bundle", package: entry.metadata.identity.name, installation: entry.install_id, path: null },
        label: pluginId,
        version: entry.metadata.identity.version,
        enabled: entry.state === "installed",
        runtime: "code",
        status: entry.boot_error ? "failed" : entry.state === "disabled" ? "disabled" : "pending",
        error: entry.boot_error ?? entry.error,
      };
    }
  }
  if (!plugin) return null;
  const config = await getNativePluginSettings(session, pluginId);
  return { ...plugin, ...config };
}
