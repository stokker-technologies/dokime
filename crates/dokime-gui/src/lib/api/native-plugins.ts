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

import { Channel } from "@tauri-apps/api/core";

/* === Local Imports === */

import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { commands } from "@/lib/desktop/commands";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
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

export async function consentNativePlugin(session: SessionIdentity, installId: string): Promise<PluginEntry> {
  const current = await rpc.pluginsGetInstallation(session, { installId });
  const installation = await rpc.pluginsConsent(session, {
    installId,
    expectedSha256: current.sha256,
  });
  return mapPluginInstallation(installation);
}

export async function removeNativePlugin(session: SessionIdentity, installId: string): Promise<void> {
  await rpc.pluginsRemove(session, { installId });
}

export async function setNativePluginEnabled(session: SessionIdentity, installId: string, enabled: boolean): Promise<PluginEntry> {
  return mapPluginInstallation(await rpc.pluginsSetEnabled(session, { installId, enabled }));
}

/* === Plugin staging === */

/** Progress phases {@link stageNativePlugin} reports through `onProgress`, in order. */
export type StagePluginProgress = { phase: "reading" } | { phase: "uploading"; sent: number; total: number } | { phase: "inspecting" };

export interface StagePluginOptions {
  onProgress?(progress: StagePluginProgress): void;
}

export type PluginSource = "pick" | "drop" | { release: string };

/** Render a byte count the way staging progress reports it: decimal megabytes. */
export function formatStagePluginBytes(bytes: number): string {
  const megabytes = Math.round((bytes / 1_000_000) * 10) / 10;
  return Number.isInteger(megabytes) ? `${megabytes} MB` : `${megabytes.toFixed(1)} MB`;
}

/**
 * Stage an inert plugin archive; activation remains a separate consent command.
 *
 * The shell obtains the bundle itself, from its native picker or from the
 * native file drop it just recorded, then reads and hashes it once and streams
 * it to the backend; no bundle bytes cross the webview. `null` means the
 * operator cancelled the picker.
 */
export async function stageNativePlugin(
  session: SessionIdentity,
  source: PluginSource,
  options: StagePluginOptions = {},
): Promise<PluginEntry | null> {
  const progress = new Channel<StagePluginProgress>((message) => options.onProgress?.(message));
  const installId =
    typeof source === "object"
      ? await commands.desktopPluginReleaseStage(session.generation, source.release, progress)
      : await commands.desktopPluginStageFile(session.generation, source === "drop", progress);
  if (installId === null) return null;
  if (!session.isCurrent()) throw new Error("The connection changed while the plugin was being staged.");
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
  const stagedAt = timestampIso(installation.stagedAt);
  if (!stagedAt) {
    throw new Error(`Native plugin ${installation.installId} has no valid staged timestamp.`);
  }

  return {
    install_id: installation.installId,
    sha256: installation.sha256,
    staged_at: stagedAt,
    consented_at: timestampIso(installation.consentedAt),
    state: mapPluginState(installation.state),
    last_boot: mapPluginLastBoot(installation.lastBoot),
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
          environment_references: installation.metadata.environmentReferences.map((reference) => ({
            owner: reference.owner,
            setting: reference.setting,
            variable: reference.variable,
          })),
        }
      : null,
    error: installation.error ?? null,
    active: installation.active,
    restart_required: installation.restartRequired,
    boot_error: installation.bootError ?? null,
  };
}

function mapPluginState(state: PluginInstallationState): PluginEntry["state"] {
  switch (state?.HarmosVariant.case) {
    case "Staged":
      return "staged";
    case "Installed":
      return "installed";
    case "Disabled":
      return "disabled";
    case "RemovalPending":
      return "removal_pending";
    default:
      throw new Error("Native plugin returned an unknown installation state.");
  }
}

function mapPluginLastBoot(lastBoot: PluginInstallation["lastBoot"]): PluginEntry["last_boot"] {
  if (!lastBoot) return null;
  const at = timestampIso(lastBoot.at);
  if (!at) throw new Error("Native plugin returned a last-boot result without a timestamp.");
  switch (lastBoot.result?.HarmosVariant.case) {
    case "Loaded":
      return { state: "loaded", at };
    case "Error":
      return { state: "failed", error: lastBoot.result.HarmosVariant.value.field0, at };
    default:
      throw new Error("Native plugin returned a last-boot result without an outcome.");
  }
}

function mapAdmittedPlugin(artifact: AdmittedPlugin): RuntimeInstalledPluginRecord {
  const owner = artifact.owner?.HarmosVariant;
  let projectedOwner: RuntimeInstalledPluginRecord["owner"] = null;
  if (owner?.case === "Host") projectedOwner = { kind: "host" };
  if (owner?.case === "Bundle") {
    const bundle = owner.value.field0;
    if (!bundle) throw new Error("Admitted plugin omitted its bundle owner.");
    projectedOwner = {
      kind: "bundle",
      package: bundle.package,
      path: bundle.path ?? null,
      installation: bundle.installationId ?? null,
      content_digest: bundle.contentDigest,
    };
  }
  if (owner?.case === "Embedded") {
    const embedded = owner.value.field0;
    if (!embedded) throw new Error("Admitted plugin omitted its embedded owner.");
    projectedOwner = { kind: "embedded", generation: embedded.generation };
  }
  return {
    plugin_id: artifact.pluginId,
    owner: projectedOwner,
    label: artifact.label,
    version: artifact.version,
    enabled: artifact.enabled,
    // The original view model only has a code-defined runtime kind. Native
    // admitted artifacts are the code catalog exposed by Plugins, so preserve
    // that existing badge contract while retaining the owner discriminator.
    runtime: "code",
    status: artifact.enabled ? "ready" : "disabled",
    error: null,
  };
}
