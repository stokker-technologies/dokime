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

/* === Local Imports === */

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
import { toast } from "@/lib/notifications";
import type { ManagedPlugin } from "@/lib/api/native-plugins";

/* === Query Keys === */

export const pluginQueryKeys = {
  all: ["settings", "plugins"] as const,
  list: (generation?: string) =>
    generation ? ([...pluginQueryKeys.all, "list", generation] as const) : ([...pluginQueryKeys.all, "list"] as const),
};

/* === Hooks === */

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

export function pluginsListQueryOptions(session: ConnectedSession) {
  return {
    queryFn: () => listNativePlugins(session.identity),
    queryKey: pluginQueryKeys.list(session.identity.generation),
  };
}

export function installPluginOptions(session: ConnectedSession, setProgress: (progress: InstallPluginProgress | null) => void) {
  return {
    mutationFn: (source: PluginSource) => {
      setProgress(null);
      return installNativePlugin(session.identity, source, { onProgress: setProgress });
    },
    onSuccess: () => session.queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
    onSettled: () => setProgress(null),
  };
}

export function pluginReleaseOptions(session: ConnectedSession) {
  return {
    mutationFn: async () => {
      const release = await commands.desktopPluginReleaseCheck(session.identity.generation);
      if (!session.identity.isCurrent()) {
        throw new Error("Reconnect to the backend before checking plugin releases.");
      }
      return release;
    },
  };
}

export function removePluginOptions(session: ConnectedSession) {
  return {
    mutationFn: (installId: string) => removeNativePlugin(session.identity, installId),
    onSuccess: () => session.queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
  };
}

/** Reloads A Plugin And Its Observers Using Saved Settings. */
export function reloadPluginOptions(session: ConnectedSession) {
  return {
    mutationFn: (name: string) => restartNativePlugin(session.identity, name),
    onError: (error: unknown) => toast.error(failureOf(error, "Plugin Reload Failed")),
    onSuccess: () => session.queryClient.invalidateQueries({ queryKey: ["settings"] }),
  };
}

export function enablePluginOptions(session: ConnectedSession) {
  return {
    mutationFn: ({ installId, enabled }: { installId: string; enabled: boolean }) =>
      setNativePluginEnabled(session.identity, installId, enabled),
    onSuccess: () => session.queryClient.invalidateQueries({ queryKey: pluginQueryKeys.list() }),
  };
}
