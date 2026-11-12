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

pub const ID: &str = "dokime.settings.update_application_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsUpdateApplicationSettingsRequest, v1::SettingsSnapshot>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsUpdateApplicationSettingsRequest) -> Result<v1::SettingsSnapshot> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let update = request.update.ok_or_else(|| Error::Invalid("Save Settings: Supply a Revision and Settings".into()))?;
        let settings = update.settings.as_ref().ok_or_else(|| Error::Invalid("Save Settings: Supply the Complete Settings".into()))?;
        super::settings_preferences::validate(settings)?;
        let previous = super::settings_preferences::read(context)?;
        let listener_changed = previous.settings.as_ref().and_then(|settings| settings.listener.as_ref()) != settings.listener.as_ref();
        let saved = context.host.store().update_preferences(update, &super::settings_preferences::initial())?;
        if listener_changed && let Some(listener) = saved.settings.as_ref().and_then(|settings| settings.listener.clone()) {
            context.host.update_remote_listener(listener).await?;
        }
        Ok(saved)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
