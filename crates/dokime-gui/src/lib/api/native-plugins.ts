// Dokime — SPDX-License-Identifier: Apache-2.0

/* === External Imports === */

import { PluginState, type PluginEntry } from "@generated/host/model";
import { Channel } from "@tauri-apps/api/core";

/* === Local Imports === */

import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { commands } from "@/lib/desktop/commands";
import type { Wire } from "@/lib/rpc/canonical";
import { required } from "@/lib/rpc/canonical";
import { pluginsList, pluginsGet, pluginsDelete, pluginsSet, pluginsRestart } from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Types === */

export type ManagedPlugin = Wire<PluginEntry>;

/* === Public Functions === */

export async function listNativePlugins(session: SessionIdentity): Promise<ManagedPlugin[]> {
  return (await pluginsList(session, {})).entries;
}

export async function getNativePlugin(session: SessionIdentity, name: string): Promise<ManagedPlugin> {
  return required((await pluginsGet(session, { name })).entry, "Get Plugin");
}

export async function removeNativePlugin(session: SessionIdentity, name: string): Promise<void> {
  const current = await getNativePlugin(session, name);
  await pluginsDelete(session, { name, expectedRevision: current.revision });
}

export async function setNativePluginEnabled(session: SessionIdentity, name: string, enabled: boolean): Promise<ManagedPlugin> {
  const current = await getNativePlugin(session, name);
  return required((await pluginsSet(session, { name, value: { enabled }, expectedRevision: current.revision })).entry, "Set Plugin");
}

export async function restartNativePlugin(session: SessionIdentity, name: string): Promise<void> {
  const current = await getNativePlugin(session, name);
  await pluginsRestart(session, { name, expectedRevision: current.revision });
}

export type InstallPluginProgress = { phase: "reading" } | { phase: "uploading"; sent: number; total: number } | { phase: "installing" };

export interface InstallPluginOptions {
  onProgress?(progress: InstallPluginProgress): void;
}

export type PluginSource = "pick" | "drop" | { release: string };

/** Render a byte count the way staging progress reports it: decimal megabytes. */
export function formatInstallPluginBytes(bytes: number): string {
  const megabytes = Math.round((bytes / 1_000_000) * 10) / 10;
  return Number.isInteger(megabytes) ? `${megabytes} MB` : `${megabytes.toFixed(1)} MB`;
}

/**
 * Install a verified plugin archive; activation remains next boot.
 *
 * The shell obtains the bundle itself, from its native picker or from the
 * native file drop it just recorded, then reads and hashes it once and streams
 * it to the backend; no bundle bytes cross the webview. `null` means the
 * operator cancelled the picker.
 */
export async function installNativePlugin(
  session: SessionIdentity,
  source: PluginSource,
  options: InstallPluginOptions = {},
): Promise<ManagedPlugin | null> {
  const progress = new Channel<InstallPluginProgress>((message) => options.onProgress?.(message));
  const installId =
    typeof source === "object"
      ? await commands.desktopPluginReleaseInstall(session.generation, source.release, progress)
      : await commands.desktopPluginInstallFile(session.generation, source === "drop", progress);
  if (installId === null) {
    return null;
  }
  if (!session.isCurrent()) {
    throw new Error("The connection changed while the plugin was being installed. Reconnect before installing it again.");
  }
  return getNativePlugin(session, installId);
}

export function admittedPluginRecords(entries: ManagedPlugin[]): RuntimeInstalledPluginRecord[] {
  return entries.map((entry) => {
    const placement = entry.placement?.kind;
    return {
      plugin_id: entry.name,
      owner:
        placement?.case === "managed"
          ? { kind: "bundle", package: entry.name, installation: entry.name, path: null }
          : placement?.case === "remote"
            ? { kind: "remote", url: placement.value.url }
            : null,
      label: entry.description?.id || entry.name,
      version: entry.description?.version || "",
      enabled: entry.value?.enabled ?? false,
      runtime: placement?.case ?? "code",
      status: entry.failure
        ? "failed"
        : entry.state === PluginState.READY
          ? "ready"
          : entry.state === PluginState.CONNECTING
            ? "connecting"
            : entry.state === PluginState.STOPPING
              ? "stopping"
              : entry.state === PluginState.STOPPED
                ? "stopped"
                : "unavailable",
      error: entry.failure?.message ?? null,
    };
  });
}
