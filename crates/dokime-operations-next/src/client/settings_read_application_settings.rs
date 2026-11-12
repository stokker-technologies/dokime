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

pub const ID: &str = "dokime.settings.read_application_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsReadApplicationSettingsRequest, v1::SettingsSnapshot>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsReadApplicationSettingsRequest) -> Result<v1::SettingsSnapshot> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        super::settings_preferences::read(context)
    })
    .await
}

pub fn initial_preferences() -> v1::SettingsSnapshot { super::settings_preferences::initial() }

/* === Private Functions === */

/* === Tests === */
