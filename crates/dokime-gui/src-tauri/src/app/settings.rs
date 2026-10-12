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
use std::{env, path::PathBuf, sync::Arc};
// Third Party Imports
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
// Local Imports
use dokime::paths::ApplicationPaths;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct DesktopSettingsState {
    parameters: Result<PathBuf, String>,
    serial: Arc<tokio::sync::Mutex<()>>,
}

/* === Enums === */
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DesktopSettingsError {
    Unavailable { message: String },
    Persistence { message: String },
}

/* === Traits === */

/* === Implementations === */
impl From<String> for DesktopSettingsError {
    fn from(message: String) -> Self {
        Self::Unavailable { message }
    }
}

/* === Public Functions === */
pub fn setup(app: &tauri::App) {
    let parameters = application_root(&app.config().identifier)
        .map(|root| root.join("parameters.json"))
        .map_err(|error| error.to_string());
    app.manage(DesktopSettingsState {
        parameters,
        serial: Arc::new(tokio::sync::Mutex::new(())),
    });
}

pub fn application_root(identifier: &str) -> std::io::Result<PathBuf> {
    match env::var_os("DOKIME_APPLICATION_ROOT").filter(|value| !value.is_empty()) {
        Some(root) => Ok(PathBuf::from(root)),
        None => ApplicationPaths::resolve(identifier).map(|paths| paths.root),
    }
}

pub async fn desktop_parameters_read(
    state: State<'_, DesktopSettingsState>,
) -> Result<Option<crate::parameters::Values>, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.parameters.clone()?;
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::parameters::load(&path)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence {
        message: error.to_string(),
    })
}

pub async fn desktop_parameters_write(
    state: State<'_, DesktopSettingsState>,
    values: crate::parameters::Values,
) -> Result<(), DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.parameters.clone()?;
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::parameters::save(&path, &values)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence {
        message: error.to_string(),
    })
}

pub async fn desktop_pick_execution_directory(
    app: AppHandle,
) -> Result<Option<String>, DesktopSettingsError> {
    if remote_backend(&app) {
        return Err("Execution storage is managed by the remote backend"
            .to_owned()
            .into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_folder()
            .map(|path| {
                path.into_path()
                    .map(|path| path.display().to_string())
                    .map_err(|error| error.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(DesktopSettingsError::from)
}

/* === Private Functions === */
fn remote_backend(app: &AppHandle) -> bool {
    app.try_state::<crate::native_session::DesktopSessionControl>()
        .is_some_and(|state| {
            state.status().ownership == crate::native_session::BackendOwnership::Attached
        })
}

/* === Tests === */
