/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tauri::AppHandle;

use crate::native_session::backend::Backend;

pub fn configuration(app: &AppHandle) -> Result<Backend, String> {
    let root = super::settings::application_root(&app.config().identifier).map_err(|error| error.to_string())?;
    let args = dokime_server::host::Args {
        shutdown_stdin: false,
        dev_control: false,
        network: dokime_server::host::NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: None,
            grpc_web: false,
            allowed_origins: Vec::new(),
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
