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
    const entry = installations.find(
      (candidate) => candidate.boot_error && candidate.last_boot?.state === "failed" && candidate.last_boot.failure?.plugin_id === pluginId,
    );
    if (entry?.last_boot?.state === "failed" && entry.last_boot.failure) {
      const failure = entry.last_boot.failure;
      plugin = {
        plugin_id: pluginId,
        label: failure.plugin_name,
        version: entry.metadata?.identity.version ?? "",
        owner: { kind: "bundle", package: failure.plugin_name, path: null, installation: entry.install_id },
        enabled: true,
        runtime: "code",
        status: "failed",
        error: failure.message,
      };
    }
  }
  if (!plugin) return null;
  const config = await getNativePluginSettings(session, pluginId);
  return { ...plugin, ...config };
}
