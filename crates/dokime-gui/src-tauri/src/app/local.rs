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
    let paths = app.state::<dokime::paths::ApplicationPaths>().inner().clone();
    let root = paths.root.clone();
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
        plugins_root: std::env::var_os("DOKIME_PLUGINS_DIR").filter(|value| !value.is_empty()).map(std::path::PathBuf::from),
    };
    let journal = app.state::<std::sync::Arc<dokime_server::trace::Journal>>().inner().clone();
    Ok(Backend::new(args).with_application(paths).with_log(super::logging::active_log()).with_trace_journal(journal))
}

/* === Private Functions === */

/* === Tests === */
