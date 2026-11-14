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
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::AppearanceSnapshot;
use dokime_protocol::host::BackendAppearanceRequest;
use dokime_protocol::host::BackendAppearanceResponse;
use dokime_protocol::host::ThemeModePolicy;
use harmos_protocol::host::PluginState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Backend/Appearance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<BackendAppearanceRequest, BackendAppearanceResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: BackendAppearanceRequest) -> Result<BackendAppearanceResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (appearance, cut) = crate::host::views::read(context, ID, request.at.as_ref(), async {
            context.host.refresh_plugin_states().await?;

            let mut appearance = None;
            for plugin in context.host.plugins()?.into_iter().filter(|plugin| plugin.state == PluginState::Ready as i32) {
                let Some(catalog) = plugin.description.and_then(|description| description.declarations) else {
                    continue;
                };
                for section in catalog.sections {
                    if section.r#type != "dokime.host.v1.AppearanceSnapshot" {
                        continue;
                    }
                    if appearance.is_some() {
                        return Err(Error::Invalid("Choose One Appearance Provider; Disable the Conflicting Plugin".into()));
                    }
                    let mut snapshot = AppearanceSnapshot::decode(section.payload.as_slice())?;
                    if snapshot.app_name.trim().is_empty() {
                        return Err(Error::Invalid("Read Appearance: Give the Provider an Application Name".into()));
                    }
                    snapshot.source_plugin_id = Some(plugin.entry.clone());
                    for logo in &mut snapshot.logos {
                        let Some(asset) = logo.asset.as_mut() else {
                            continue;
                        };
                        let Some(reference) = asset.reference.as_ref() else {
                            continue;
                        };
                        // A Provider May Reference Only Its Published, Digest-Matched File.
                        if reference.plugin_id != plugin.entry {
                            return Err(Error::Invalid("Read Appearance: Keep Assets with Their Declaring Provider".into()));
                        }
                        match context.host.store().file_reference(&reference.asset_id) {
                            Ok(file) => {
                                if crate::host::views::hex(&file.sha256) != reference.sha256 || file.length != asset.size_bytes {
                                    return Err(Error::Invalid("Read Appearance: Publish the Declared Asset Length and SHA-256".into()));
                                }
                                asset.file = Some(file);
                            }
                            Err(dokime_storage::Error::NotFound(_)) => {
                                asset.file = None;
                            }
                            Err(error) => return Err(error.into()),
                        }
                    }
                    appearance = Some(snapshot);
                }
            }
            let reply = appearance.unwrap_or_else(|| AppearanceSnapshot {
                app_name: "Dokime".into(),
                mode_policy: ThemeModePolicy::UserOrSystem as i32,
                ..Default::default()
            });
            if reply.encoded_len() > context.settings.message_bytes {
                return Err(Error::Invalid("Read Appearance: Reduce Provider Assets to the Response Budget".into()));
            }
            Ok(reply)
        })
        .await?;
        Ok(BackendAppearanceResponse {
            appearance: Some(appearance),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
