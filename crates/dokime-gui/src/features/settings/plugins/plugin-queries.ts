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
import type { InstallPluginProgress, PluginSource } from "@/lib/api/native-plugins";
import {
  installNativePlugin,
  listNativePlugins,
  removeNativePlugin,
  retryNativePlugin,
  setNativePluginEnabled,
} from "@/lib/api/native-plugins";
import { commands } from "@/lib/desktop/commands";
import { toast } from "@/lib/notifications";
import type { PluginEntry } from "@/lib/types/view-contracts";

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
      const title = startNotification(entry);
      if (!title || entry.last_boot?.state !== "failed") continue;
      const key = `${session.identity.generation}:${entry.install_id}:${entry.last_boot.at}`;
      if (seen.current.has(key)) continue;
      seen.current.add(key);
      const failure = entry.last_boot.failure;
      toast.error(title, {
        description: failure.message,
        source: failure.plugin_name,
        pluginSettings: entry.install_id,
        missingSecret: failure.missing_secret ?? undefined,
        consequences: failure.consequences,
        failure: { reason: failure.code ?? undefined, operation: "Plugin Startup", causes: [failure.message] },
        action:
          failure.missing_secret || (entry.metadata?.owners.length && entry.state !== "removal_pending")
            ? {
                label: failure.missing_secret ? "Set Secret" : "Open Plugin Settings",
                onClick: () =>
                  void navigate(
                    failure.missing_secret
                      ? { to: "/settings/$section", params: { section: "secrets" }, hash: `add-secret=${failure.missing_secret}` }
                      : { to: "/settings/plugins/$pluginId", params: { pluginId: entry.install_id } },
                  ),
              }
            : undefined,
      });
    }
  }, [query.data, session.identity.generation, navigate]);
  return null;
}

/**
 * A failed startup, and a refused restart that left the running generation in
 * place, notify once per recorded outcome.
 */
export function startNotification(entry: PluginEntry): string | null {
  if (entry.last_boot?.state !== "failed") return null;
  if (entry.boot_failed) return "Plugin Could Not Start";
  return entry.active ? "Plugin Restart Refused" : null;
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

/** Starts one installation again through the running backend; nothing else restarts. */
export function useRetryPluginMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (installId: string) => retryNativePlugin(session.identity, installId),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: pluginQueryKeys.all });
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
