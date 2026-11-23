/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin queries. */

/* === External Imports === */

/* === Local Imports === */

import type { ConnectedSession } from "@/app/native-session/session";
import type { SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { settingsQueryKeys } from "@/features/settings/settings-queries";
import { admittedPluginRecords } from "@/lib/api/native-plugins";
import { pluginsListQueryOptions } from "../plugins/plugin-queries";
import { getNativePluginSettings, updateNativePluginSettings } from "@/lib/api/native-settings";
import { appearanceQueryKey } from "@/app/desktop-appearance";

/* === Query Keys === */

export const pluginQueryKeys = {
  all: ["settings", "installed-plugins"] as const,
  detail: (pluginId: string | null, generation?: string) =>
    generation
      ? ([...pluginQueryKeys.all, "detail", generation, pluginId] as const)
      : ([...pluginQueryKeys.all, "detail", pluginId] as const),
};

/* === Hooks === */

export function installedPluginDetailsOptions(session: ConnectedSession, pluginId: string, ownerKey: string) {
  return {
    enabled: Boolean(pluginId),
    queryFn: async () => {
      const entries = await session.queryClient.query({ ...pluginsListQueryOptions(session), staleTime: Number.POSITIVE_INFINITY });
      const plugin = admittedPluginRecords(entries).find((candidate) => candidate.plugin_id === pluginId);
      if (!plugin) {
        return null;
      }
      const config = await getNativePluginSettings(session.identity, pluginId);
      return { ...plugin, ...config };
    },
    queryKey: [...pluginQueryKeys.detail(pluginId, session.identity.generation), ownerKey],
  };
}

export function installedPluginSettingsOptions(session: ConnectedSession) {
  const queryClient = session.queryClient;
  return {
    mutationFn: ({ pluginId, values }: { pluginId: string; values: SettingValues }) =>
      updateNativePluginSettings(session.identity, pluginId, values),
    onSuccess: async (_settings: unknown, variables: { pluginId: string; values: SettingValues }) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: pluginsListQueryOptions(session).queryKey }),
        queryClient.invalidateQueries({ queryKey: settingsQueryKeys.frame() }),
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.detail(variables.pluginId, session.identity.generation) }),
        queryClient.invalidateQueries({ queryKey: appearanceQueryKey }),
      ]);
    },
  };
}
