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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn configuration(_app: &AppHandle) -> Result<Backend, String> {
    Ok(Backend::new(backend::locate()?, arguments()))
}

pub(crate) fn arguments() -> Vec<String> {
    let mut arguments = [
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
    .to_vec();
    arguments.extend([
        "--allowed-origin".to_owned(),
        if cfg!(target_os = "windows") {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .to_owned(),
    ]);
    #[cfg(debug_assertions)]
    arguments.extend([
        "--allowed-origin".to_owned(),
        "http://127.0.0.1:5173".to_owned(),
    ]);
    arguments
}

/* === Private Functions === */

/* === Tests === */
