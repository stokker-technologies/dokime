/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

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

pub const ID: &str = "dokime.v1.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SettingsSetRequest, v1::SettingsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsSetRequest) -> Result<v1::SettingsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let value=request.value.as_ref().ok_or_else(||Error::Invalid("Set Settings: Supply the Whole Writable Value".into()))?;
super::settings_preferences::validate(value)?;
let fingerprint=super::views::hex(&super::paging::bytes(&request));
let (payload,revision,receipt)=context.host.store().replace_administration_value("preferences",&request.expected_revision,&request.request_id,fingerprint.as_bytes(),&value.encode_to_vec())?;
context.host.signal_change();Ok(v1::SettingsSetResponse {value:Some(v1::ApplicationSettings::decode(payload.as_slice())?),revision,receipt:Some(receipt)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
