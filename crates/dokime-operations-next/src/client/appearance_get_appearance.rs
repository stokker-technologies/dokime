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
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.appearance.get_appearance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::GetAppearance, v1::AppearanceSnapshot>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::GetAppearance) -> Result<v1::AppearanceSnapshot> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        let _ = request;
        let mut appearance = None;
        for plugin in context.host.plugins()?.into_iter().filter(|plugin| plugin.state == v1::PluginState::Ready as i32) {
            let Some(catalog) = plugin.description.and_then(|description| description.catalog) else {
                continue;
            };
            for section in catalog.sections {
                if section.r#type != "dokime.v1.AppearanceSnapshot" {
                    continue;
                }
                if appearance.is_some() {
                    return Err(Error::Invalid("Choose One Appearance Provider; Disable the Conflicting Plugin".into()));
                }
                let mut snapshot = v1::AppearanceSnapshot::decode(section.payload.as_slice())?;
                if snapshot.app_name.trim().is_empty() {
                    return Err(Error::Invalid("Read Appearance: Give the Provider an Application Name".into()));
                }
                snapshot.source_plugin_id = Some(plugin.entry.clone());
                appearance = Some(snapshot);
            }
        }
        let reply = appearance.unwrap_or_else(|| v1::AppearanceSnapshot {
            app_name: "Dokime".into(),
            mode_policy: v1::ThemeModePolicy::UserOrSystem as i32,
            ..Default::default()
        });
        if reply.encoded_len() > context.settings.message_bytes {
            return Err(Error::Invalid("Read Appearance: Reduce Provider Assets to the Response Budget".into()));
        }
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
