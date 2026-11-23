/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { cutKey } from "@/lib/rpc/canonical";
import { nativeFailure } from "@/lib/rpc/failure-details";
import { failureOf } from "@/lib/notification-details";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import type { InstallPluginProgress, PluginSource } from "@/lib/api/native-plugins";
import {
  installNativePlugin,
  listNativePlugins,
  removeNativePlugin,
  restartNativePlugin,
  setNativePluginEnabled,
} from "@/lib/api/native-plugins";
import { commands } from "@/lib/desktop/commands";
import { clearPluginNotifications, toast } from "@/lib/notifications";
import type { ManagedPlugin } from "@/lib/api/native-plugins";

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
      if (!title || !entry.failure) continue;
      const failure = pluginStartFailure(entry, session.identity.generation);
      const key = failure.id;
      if (seen.current.has(key)) continue;
      seen.current.add(key);
      toast.error(failure, {
        context: "background",
        description: entry.failure.message,
        source: entry.description?.id || entry.name,
        pluginSettings: entry.name,
        action: {
          label: "Open Plugin Settings",
          onClick: () => void navigate({ to: "/settings/plugins/$pluginId", params: { pluginId: entry.name } }),
        },
      });
    }
    const installations = new Set(query.data.map((entry) => entry.name));
    void clearPluginNotifications(session.identity.generation, installations).catch((error: unknown) =>
      toast.error(failureOf(error, "Notification History Unavailable"), { context: "background" }),
    );
  }, [query.data, session.identity.generation, session.identity.isCurrent, navigate]);
  return null;
}

export function pluginStartFailure(entry: ManagedPlugin, generation: string) {
  if (!entry.failure) throw new Error("Read Plugin Failure: Plugin Has No Reported Failure");
  const detail = { ...nativeFailure(entry.failure, entry.failure.code), plugin: entry.name };
  const identity = cutKey({
    ...detail,
    identifiers: Object.entries(detail.identifiers ?? {}).sort(([left], [right]) => left.localeCompare(right)),
  });
  return failureOf({ notificationFailure: detail }, "Plugin Unavailable", [generation, entry.name, entry.activation, identity].join(":"));
}

export function startNotification(entry: ManagedPlugin): string | null {
  return entry.failure ? "Plugin Unavailable" : null;
}

export function usePluginsQuery() {
  return useQuery(pluginsListQueryOptions(useNativeSession()));
}

export function pluginsListQueryOptions(session: ConnectedSession) {
  return {
    queryFn: () => listNativePlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
  };
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
  const mutation = useMutation({
    mutationFn: async () => {
      const release = await commands.desktopPluginReleaseCheck(session.identity.generation);
      if (!session.identity.isCurrent()) throw new Error("The backend changed while checking plugin releases.");
      return release;
    },
  });
  const unavailableReason = session.snapshot.connection.backend?.target
    ? null
    : "The host does not publish its build target. Install a verified bundle file until release selection is available.";
  return { ...mutation, unavailableReason };
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
  return useMutation(reloadPluginOptions(session));
}

export function reloadPluginOptions(session: ConnectedSession) {
  return {
    mutationFn: (name: string) => restartNativePlugin(session.identity, name),
    onError: (error: unknown) => toast.error(failureOf(error, "Plugin Reload Failed")),
    onSuccess: async () => {
      await session.queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  };
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
