/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings Schema

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Settings/Schema";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SettingsSchemaRequest, dokime_protocol::client::SettingsSchemaResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SettingsSchemaRequest,
) -> Result<dokime_protocol::client::SettingsSchemaResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (schema, cut) = super::views::read(context, ID, request.at.as_ref(), async {
            let mut sections = vec![dokime_protocol::client::SettingsSection {
                section_id: "application".into(),
                label: "Application".into(),
                owner: Some(dokime_protocol::client::SettingsSectionOwner {
                    kind: Some(dokime_protocol::client::settings_section_owner::Kind::App(true)),
                }),
                ..Default::default()
            }];
            for plugin in context.host.plugins()? {
                let label = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.entry.clone());
                sections.push(dokime_protocol::client::SettingsSection {
                    section_id: plugin.entry.clone(),
                    label,
                    owner: Some(dokime_protocol::client::SettingsSectionOwner {
                        kind: Some(dokime_protocol::client::settings_section_owner::Kind::InstalledPluginId(plugin.entry)),
                    }),
                    ..Default::default()
                });
            }
            Ok(dokime_protocol::client::SettingsFrame {
                bootstrap_paths: Some(context.host.bootstrap_paths()),
                sections,
                ..Default::default()
            })
        })
        .await?;
        Ok(dokime_protocol::client::SettingsSchemaResponse {
            schema: Some(schema),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
