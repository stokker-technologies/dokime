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
use crate::native_session::backend::Backend;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn configuration(app: &AppHandle) -> Result<Backend, String> {
    let paths = app.state::<crate::paths::ApplicationPaths>().inner().clone();
    let root = crate::native_session::storage::StorageRoot::resolve(paths.root, 1_048_576)?.path().to_path_buf();
    Ok(Backend::new(root).with_log(super::logging::active_log()))
}

/* === Private Functions === */

/* === Tests === */
