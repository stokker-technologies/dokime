/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::SettingsGetRequest;
use dokime_protocol::host::SettingsGetResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SettingsGetRequest, SettingsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SettingsGetRequest) -> Result<SettingsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async { crate::host::settings::preferences::read(context, request.at).await })
        .await
}

/* === Private Functions === */

pub fn initial_preferences() -> SettingsGetResponse {
    crate::host::settings::preferences::initial()
}

/* === Tests === */
