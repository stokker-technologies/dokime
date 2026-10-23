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
use tauri::AppHandle;

// Local Imports
use crate::native_session::backend::{self, Backend};
use dokime::paths::ApplicationPaths;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn configuration(app: &AppHandle) -> Result<Backend, String> {
    let backend = Backend::new(backend::locate()?, arguments());
    // A desktop launch has no console for the backend's stderr; its log lives
    // with the rest of this application's data, so an unresolved root keeps
    // only the in-memory diagnostics tail.
    Ok(match super::settings::application_root(&app.config().identifier) {
        Ok(root) => backend.with_logs(ApplicationPaths::new(root).logs()),
        Err(_) => backend,
    })
}

pub(crate) fn arguments() -> Vec<String> {
    [
        // EOF on the control pipe is this desktop's stop signal, and the
        // development control protocol is what publishes the endpoint to
        // attach to instead of printing a terminal banner.
        "--shutdown-stdin",
        "--dev-control",
        "--host",
        "127.0.0.1",
        // An owned session takes any free port, so it never competes with a
        // backend started for plugin development on the default one.
        "--port",
        "0",
    ]
    .map(str::to_owned)
    .to_vec()
}

/* === Private Functions === */

/* === Tests === */
