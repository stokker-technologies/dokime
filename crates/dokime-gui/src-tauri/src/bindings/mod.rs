/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

macro_rules! call_handler {
    (sync, $handler:path, $($arg:ident),*) => { $handler($($arg),*) };
    (async, $handler:path, $($arg:ident),*) => { $handler($($arg),*).await };
}

macro_rules! native_adapter {
    (sync $name:ident ($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty => $handler:path) => {
        #[tauri::command]
        fn $name($($arg: $ty),*) -> $ret { call_handler!(sync, $handler, $($arg),*) }
    };
    (async $name:ident ($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty => $handler:path) => {
        #[tauri::command]
        async fn $name($($arg: $ty),*) -> $ret { call_handler!(async, $handler, $($arg),*) }
    };
}

macro_rules! native_commands {
    ($($mode:ident $name:ident [$($ctx:ident: $ctx_ty:ty),*] ($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty => $handler:path;)*) => {
        $(native_adapter!($mode $name ($($ctx: $ctx_ty,)* $($arg: $ty),*) -> $ret => $handler);)*
        pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
            tauri::generate_handler![$($name),*]
        }
    };
}

native_commands! {
    async desktop_pairing_create
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (label: String, endpoint: String) -> Result<Option<std::path::PathBuf>, crate::connection::ConnectionError>
        => crate::app::desktop::desktop_pairing_create;
    async desktop_backend_import_pairing
        [window: tauri::WebviewWindow, app: tauri::AppHandle, registry: tauri::State<'_, crate::connection::registry::Registry>, connection: tauri::State<'_, crate::connection::ConnectionSession>]
        (identifier: Option<String>) -> Result<Option<dokime::client::profiles::Document>, crate::connection::ConnectionError>
        => crate::connection::registry::desktop_backend_import_pairing;

    async desktop_backends_list [] () -> Result<dokime::client::profiles::Document, crate::connection::ConnectionError>
        => crate::connection::registry::desktop_backends_list;
    async desktop_backend_add [registry: tauri::State<'_, crate::connection::registry::Registry>]
        (profile: dokime::client::profiles::Profile) -> Result<dokime::client::profiles::Document, crate::connection::ConnectionError>
        => crate::connection::registry::desktop_backend_add;
    async desktop_backend_remove
        [registry: tauri::State<'_, crate::connection::registry::Registry>, owner: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        (identifier: String) -> Result<dokime::client::profiles::Document, crate::connection::ConnectionError>
        => crate::connection::registry::desktop_backend_remove;
    async desktop_backend_use
        [registry: tauri::State<'_, crate::connection::registry::Registry>, owner: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        (identifier: String) -> Result<crate::connection::ConnectionSnapshot, crate::connection::ConnectionError>
        => crate::connection::registry::desktop_backend_use;
    async desktop_settings_read [owner: tauri::State<'_, crate::native_session::DesktopSessionControl>, connection: tauri::State<'_, crate::connection::ConnectionSession>]
        (scope: dokime::domain::model::settings::application::SettingsScope) -> Result<dokime::domain::model::settings::application::SettingsSnapshot, crate::app::preferences::SettingsCommandError>
        => crate::app::preferences::desktop_settings_read;
    async desktop_settings_update [owner: tauri::State<'_, crate::native_session::DesktopSessionControl>, connection: tauri::State<'_, crate::connection::ConnectionSession>]
        (scope: dokime::domain::model::settings::application::SettingsScope, update: dokime::domain::model::settings::application::UpdateSettings)
        -> Result<dokime::domain::model::settings::application::SettingsSnapshot, crate::app::preferences::SettingsCommandError>
        => crate::app::preferences::desktop_settings_update;

    // Secret-free native launch preferences.
    sync desktop_session_launch
        [state: tauri::State<'_, crate::native_session::launch::DesktopLaunch>]
        () -> Result<crate::native_session::launch::DesktopLaunch, String>
        => crate::native_session::launch::desktop_session_launch;

    // Select the native desktop backend.
    async desktop_session_select
        [state: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        (selection: crate::native_session::BackendSelection) -> Result<crate::native_session::DesktopSessionSnapshot, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_session_select;

    async desktop_session_disconnect
        [state: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        () -> Result<crate::native_session::DesktopSessionSnapshot, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_session_disconnect;

    sync desktop_session_status
        [state: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        () -> Result<crate::native_session::DesktopSessionSnapshot, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_session_status;


    // Open the failed owned start's log; the shell supplies the path.
    async desktop_open_backend_log
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::native_session::DesktopSessionControl>, app: tauri::AppHandle]
        () -> Result<(), String>
        => crate::native_session::desktop_open_backend_log;

    // Development only: delete the owned backend's storage and restart it.
    async desktop_reset_local_storage
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        (directory: String) -> Result<crate::native_session::DesktopSessionSnapshot, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_reset_local_storage;

    // Name the archive a local storage backup writes: beside the data root, or
    // where the operator picks; `None` when the operator cancels.
    async desktop_storage_backup_destination
        [window: tauri::WebviewWindow, app: tauri::AppHandle]
        (choose: bool) -> Result<Option<String>, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_storage_backup_destination;

    // After a failed owned start: archive the data root, reset it and restart.
    async desktop_backup_reset_local_storage
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::native_session::DesktopSessionControl>]
        (archive: String) -> Result<crate::native_session::DesktopSessionSnapshot, crate::native_session::DesktopSessionError>
        => crate::native_session::desktop_backup_reset_local_storage;

    async desktop_open_path
        [app: tauri::AppHandle]
        (path: String) -> Result<(), String>
        => crate::app::desktop::desktop_open_path;

    async desktop_open_target
        [app: tauri::AppHandle]
        (target: String) -> Result<(), String>
        => crate::app::desktop::desktop_open_target;

    // Save a verified document through the native file picker.
    async desktop_save_document
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String, document: crate::app::desktop::DesktopSaveDocument) -> Result<Option<String>, String>
        => crate::app::desktop::desktop_save_document;

    async desktop_pick_secret_archive
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String) -> Result<Option<String>, String>
        => crate::app::desktop::desktop_pick_secret_archive;

    // Write a verified diagnostics bundle into the operator's Downloads folder;
    // returns the saved path.
    async desktop_save_diagnostics_bundle
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String, document: crate::app::desktop::DesktopSaveDocument) -> Result<String, String>
        => crate::app::desktop::desktop_save_diagnostics_bundle;

    // Stream one verified artifact, execution output or cycle bundle to a
    // native destination.
    async desktop_artifact_deliver
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String, request: crate::app::desktop::ArtifactDeliveryRequest) -> Result<Option<crate::app::desktop::Delivered>, crate::app::content::DeliveryError>
        => crate::app::desktop::desktop_artifact_deliver;

    // Materialize one verified artifact in the private media cache and return its preview URL path.
    async desktop_media_prepare
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String, source: crate::app::desktop::ArtifactSource) -> Result<String, String>
        => crate::app::media::desktop_media_prepare;

    // Release one generation's media cache.
    async desktop_media_release
        [state: tauri::State<'_, crate::connection::ConnectionSession>, app: tauri::AppHandle]
        (generation: String) -> Result<(), String>
        => crate::app::media::desktop_media_release;

    // Patch an MP4 recording's duration in place and upload it; returns the artifact id.
    async desktop_recording_finish
        [window: tauri::WebviewWindow, bridge: tauri::State<'_, crate::rpc::Bridge>]
        (generation: String, duration_ms: u64, media_type: String) -> Result<String, String>
        => crate::app::recording::desktop_recording_finish;

    // Stage the bundle just dropped on the window, or pick one natively; `None` when the operator cancels.
    async desktop_plugin_install_file
        [window: tauri::WebviewWindow, state: tauri::State<'_, crate::connection::ConnectionSession>, dropped: tauri::State<'_, crate::app::plugins::DroppedFiles>, app: tauri::AppHandle]
        (generation: String, drop: bool, progress: tauri::ipc::Channel<crate::app::plugins::InstallProgress>) -> Result<Option<String>, String>
        => crate::app::plugins::desktop_plugin_install_file;

    sync desktop_set_title
        [window: tauri::WebviewWindow]
        (title: String) -> Result<(), String>
        => crate::app::desktop::desktop_set_title;

    // The organization pack applied at boot and the one that applies on the next start.
    sync desktop_organization_status
        [state: tauri::State<'_, crate::app::organization::OrganizationState>]
        () -> Result<crate::app::organization::OrganizationStatus, String>
        => crate::app::organization::desktop_organization_status;

    // Pick, validate and stage a organization pack zip; None when the picker is dismissed.
    async desktop_organization_choose
        [app: tauri::AppHandle]
        () -> Result<Option<crate::app::organization::OrganizationStatus>, String>
        => crate::app::organization::desktop_organization_choose;

    // Delete the pending and active packs; the compiled brand applies on the next start.
    async desktop_organization_remove
        [app: tauri::AppHandle]
        () -> Result<crate::app::organization::OrganizationStatus, String>
        => crate::app::organization::desktop_organization_remove;

    // Drain all runtime resources before requesting Tauri's non-cancellable restart.
    async desktop_restart
        [app: tauri::AppHandle]
        () -> Result<(), String>
        => crate::app::desktop_restart;

    async desktop_parameters_read
        [state: tauri::State<'_, crate::app::settings::DesktopSettingsState>]
        () -> Result<Option<crate::parameters::Values>, crate::app::settings::DesktopSettingsError>
        => crate::app::settings::desktop_parameters_read;

    async desktop_parameters_write
        [state: tauri::State<'_, crate::app::settings::DesktopSettingsState>]
        (values: crate::parameters::Values) -> Result<(), crate::app::settings::DesktopSettingsError>
        => crate::app::settings::desktop_parameters_write;

    async desktop_updates_status
        [state: tauri::State<'_, crate::app::updates::UpdateState>]
        () -> Result<crate::app::updates::UpdateStatus, String>
        => crate::app::updates::desktop_updates_status;

    async desktop_updates_configure
        [app: tauri::AppHandle, state: tauri::State<'_, crate::app::updates::UpdateState>]
        (selection: crate::app::updates::Selection) -> Result<crate::app::updates::UpdateStatus, String>
        => crate::app::updates::desktop_updates_configure;

    async desktop_updates_check
        [app: tauri::AppHandle, state: tauri::State<'_, crate::app::updates::UpdateState>]
        () -> Result<crate::app::updates::UpdateCheck, String>
        => crate::app::updates::desktop_updates_check;

    async desktop_updates_install
        [state: tauri::State<'_, crate::app::updates::UpdateState>]
        (candidate: String) -> Result<crate::app::updates::InstallResult, String>
        => crate::app::updates::desktop_updates_install;

    async desktop_plugin_release_check
        [window: tauri::WebviewWindow, connection: tauri::State<'_, crate::connection::ConnectionSession>, state: tauri::State<'_, crate::app::updates::UpdateState>]
        (generation: String) -> Result<Vec<crate::app::updates::plugins::Release>, String>
        => crate::app::updates::plugins::desktop_plugin_release_check;

    async desktop_plugin_release_install
        [window: tauri::WebviewWindow, connection: tauri::State<'_, crate::connection::ConnectionSession>, state: tauri::State<'_, crate::app::updates::UpdateState>]
        (generation: String, candidate: String, progress: tauri::ipc::Channel<crate::app::plugins::InstallProgress>) -> Result<String, String>
        => crate::app::updates::plugins::desktop_plugin_release_install;
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
