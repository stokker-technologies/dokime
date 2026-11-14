/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SettingsSetRequest, dokime_protocol::host::SettingsSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SettingsSetRequest,
) -> Result<dokime_protocol::host::SettingsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Settings: Supply the Whole Writable Value".into()))?;
        crate::host::settings::preferences::validate(value)?;
        let fingerprint = crate::host::views::hex(&crate::host::paging::bytes(&request));
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            "preferences",
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &value.encode_to_vec(),
        )?;
        context.host.signal_change();
        Ok(dokime_protocol::host::SettingsSetResponse {
            value: Some(dokime_protocol::host::ApplicationSettings::decode(payload.as_slice())?),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
