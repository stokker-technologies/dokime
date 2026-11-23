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

import { useQuery } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import type { SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { settingsQueryKeys } from "@/features/settings/settings-queries";
import { listNativeAdmittedPlugins } from "@/lib/api/native-plugins";
import { updateNativePluginSettings } from "@/lib/api/native-settings";
import { appearanceQueryKey } from "@/lib/appearance";

/* === Query Keys === */

export const pluginQueryKeys = {
  all: ["settings", "installed-plugins"] as const,
  detail: (pluginId: string | null, generation?: string) =>
    generation
      ? ([...pluginQueryKeys.all, "detail", generation, pluginId] as const)
      : ([...pluginQueryKeys.all, "detail", pluginId] as const),
  list: (generation?: string) =>
    generation ? ([...pluginQueryKeys.all, "list", generation] as const) : ([...pluginQueryKeys.all, "list"] as const),
};

/* === Hooks === */

export function useInstalledPluginsQuery() {
  return useQuery(installedPluginsListOptions(useNativeSession()));
}

export function installedPluginsListOptions(session: ConnectedSession) {
  return {
    queryFn: () => listNativeAdmittedPlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
  };
}

export function installedPluginDetailsOptions(session: ConnectedSession, pluginId: string, ownerKey: string) {
  return {
    enabled: Boolean(pluginId),
    queryFn: () => {
      if (!pluginId) {
        return null;
      }
      return getInstalledPluginDetail(session.identity, pluginId);
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
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
        queryClient.invalidateQueries({ queryKey: settingsQueryKeys.frame() }),
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.detail(variables.pluginId, session.identity.generation) }),
        queryClient.invalidateQueries({ queryKey: appearanceQueryKey }),
      ]);
    },
  };
}
