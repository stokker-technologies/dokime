/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Settings/Schema";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SettingsSchemaRequest, v1::SettingsSchemaResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsSchemaRequest) -> Result<v1::SettingsSchemaResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (schema,cut)=super::views::read(context,ID,request.at.as_ref(),async { 
        
        let mut sections = vec![v1::SettingsSection {
            section_id: "application".into(),
            label: "Application".into(),
            owner: Some(v1::SettingsSectionOwner {
                kind: Some(v1::settings_section_owner::Kind::App(true)),
            }),
            ..Default::default()
        }];
        for plugin in context.host.plugins()? {
            let label = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.entry.clone());
            sections.push(v1::SettingsSection {
                section_id: plugin.entry.clone(),
                label,
                owner: Some(v1::SettingsSectionOwner {
                    kind: Some(v1::settings_section_owner::Kind::InstalledPluginId(plugin.entry)),
                }),
                ..Default::default()
            });
        }
        Ok(v1::SettingsFrame {
            bootstrap_paths: Some(context.host.bootstrap_paths()),
            sections,
            ..Default::default()
        }) }).await?;
Ok(v1::SettingsSchemaResponse {schema:Some(schema),resolved_at:Some(cut)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
