/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The desktop IPC surface: native desktop operations, backend connections,
   startup status, account login, and updates. Every command declared in
   `src-tauri/src/bindings/mod.rs` appears here once, under the same name and
   with the same arguments in the same order. */

/* === External Imports === */

import { type Channel, invoke as invokeCommand } from "@tauri-apps/api/core";

/* === Local Imports === */

import type { StagePluginProgress } from "@/lib/api/native-plugins";
import { isReportBuild } from "@/lib/ui-mode";
import type { BrandingStatus } from "./branding";
import type { BackendProfiles } from "./connection";
import type { ArtifactDeliveryRequest, ArtifactSource, Delivered, DesktopSaveDocument } from "./delivery";
import type { GitlabAuthStatus, GitlabInstallResult, GitlabUpdateCheck } from "./gitlab";
import type { BackendSelection, DesktopLaunch, DesktopSessionSnapshot } from "./session";
import type { ParameterOverrides } from "./settings";

/* === Private Functions === */

/**
 * Every desktop command goes through here, so one check covers the surface.
 *
 * The frozen report bundles parts of this module because it shares components
 * with the application; none of those paths may reach the IPC bridge. A report
 * build that gets this far is a defect in a caller's own guard, so it throws
 * rather than resolving with something invented.
 */
function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isReportBuild()) {
    return Promise.reject(new Error(`A frozen report cannot call the desktop command "${command}".`));
  }
  return invokeCommand<T>(command, args);
}

/* === Public Constants === */

export const commands = {
  /** Secret-free native launch preferences. */
  desktopSessionLaunch: () => invoke<DesktopLaunch>("desktop_session_launch"),
  /** Select the native desktop backend. */
  desktopSessionSelect: (selection: BackendSelection) => invoke<DesktopSessionSnapshot>("desktop_session_select", { selection }),
  desktopSessionDisconnect: () => invoke<DesktopSessionSnapshot>("desktop_session_disconnect"),
  desktopSessionStatus: () => invoke<DesktopSessionSnapshot>("desktop_session_status"),
  desktopProfilesList: () => invoke<BackendProfiles>("desktop_profiles_list"),
  /** Open the failed owned start's log; the shell supplies the path. */
  desktopOpenBackendLog: () => invoke<null>("desktop_open_backend_log"),
  /** Development only: delete the owned backend's storage and restart it. */
  desktopResetLocalStorage: (directory: string) => invoke<DesktopSessionSnapshot>("desktop_reset_local_storage", { directory }),
  /** Name the archive a local storage backup writes: beside the data root, or
   * where the operator picks; `null` when the operator cancels. */
  desktopStorageBackupDestination: (choose: boolean) => invoke<string | null>("desktop_storage_backup_destination", { choose }),
  /** After a failed owned start: archive the data root, reset it and restart. */
  desktopBackupResetLocalStorage: (archive: string) => invoke<DesktopSessionSnapshot>("desktop_backup_reset_local_storage", { archive }),
  desktopOpenPath: (path: string) => invoke<null>("desktop_open_path", { path }),
  desktopOpenTarget: (target: string) => invoke<null>("desktop_open_target", { target }),
  /** Save a verified document through the native file picker: the saved path, `null` when the operator cancels. */
  desktopSaveDocument: (generation: string, document: DesktopSaveDocument) =>
    invoke<string | null>("desktop_save_document", { generation, document }),
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
  /** Stage the bundle just dropped on the window, or pick one natively; `null` when the operator cancels. */
  desktopPluginStageFile: (generation: string, drop: boolean, progress: Channel<StagePluginProgress>) =>
    invoke<string | null>("desktop_plugin_stage_file", { generation, drop, progress }),
  desktopSetTitle: (title: string) => invoke<null>("desktop_set_title", { title }),
  /** The branding pack applied at boot and the one that applies on the next start. */
  desktopBrandingStatus: () => invoke<BrandingStatus>("desktop_branding_status"),
  /** Pick, validate and stage a branding pack zip; null when the picker is dismissed. */
  desktopBrandingChoose: () => invoke<BrandingStatus | null>("desktop_branding_choose"),
  /** Delete the pending and active packs; the compiled brand applies on the next start. */
  desktopBrandingRemove: () => invoke<BrandingStatus>("desktop_branding_remove"),
  /** Drain all runtime resources before requesting Tauri's non-cancellable restart. */
  desktopRestart: () => invoke<null>("desktop_restart"),
  desktopParametersRead: () => invoke<ParameterOverrides | null>("desktop_parameters_read"),
  desktopParametersWrite: (values: ParameterOverrides) => invoke<null>("desktop_parameters_write", { values }),
  desktopPickExecutionDirectory: () => invoke<string | null>("desktop_pick_execution_directory"),
  gitlabAuthStatus: () => invoke<GitlabAuthStatus>("gitlab_auth_status"),
  gitlabSignIn: () => invoke<GitlabAuthStatus>("gitlab_sign_in"),
  gitlabSignOut: () => invoke<GitlabAuthStatus>("gitlab_sign_out"),
  gitlabCheckForUpdates: () => invoke<GitlabUpdateCheck>("gitlab_check_for_updates"),
  gitlabInstallUpdate: () => invoke<GitlabInstallResult>("gitlab_install_update"),
};
