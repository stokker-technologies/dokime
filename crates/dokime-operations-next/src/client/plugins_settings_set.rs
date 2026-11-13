/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugins Settings Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.plugins.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::plugins::SettingsSetRequest, v1::plugins::SettingsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::plugins::SettingsSetRequest) -> Result<v1::plugins::SettingsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let current = super::settings_preferences::plugin(context, &request.name)?;
        let values = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Plugin Settings: Supply the Whole Writable Value".into()))?;
        super::settings_preferences::validate_plugin(current.schema.as_ref().unwrap(), values)?;
        let fingerprint = super::views::hex(&super::paging::bytes(&request));
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            &super::settings_preferences::plugin_key(context, &request.name)?,
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &values.encode_to_vec(),
        )?;
        context.host.signal_change();
        Ok(v1::plugins::SettingsSetResponse {
            value: Some(v1::MapValue::decode(payload.as_slice())?),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
