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
    let root = super::settings::application_root(&app.config().identifier).map_err(|error| error.to_string())?;
    let args = dokime_server::host::Args {
        shutdown_stdin: false,
        dev_control: false,
        network: dokime_server::host::NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: None,
        },
        title: None,
        label: None,
        output_root: None,
        application_root: Some(root),
        env_file: None,
        plugins: Vec::new(),
        plugins_root: std::env::var_os("DOKIME_PLUGINS_DIR").map(std::path::PathBuf::from),
        no_seed_validation_artifact: false,
    };
    Ok(Backend::new(args).with_log(super::logging::active_log()))
}

/* === Private Functions === */

/* === Tests === */
