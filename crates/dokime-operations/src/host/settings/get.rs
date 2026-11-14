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

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SettingsGetRequest, dokime_protocol::host::SettingsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SettingsGetRequest,
) -> Result<dokime_protocol::host::SettingsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, ID, request.at.as_ref(), async { crate::host::settings::preferences::read(context) }).await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

pub fn initial_preferences() -> dokime_protocol::host::SettingsGetResponse {
    crate::host::settings::preferences::initial()
}

/* === Tests === */
