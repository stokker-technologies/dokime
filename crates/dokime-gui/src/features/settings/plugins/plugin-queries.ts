/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { failureOf } from "@/lib/notification-details";
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
  reloadNativePlugin,
  setNativePluginEnabled,
} from "@/lib/api/native-plugins";
import { commands } from "@/lib/desktop/commands";
import { clearPluginNotifications, toast } from "@/lib/notifications";
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
    if (!query.data || !session.identity.isCurrent()) return;
    for (const entry of query.data) {
      const title = startNotification(entry);
      if (!title || entry.last_boot?.state !== "failed") continue;
      const key = `${session.identity.generation}:${entry.install_id}:${entry.last_boot.at}`;
      if (seen.current.has(key)) continue;
      seen.current.add(key);
      const failure = entry.last_boot.failure;
      toast.error(pluginStartFailure(entry, session.identity.generation), {
        context: "background",
        description: failure.message,
        source: failure.plugin_name,
        pluginSettings: entry.install_id,
        missingSecret: failure.missing_secret ?? undefined,
        consequences: failure.consequences,
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
    const installations = new Set(query.data.map((entry) => entry.install_id));
    void clearPluginNotifications(session.identity.generation, installations).catch((error: unknown) =>
      toast.error(failureOf(error, "Notification History Unavailable"), { context: "background" }),
    );
  }, [query.data, session.identity.generation, session.identity.isCurrent, navigate]);
  return null;
}

export function pluginStartFailure(entry: PluginEntry, generation: string) {
  if (entry.last_boot?.state !== "failed") throw new Error("Plugin has no failed startup.");
  return failureOf(
    entry.last_boot.failure,
    startNotification(entry) ?? "Plugin Startup Failed",
    [generation, entry.install_id, entry.last_boot.at].join(":"),
  );
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

/** Reloads a plugin and its observers using saved settings. */
export function useReloadPluginMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (target: { installId: string } | { pluginId: string }) => reloadNativePlugin(session.identity, target),
    onError: (error) => toast.error(failureOf(error, "Plugin Reload Failed")),
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
