/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugins Settings Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.plugins.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::plugins::SettingsGetRequest, dokime_protocol::client::plugins::SettingsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::plugins::SettingsGetRequest,
) -> Result<dokime_protocol::client::plugins::SettingsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let key = super::settings_preferences::plugin_key(context, &request.name)?;
        let (mut reply, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            let settings = super::settings_preferences::plugin(context, &request.name)?;
            let (_, revision) = context.host.store().administration_value(&key, &[])?;
            Ok(dokime_protocol::client::plugins::SettingsGetResponse {
                value: settings.value,
                revision,
                schema: settings.schema,
                secrets: settings.secrets,
                discovered: settings.discovered,
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
