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
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_storage::persisted::Scope;

use dokime_protocol::host::ApplicationSettings;
use dokime_protocol::host::SettingsSetRequest;
use dokime_protocol::host::SettingsSetResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SettingsSetRequest) -> Result<SettingsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Settings: Supply the Whole Writable Value".into()))?;
        crate::host::settings::preferences::validate(value)?;
        let (snapshot, receipt) = crate::host::storage::administration::replace(
            context,
            "preferences",
            Scope::Root(&context.host.identity().backend_id).identifier()?,
            request.expected_revision,
            &request.request_id,
            value.encode_to_vec(),
        )
        .await?;
        context.host.signal_change();
        Ok(SettingsSetResponse {
            value: Some(ApplicationSettings::decode(snapshot.payload.as_slice())?),
            revision: snapshot.revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SettingsSetRequest, SettingsSetResponse, context, "dokime.host.v1", "Settings", "Set");
