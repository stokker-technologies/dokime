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

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import type { SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { settingsQueryKeys } from "@/features/settings/settings-queries";
import { listNativeAdmittedPlugins } from "@/lib/api/native-plugins";
import { updateNativePluginSecret, updateNativePluginSettings } from "@/lib/api/native-settings";
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
  const session = useNativeSession();
  return useQuery({
    queryFn: () => listNativeAdmittedPlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
    staleTime: 15_000,
  });
}

export function useInstalledPluginDetailQuery(pluginId: string, ownerKey: string) {
  const session = useNativeSession();
  return useQuery({
    enabled: Boolean(pluginId),
    queryFn: () => {
      if (!pluginId) {
        return null;
      }
      return getInstalledPluginDetail(session.identity, pluginId);
    },
    queryKey: [...pluginQueryKeys.detail(pluginId, session.identity.generation), ownerKey],
    staleTime: 15_000,
  });
}

export function useInstalledPluginSettingsMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ pluginId, values }: { pluginId: string; values: SettingValues }) =>
      updateNativePluginSettings(session.identity, pluginId, values),
    onSuccess: async (_settings, variables) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
        queryClient.invalidateQueries({ queryKey: settingsQueryKeys.frame() }),
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.detail(variables.pluginId) }),
        queryClient.invalidateQueries({ queryKey: appearanceQueryKey }),
      ]);
    },
  });
}

export function useInstalledPluginSecretMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    gcTime: 0,
    mutationFn: ({ secretId, pluginId, secret }: { secretId: string; pluginId: string; secret: string }) =>
      updateNativePluginSecret(session.identity, pluginId, secretId, secret),
    onSuccess: async (_settings, variables) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
        queryClient.invalidateQueries({ queryKey: settingsQueryKeys.frame() }),
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.detail(variables.pluginId) }),
      ]);
    },
  });
}
