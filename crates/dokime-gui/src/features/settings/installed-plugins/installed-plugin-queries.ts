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
import { useEffect } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import type { SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { settingsQueryKeys } from "@/features/settings/settings-queries";
import { listNativeAdmittedPlugins } from "@/lib/api/native-plugins";
import { updateNativePluginSettings } from "@/lib/api/native-settings";
import { appearanceQueryKey } from "@/lib/appearance";
import type { PluginRestart } from "@/lib/types/view-contracts";
import { pluginQueryKeys as installationQueryKeys } from "@/features/settings/plugins/plugin-queries";
import { useHubStore } from "@/stores/hub-store";

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
  });
}

export function useInstalledPluginDetailQuery(pluginId: string, ownerKey: string) {
  const session = useNativeSession();
  const client = useQueryClient();
  const operations = useHubStore((state) => state.operations.map((operation) => `${operation.id}:${operation.status}`).join());
  // biome-ignore lint/correctness/useExhaustiveDependencies: operation transitions refresh settings applied by another GUI
  useEffect(() => {
    void client.invalidateQueries({ queryKey: pluginQueryKeys.detail(pluginId, session.identity.generation) });
  }, [client, pluginId, session.identity.generation, operations]);
  return useQuery({
    enabled: Boolean(pluginId),
    queryFn: () => {
      if (!pluginId) {
        return null;
      }
      return getInstalledPluginDetail(session.identity, pluginId);
    },
    queryKey: [...pluginQueryKeys.detail(pluginId, session.identity.generation), ownerKey],
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
        queryClient.invalidateQueries({ queryKey: pluginQueryKeys.detail(variables.pluginId, session.identity.generation) }),
        queryClient.invalidateQueries({ queryKey: appearanceQueryKey }),
      ]);
    },
  });
}

/**
 * While an explicit reload waits to restart its program, re-read the plugin
 * views whenever the backend's operations change: the restart runs as one,
 * and a failed restart is recorded like a failed startup.
 */
export function usePluginRestartRefresh(restart: PluginRestart | undefined) {
  const queryClient = useQueryClient();
  const operations = useHubStore((state) => state.operations.map((operation) => `${operation.id}:${operation.status}`).join());
  // biome-ignore lint/correctness/useExhaustiveDependencies: a changed operation set is what re-reads the views
  useEffect(() => {
    if (!restart || restart === "idle") return;
    void Promise.all([
      queryClient.invalidateQueries({ queryKey: pluginQueryKeys.all }),
      queryClient.invalidateQueries({ queryKey: installationQueryKeys.all }),
    ]);
  }, [operations, queryClient, restart]);
}
