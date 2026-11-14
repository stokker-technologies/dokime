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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SettingsGetRequest, dokime_protocol::client::SettingsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SettingsGetRequest,
) -> Result<dokime_protocol::client::SettingsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            super::views::read(context, ID, request.at.as_ref(), async { super::settings_preferences::read(context) }).await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

pub fn initial_preferences() -> dokime_protocol::client::SettingsGetResponse {
    super::settings_preferences::initial()
}

/* === Tests === */
