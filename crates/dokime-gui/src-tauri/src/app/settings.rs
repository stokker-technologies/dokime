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
use std::{num::NonZeroU32, sync::Arc};

// Third Party Imports
use serde::{Deserialize, Serialize};
use tauri::{App, Manager, State};
use tokio::task;

// Local Imports
use crate::paths::ApplicationPaths;
use crate::settings::{self, DesktopSettings, Store, Values};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One change: only the keys present replace the saved ones.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SettingsChange {
    pub interval_window_ms: Option<u64>,
    pub interval_lag_windows: Option<NonZeroU32>,
    pub parameters: Option<Values>,
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

pub fn setup(app: &App) {
    let root = &app.state::<ApplicationPaths>().root;
    app.manage(Arc::new(Store::new(settings::path(root))));
}

pub async fn desktop_saved_settings_read(store: State<'_, Arc<Store>>) -> Result<DesktopSettings, DesktopSettingsError> {
    let path = store.path().to_path_buf();
    task::spawn_blocking(move || settings::load(&path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

/// Saves the keys present; the owned local host applies stream settings when it next starts.
pub async fn desktop_saved_settings_update(
    store: State<'_, Arc<Store>>, change: SettingsChange,
) -> Result<DesktopSettings, DesktopSettingsError> {
    let store = store.inner().clone();
    task::spawn_blocking(move || {
        store.update(|settings| {
            if let Some(window) = change.interval_window_ms {
                settings.interval_window_ms = window;
            }
            if let Some(lag) = change.interval_lag_windows {
                settings.interval_lag_windows = lag;
            }
            if let Some(parameters) = change.parameters {
                settings.parameters = Some(parameters);
            }
            Ok(settings.clone())
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| DesktopSettingsError::Persistence { message: error.to_string() })
}

/* === Private Functions === */

/* === Tests === */
