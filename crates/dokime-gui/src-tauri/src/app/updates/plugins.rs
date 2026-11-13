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

// Third Party Imports
use serde::Serialize;
use tauri::{State, WebviewWindow, ipc::Channel};

// Local Imports
use crate::app::plugins::{self, InstallProgress};
use crate::connection::ConnectionSession;

/* === Constants === */

const HOST_BUILD_REQUIRED: &str = "Check Plugin Releases: The Host Does Not Yet Publish Its Target; Install a Verified Bundle File Instead";

/* === Structs === */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub candidate: String,
    pub package: String,
    pub version: String,
    pub target: String,
    pub requires_dokime: String,
}

/* === Public Functions === */

pub async fn desktop_plugin_release_check(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, generation: String,
) -> Result<Vec<Release>, String> {
    let _lease = plugins::admit(&window, &connection, &generation)?;
    Err(HOST_BUILD_REQUIRED.into())
}

pub async fn desktop_plugin_release_install(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, generation: String, _candidate: String,
    _progress: Channel<InstallProgress>,
) -> Result<String, String> {
    let _lease = plugins::admit(&window, &connection, &generation)?;
    Err(HOST_BUILD_REQUIRED.into())
}
