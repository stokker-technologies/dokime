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
use std::{path::PathBuf, sync::Arc};

// Third Party Imports
use serde::Serialize;
use tauri::{Manager, State};

// Local Imports
use crate::paths::ApplicationPaths;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct DesktopSettingsState {
    parameters: PathBuf,
    capture_folders: PathBuf,
    interval_settings: PathBuf,
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
    let root = &app.state::<ApplicationPaths>().root;
    app.manage(DesktopSettingsState {
        parameters: root.join("parameters.json"),
        capture_folders: crate::capture_folders::path(root),
        interval_settings: crate::interval_settings::path(root),
        serial: Arc::new(tokio::sync::Mutex::new(())),
    });
}

pub async fn desktop_parameters_read(
    state: State<'_, DesktopSettingsState>,
) -> Result<Option<crate::parameters::Values>, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.parameters.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::parameters::load(&path)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

pub async fn desktop_parameters_write(
    state: State<'_, DesktopSettingsState>, values: crate::parameters::Values,
) -> Result<(), DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.parameters.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::parameters::save(&path, &values)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

pub async fn desktop_capture_folders_read(state: State<'_, DesktopSettingsState>) -> Result<Vec<String>, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.capture_folders.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::capture_folders::load(&path)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

/// Saves the folders, then applies them to the owned local host for captures opened from now on.
pub async fn desktop_capture_folders_write(
    state: State<'_, DesktopSettingsState>, owner: State<'_, crate::native_session::DesktopSessionControl>, folders: Vec<String>,
) -> Result<Vec<String>, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.capture_folders.clone();
    let saved = folders.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::capture_folders::save(&path, &saved)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })?;
    owner.set_capture_folders(folders.iter().map(PathBuf::from).collect()).await;
    Ok(folders)
}

pub async fn desktop_interval_settings_read(
    state: State<'_, DesktopSettingsState>,
) -> Result<crate::interval_settings::IntervalSettings, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.interval_settings.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::interval_settings::load(&path)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

/// Saves the interval settings the owned local host starts with; the running host keeps its own until it restarts.
pub async fn desktop_interval_settings_write(
    state: State<'_, DesktopSettingsState>, settings: crate::interval_settings::IntervalSettings,
) -> Result<crate::interval_settings::IntervalSettings, DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.interval_settings.clone();
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::interval_settings::save(&path, &settings)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })?;
    Ok(settings)
}

/* === Private Functions === */

/* === Tests === */
