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
use crate::{Error, Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.settings.update_plugin_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsUpdatePluginSettingsRequest, v1::PluginRuntimeSettings>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsUpdatePluginSettingsRequest) -> Result<v1::PluginRuntimeSettings> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let current = super::settings_preferences::plugin(context, &request.plugin_id)?;
        let values = request.values.unwrap_or_default();
        super::settings_preferences::validate_plugin(current.settings.as_ref().unwrap(), &values)?;
        let plugin = context
            .host
            .plugins()?
            .into_iter()
            .find(|plugin| {
                plugin.entry == request.plugin_id
                    || plugin.description.as_ref().is_some_and(|description| description.id == request.plugin_id)
            })
            .ok_or_else(|| Error::Invalid("Save Plugin Settings: Select a Plugin Entry".into()))?;
        let settings_owner = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.entry.clone());
        context.host.store().save_administration(&format!("plugin-settings:{}", settings_owner), &values)?;
        super::settings_preferences::plugin(context, &plugin.entry)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
