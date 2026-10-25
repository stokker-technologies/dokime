/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { restartDesktopApp } from "@/features/settings/plugins/plugin-data-source";
import type { InstallPluginProgress, PluginSource } from "@/lib/api/native-plugins";
import { installNativePlugin, listNativePlugins, removeNativePlugin, setNativePluginEnabled } from "@/lib/api/native-plugins";
import { commands } from "@/lib/desktop/commands";

/* === Query Keys === */

export const pluginQueryKeys = {
  all: ["settings", "plugins"] as const,
  list: (generation?: string) =>
    generation ? ([...pluginQueryKeys.all, "list", generation] as const) : ([...pluginQueryKeys.all, "list"] as const),
};

/* === Hooks === */

export function usePluginsQuery() {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => listNativePlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
    staleTime: 5_000,
  });
}

export function useInstallPluginMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  const [progress, setProgress] = useState<InstallPluginProgress | null>(null);
  const mutation = useMutation({
    mutationFn: (source: PluginSource) => {
      setProgress(null);
      return installNativePlugin(session.identity, source, { onProgress: setProgress });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() });
    },
    onSettled: () => setProgress(null),
  });
  return { ...mutation, progress };
}

export function usePluginReleaseMutation() {
  const session = useNativeSession();
  return useMutation({
    mutationFn: async () => {
      const release = await commands.desktopPluginReleaseCheck(session.identity.generation);
      if (!session.identity.isCurrent()) throw new Error("The backend changed while checking plugin releases.");
      return release;
    },
  });
}

export function useRemovePluginMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (installId: string) => removeNativePlugin(session.identity, installId),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() });
    },
  });
}

export function useRestartAppMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => restartDesktopApp(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}

export function useSetPluginEnabledMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ installId, enabled }: { installId: string; enabled: boolean }) =>
      setNativePluginEnabled(session.identity, installId, enabled),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() });
    },
  });
}
