/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Backend/Appearance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::BackendAppearanceRequest, dokime_protocol::client::BackendAppearanceResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::BackendAppearanceRequest,
) -> Result<dokime_protocol::client::BackendAppearanceResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (appearance, cut) = super::views::read(context, ID, request.at.as_ref(), async {
            context.host.refresh_plugin_states().await?;

            let mut appearance = None;
            for plugin in
                context.host.plugins()?.into_iter().filter(|plugin| plugin.state == dokime_protocol::client::PluginState::Ready as i32)
            {
                let Some(catalog) = plugin.description.and_then(|description| description.catalog) else {
                    continue;
                };
                for section in catalog.sections {
                    if section.r#type != "dokime.client.v1.AppearanceSnapshot" {
                        continue;
                    }
                    if appearance.is_some() {
                        return Err(Error::Invalid("Choose One Appearance Provider; Disable the Conflicting Plugin".into()));
                    }
                    let mut snapshot = dokime_protocol::client::AppearanceSnapshot::decode(section.payload.as_slice())?;
                    if snapshot.app_name.trim().is_empty() {
                        return Err(Error::Invalid("Read Appearance: Give the Provider an Application Name".into()));
                    }
                    snapshot.source_plugin_id = Some(plugin.entry.clone());
                    appearance = Some(snapshot);
                }
            }
            let reply = appearance.unwrap_or_else(|| dokime_protocol::client::AppearanceSnapshot {
                app_name: "Dokime".into(),
                mode_policy: dokime_protocol::client::ThemeModePolicy::UserOrSystem as i32,
                ..Default::default()
            });
            if reply.encoded_len() > context.settings.message_bytes {
                return Err(Error::Invalid("Read Appearance: Reduce Provider Assets to the Response Budget".into()));
            }
            Ok(reply)
        })
        .await?;
        Ok(dokime_protocol::client::BackendAppearanceResponse {
            appearance: Some(appearance),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
