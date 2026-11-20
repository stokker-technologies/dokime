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
use tauri::{AppHandle, Manager};

// Local Imports
use super::logging;
use crate::native_session::backend::Backend;
use crate::native_session::storage::StorageRoot;
use crate::paths::ApplicationPaths;
use crate::settings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn configuration(app: &AppHandle) -> Result<Backend, String> {
    let paths = app.state::<ApplicationPaths>().inner().clone();
    let root = StorageRoot::resolve(paths.root.clone(), 1_048_576)?.path().to_path_buf();
    Ok(Backend::new(root).with_log(logging::active_log()).with_settings(settings::path(&paths.root)))
}

/* === Private Functions === */

/* === Tests === */
