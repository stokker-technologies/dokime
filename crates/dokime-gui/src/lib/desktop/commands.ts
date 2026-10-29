/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The desktop IPC surface: native desktop operations, backend connections,
   startup status and public updates. Every command declared in
   `src-tauri/src/bindings/mod.rs` appears here once, under the same name and
   with the same arguments in the same order. */

/* === External Imports === */

import { type Channel, invoke as invokeCommand } from "@tauri-apps/api/core";

/* === Local Imports === */

import type { InstallPluginProgress } from "@/lib/api/native-plugins";
import { isReportBuild } from "@/lib/ui-mode";
import type { ArtifactDeliveryRequest, ArtifactSource, Delivered, DesktopSaveDocument } from "./delivery";
import type { OrganizationStatus } from "./organization";
import type { BackendSelection, DesktopLaunch, DesktopSessionSnapshot } from "./session";
import type { ParameterOverrides } from "./settings";
import { settleUnlessStopping } from "./stopping";
import type { PluginRelease, UpdateCheck, UpdateSelection, UpdateStatus } from "./updates";

/* === Private Functions === */

/**
 * Every desktop command goes through here, so one check covers the surface.
 *
 * The frozen report bundles parts of this module because it shares components
 * with the application; none of those paths may reach the IPC bridge. A report
 * build that gets this far is a defect in a caller's own guard, so it throws
 * rather than resolving with something invented.
 */
export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isReportBuild()) {
    return Promise.reject(new Error(`A frozen report cannot call the desktop command "${command}".`));
  }
  // Once the desktop stops, a refused or interrupted command belongs to the stop.
  return invokeCommand<T>(command, args).catch(settleUnlessStopping<T>);
}

/* === Public Constants === */

export const commands = {
  desktopPairingCreate: (label: string, endpoint: string) => invoke<string | null>("desktop_pairing_create", { label, endpoint }),
  /** Secret-free native launch preferences. */
  desktopSessionLaunch: () => invoke<DesktopLaunch>("desktop_session_launch"),
  /** Select the native desktop backend. */
  desktopSessionSelect: (selection: BackendSelection) => invoke<DesktopSessionSnapshot>("desktop_session_select", { selection }),
  desktopSessionDisconnect: () => invoke<DesktopSessionSnapshot>("desktop_session_disconnect"),
  desktopSessionStatus: () => invoke<DesktopSessionSnapshot>("desktop_session_status"),
  /** Open the failed owned start's log; the shell supplies the path. */
  desktopOpenBackendLog: () => invoke<null>("desktop_open_backend_log"),
  /** Development only: delete the owned backend's storage and restart it. */
  desktopResetLocalStorage: (directory: string) => invoke<DesktopSessionSnapshot>("desktop_reset_local_storage", { directory }),
  desktopOpenPath: (path: string) => invoke<null>("desktop_open_path", { path }),
  desktopOpenTarget: (target: string) => invoke<null>("desktop_open_target", { target }),
  /** Save a verified document through the native file picker: the saved path, `null` when the operator cancels. */
  desktopSaveDocument: (generation: string, document: DesktopSaveDocument) =>
    invoke<string | null>("desktop_save_document", { generation, document }),
  desktopPickSecretArchive: (generation: string) => invoke<string | null>("desktop_pick_secret_archive", { generation }),
  /**
   * Write a verified diagnostics bundle into the operator's Downloads folder;
   * resolves with the saved path. The `document` kind must be
   * `diagnostics_bundle` and its filename must already name a `.zip`.
   */
  desktopSaveDiagnosticsBundle: (generation: string, document: DesktopSaveDocument) =>
    invoke<string>("desktop_save_diagnostics_bundle", { generation, document }),
  /** Stream one verified artifact, execution output or sealed evidence bundle to a native destination. */
  desktopArtifactDeliver: (generation: string, request: ArtifactDeliveryRequest) =>
    invoke<Delivered | null>("desktop_artifact_deliver", { generation, request }),
  /** Materialize one verified artifact in the private media cache; returns its preview URL path. */
  desktopMediaPrepare: (generation: string, source: ArtifactSource) => invoke<string>("desktop_media_prepare", { generation, source }),
  /** Release one generation's media cache. */
  desktopMediaRelease: (generation: string) => invoke<null>("desktop_media_release", { generation }),
  /** Patch an MP4 recording's duration in place and upload it; returns the artifact id. */
  desktopRecordingFinish: (generation: string, durationMs: number, mediaType: string) =>
    invoke<string>("desktop_recording_finish", { generation, durationMs, mediaType }),
  /** Install the bundle just dropped on the window, or pick one natively; `null` when the operator cancels. */
  desktopPluginInstallFile: (generation: string, drop: boolean, progress: Channel<InstallPluginProgress>) =>
    invoke<string | null>("desktop_plugin_install_file", { generation, drop, progress }),
  desktopSetTitle: (title: string) => invoke<null>("desktop_set_title", { title }),
  /** The organization pack applied at boot and the one that applies on the next start. */
  desktopOrganizationStatus: () => invoke<OrganizationStatus>("desktop_organization_status"),
  /** Pick, validate and stage an organization pack zip; null when the picker is dismissed. */
  desktopOrganizationChoose: () => invoke<OrganizationStatus | null>("desktop_organization_choose"),
  /** Delete the pending and active packs; the compiled brand applies on the next start. */
  desktopOrganizationRemove: () => invoke<OrganizationStatus>("desktop_organization_remove"),
  /** Drain all runtime resources before requesting Tauri's non-cancellable restart. */
  desktopRestart: () => invoke<null>("desktop_restart"),
  desktopParametersRead: () => invoke<ParameterOverrides | null>("desktop_parameters_read"),
  desktopParametersWrite: (values: ParameterOverrides) => invoke<null>("desktop_parameters_write", { values }),
  desktopUpdatesStatus: () => invoke<UpdateStatus>("desktop_updates_status"),
  desktopUpdatesConfigure: (selection: UpdateSelection) => invoke<UpdateStatus>("desktop_updates_configure", { selection }),
  desktopUpdatesCheck: () => invoke<UpdateCheck>("desktop_updates_check"),
  desktopUpdatesInstall: (candidate: string) => invoke<{ version: string }>("desktop_updates_install", { candidate }),
  desktopPluginReleaseCheck: (generation: string) => invoke<PluginRelease>("desktop_plugin_release_check", { generation }),
  desktopPluginReleaseInstall: (generation: string, candidate: string, progress: Channel<InstallPluginProgress>) =>
    invoke<string>("desktop_plugin_release_install", { generation, candidate, progress }),
};
