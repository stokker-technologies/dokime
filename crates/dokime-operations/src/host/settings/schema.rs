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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::SettingsFrame;
use dokime_protocol::host::SettingsSchemaRequest;
use dokime_protocol::host::SettingsSchemaResponse;
use dokime_protocol::host::SettingsSection;
use dokime_protocol::host::SettingsSectionOwner;
use dokime_protocol::host::settings_section_owner::Kind;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Settings/Schema";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SettingsSchemaRequest, SettingsSchemaResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SettingsSchemaRequest) -> Result<SettingsSchemaResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (schema, cut) = crate::host::views::read(context, ID, request.at.as_ref(), async {
            let mut sections = vec![SettingsSection {
                section_id: "application".into(),
                label: "Application".into(),
                owner: Some(SettingsSectionOwner { kind: Some(Kind::App(true)) }),
                ..Default::default()
            }];
            for plugin in context.host.plugins().await? {
                let label = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.name.clone());
                sections.push(SettingsSection {
                    section_id: plugin.name.clone(),
                    label,
                    owner: Some(SettingsSectionOwner {
                        kind: Some(Kind::InstalledPluginId(plugin.name)),
                    }),
                    ..Default::default()
                });
            }
            Ok(SettingsFrame {
                bootstrap_paths: Some(context.host.bootstrap_paths()),
                sections,
                ..Default::default()
            })
        })
        .await?;
        Ok(SettingsSchemaResponse {
            schema: Some(schema),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SettingsSchemaRequest, SettingsSchemaResponse, context, "dokime.host.v1", "Settings", "Schema");
