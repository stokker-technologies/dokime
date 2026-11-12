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
use crate::{Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.settings.get_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsGetSettingsRequest, v1::SettingsFrame>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsGetSettingsRequest) -> Result<v1::SettingsFrame> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        let mut sections = vec![v1::SettingsSection { section_id: "application".into(), label: "Application".into(), owner: Some(v1::SettingsSectionOwner { kind: Some(v1::settings_section_owner::Kind::App(true)) }), ..Default::default() }];
        for plugin in context.host.plugins()? {
            let label = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.entry.clone());
            sections.push(v1::SettingsSection { section_id: plugin.entry.clone(), label, owner: Some(v1::SettingsSectionOwner { kind: Some(v1::settings_section_owner::Kind::InstalledPluginId(plugin.entry)) }), ..Default::default() });
        }
        Ok(v1::SettingsFrame { bootstrap_paths: Some(context.host.bootstrap_paths()), sections, ..Default::default() })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
