/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{env, path::PathBuf, sync::Arc};

use serde::Serialize;
use tauri::{Manager, State};

use dokime::paths::ApplicationPaths;

pub struct DesktopSettingsState {
    parameters: Result<PathBuf, String>,
    serial: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DesktopSettingsError {
    Unavailable { message: String },
    Persistence { message: String },
}

impl From<String> for DesktopSettingsError {
    fn from(message: String) -> Self {
        Self::Unavailable { message }
    }
}

pub fn setup(app: &tauri::App) {
    let parameters = application_root(&app.config().identifier).map(|root| root.join("parameters.json")).map_err(|error| error.to_string());
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
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

pub async fn desktop_parameters_write(
    state: State<'_, DesktopSettingsState>, values: crate::parameters::Values,
) -> Result<(), DesktopSettingsError> {
    let serial = state.serial.clone().lock_owned().await;
    let path = state.parameters.clone()?;
    tokio::task::spawn_blocking(move || {
        let _serial = serial;
        crate::parameters::save(&path, &values)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}
