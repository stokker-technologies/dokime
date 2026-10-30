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
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { restartDesktopApp } from "@/features/settings/plugins/plugin-data-source";
import type { InstallPluginProgress, PluginSource } from "@/lib/api/native-plugins";
import { installNativePlugin, listNativePlugins, removeNativePlugin, setNativePluginEnabled } from "@/lib/api/native-plugins";
import { commands } from "@/lib/desktop/commands";
import { toast } from "@/lib/notifications";

/* === Query Keys === */

export const pluginQueryKeys = {
  all: ["settings", "plugins"] as const,
  list: (generation?: string) =>
    generation ? ([...pluginQueryKeys.all, "list", generation] as const) : ([...pluginQueryKeys.all, "list"] as const),
};

/* === Hooks === */

export function PluginStartNotifications() {
  const session = useNativeSession();
  const query = usePluginsQuery();
  const navigate = useNavigate();
  const seen = useRef(new Set<string>());
  useEffect(() => {
    for (const entry of query.data ?? []) {
      if (!entry.boot_failed || entry.last_boot?.state !== "failed") continue;
      const key = `${session.identity.generation}:${entry.install_id}:${entry.last_boot.at}`;
      if (seen.current.has(key)) continue;
      seen.current.add(key);
      const failure = entry.last_boot.failure;
      toast.error("Plugin Could Not Start", {
        description: failure.message,
        source: failure.plugin_name,
        pluginSettings: entry.install_id,
        action: entry.metadata?.owners.length
          ? {
              label: "Open Plugin Settings",
              onClick: () => void navigate({ to: "/settings/plugins/$pluginId", params: { pluginId: entry.install_id } }),
            }
          : undefined,
      });
    }
  }, [query.data, session.identity.generation, navigate]);
  return null;
}

export function usePluginsQuery() {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => listNativePlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
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
