/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports


// Local Imports
use crate::{Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.settings.get_plugin_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsGetPluginSettingsRequest, v1::PluginRuntimeSettings>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsGetPluginSettingsRequest) -> Result<v1::PluginRuntimeSettings> {
    super::complete(ID, context.settings.read_timeout, async {
        super::settings_preferences::plugin(context, &request.plugin_id)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
