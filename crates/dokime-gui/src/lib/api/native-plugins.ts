/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native plugin and admitted-artifact adapters for the original settings UI. */

/* === External Imports === */

import { PluginInstallationState as InstallationState } from "@host/dokime/client/v1/plugins_pb.js";
import { PluginState } from "@host/dokime/client/v1/plugins_pb.js";
import { Channel } from "@tauri-apps/api/core";

/* === Local Imports === */

import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { commands } from "@/lib/desktop/commands";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { PluginEntry } from "@/lib/types/view-contracts";
import { NATIVE_MAX_PAGES, NATIVE_SMALL_PAGE_SIZE, NativeApiLimitError, safeNumber, timestampIso } from "./native-values";

/* === Types === */

type PluginInstallation = Wire<Awaited<ReturnType<typeof rpc.pluginsGetInstallation>>>;
type PluginInstallationState = PluginInstallation["state"];
type AdmittedPlugin = Wire<Awaited<ReturnType<typeof rpc.pluginsListAdmittedPlugins>>["plugins"][number]>;

/* === Plugin installations === */

export async function listNativePlugins(session: SessionIdentity): Promise<PluginEntry[]> {
  const installations: PluginInstallation[] = [];
  let after: { manifestSha256: string; afterInstallId: string } | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
    const response = await rpc.pluginsListInstallations(session, {
      pageSize: NATIVE_SMALL_PAGE_SIZE,
      ...(after ? { after } : {}),
    });
    installations.push(...response.installations);
    if (!response.next) return installations.map(mapPluginInstallation);

    const next = {
      manifestSha256: response.next.manifestSha256,
      afterInstallId: response.next.afterInstallId,
    };
    if (after && after.manifestSha256 === next.manifestSha256 && after.afterInstallId === next.afterInstallId) {
      throw new NativeApiLimitError("Plugins.ListInstallations", "cursor did not advance");
    }
    after = next;
  }

  throw new NativeApiLimitError("Plugins.ListInstallations", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function getNativePlugin(session: SessionIdentity, installId: string): Promise<PluginEntry> {
  return mapPluginInstallation(await rpc.pluginsGetInstallation(session, { installId }));
}

export async function removeNativePlugin(session: SessionIdentity, installId: string): Promise<void> {
  await rpc.pluginsRemove(session, { entry: installId });
}

export async function setNativePluginEnabled(session: SessionIdentity, installId: string, enabled: boolean): Promise<PluginEntry> {
  return mapPluginInstallation(await rpc.pluginsSetEnabled(session, { installId, enabled }));
}

/** Starts the installation again in the running backend, without restarting it. */
export async function reloadNativePlugin(session: SessionIdentity, target: { installId: string } | { pluginId: string }): Promise<void> {
  await rpc.pluginsReload(session, target);
}

/* === Plugin Installation === */

/** Progress phases {@link installNativePlugin} reports through `onProgress`, in order. */
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
): Promise<PluginEntry | null> {
  const progress = new Channel<InstallPluginProgress>((message) => options.onProgress?.(message));
  const installId =
    typeof source === "object"
      ? await commands.desktopPluginReleaseInstall(session.generation, source.release, progress)
      : await commands.desktopPluginInstallFile(session.generation, source === "drop", progress);
  if (installId === null) return null;
  if (!session.isCurrent()) throw new Error("The connection changed while the plugin was being installed.");
  return getNativePlugin(session, installId);
}

/* === Admitted Plugins === */

export async function listNativeAdmittedPlugins(session: SessionIdentity): Promise<RuntimeInstalledPluginRecord[]> {
  const artifacts: AdmittedPlugin[] = [];
  let after: { epoch: string; revision: bigint; afterPluginId: string } | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
    const response = await rpc.pluginsListAdmittedPlugins(session, {
      pageSize: NATIVE_SMALL_PAGE_SIZE,
      ...(after ? { after } : {}),
    });
    artifacts.push(...response.plugins);
    if (!response.next) return artifacts.map(mapAdmittedPlugin);

    const next = {
      epoch: response.next.epoch,
      revision: response.next.revision,
      afterPluginId: response.next.afterPluginId,
    };
    if (after && after.epoch === next.epoch && after.revision === next.revision && after.afterPluginId === next.afterPluginId) {
      throw new NativeApiLimitError("Plugins.ListAdmittedPlugins", "cursor did not advance");
    }
    after = next;
  }

  throw new NativeApiLimitError("Plugins.ListAdmittedPlugins", `more than ${NATIVE_MAX_PAGES} pages`);
}

/* === Mapping === */

function mapPluginInstallation(installation: PluginInstallation): PluginEntry {
  const installedAt = timestampIso(installation.installedAt);
  if (!installedAt) {
    throw new Error(`Native plugin ${installation.installId} has no valid installation timestamp.`);
  }
  const plugin = installation.plugin;

  return {
    install_id: installation.installId,
    sha256: installation.sha256,
    installed_at: installedAt,
    state: mapPluginState(installation.state),
    last_boot: null,
    metadata: installation.metadata
      ? {
          identity: installation.metadata.identity
            ? {
                name: installation.metadata.identity.name,
                version: installation.metadata.identity.version,
                description: installation.metadata.identity.description,
              }
            : (() => {
                throw new Error(`Native plugin ${installation.installId} metadata has no identity.`);
              })(),
          owners: installation.metadata.owners,
          size_bytes: safeNumber(installation.metadata.sizeBytes, "Plugins plugin size"),
        }
      : null,
    error: installation.error || plugin?.reason || null,
    active: plugin ? plugin.state === PluginState.READY : undefined,
    restart: plugin?.state === PluginState.STARTING || plugin?.state === PluginState.CONNECTING ? "restarting" : undefined,
    boot_failed: plugin ? plugin.state === PluginState.FAILED : undefined,
  };
}

function mapPluginState(state: PluginInstallationState): PluginEntry["state"] {
  switch (state) {
    case InstallationState.INSTALLED:
      return "installed";
    case InstallationState.DISABLED:
      return "disabled";
    case InstallationState.REMOVAL_PENDING:
      return "removal_pending";
    default:
      throw new Error("Native plugin returned an unknown installation state.");
  }
}

function mapAdmittedPlugin(artifact: AdmittedPlugin): RuntimeInstalledPluginRecord {
  const plugin = artifact.plugin;
  if (!plugin?.entry) throw new Error("Read Plugins: Missing Entry Identity");
  const placement = plugin.placement;
  return {
    plugin_id: plugin.entry,
    owner:
      placement.case === "managed"
        ? { kind: "bundle", package: placement.value.bundle, path: null, installation: plugin.entry }
        : placement.case === "remote"
          ? { kind: "remote", url: placement.value.url }
          : null,
    label: plugin.description?.id || plugin.entry,
    version: plugin.description?.version || "",
    enabled: plugin.state !== PluginState.UNAVAILABLE,
    runtime: placement.case || "code",
    status:
      plugin.state === PluginState.READY
        ? "ready"
        : plugin.state === PluginState.STARTING
          ? "starting"
          : plugin.state === PluginState.CONNECTING
            ? "connecting"
            : plugin.state === PluginState.FAILED
              ? "failed"
              : "unavailable",
    error: plugin.reason || null,
  };
}
