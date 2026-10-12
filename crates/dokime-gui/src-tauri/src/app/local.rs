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
    [
        "serve",
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
