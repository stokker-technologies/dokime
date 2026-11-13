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

/* === Constants === */

pub const ID: &str = "dokime.plugins.list_admitted_plugins";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::PluginsListAdmittedPluginsRequest, dokime_protocol::client::AdmittedPluginPage>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PluginsListAdmittedPluginsRequest,
) -> Result<dokime_protocol::client::AdmittedPluginPage> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        use prost::Message;
        use sha2::{Digest, Sha256};
        let plugins: Vec<_> = context
            .host
            .plugins()?
            .into_iter()
            .filter(|plugin| plugin.state == dokime_protocol::client::PluginState::Ready as i32)
            .collect();
        let identity = context.host.identity();
        let digest: [u8; 32] = Sha256::digest(dokime_protocol::client::PluginEntries { plugins: plugins.clone() }.encode_to_vec()).into();
        let revision = u64::from_le_bytes(digest[..8].try_into().unwrap());
        if request.after.as_ref().is_some_and(|after| after.epoch != identity.epoch || after.revision != revision) {
            return Err(Error::Stale("Plugin Inventory Changed: Restart Listing".into()));
        }
        let after = request.after.map(|after| after.after_plugin_id).unwrap_or_default();
        if !after.is_empty() && !plugins.iter().any(|plugin| plugin.entry == after) {
            return Err(Error::Invalid("Continue Plugins: Select an Existing After Identifier".into()));
        }
        let total_count = plugins.len() as u64;
        let limit = super::page_size(request.page_size, context.settings.page_items)?;
        let mut entries = plugins.into_iter().filter(|plugin| plugin.entry > after);
        let plugins: Vec<_> =
            entries.by_ref().take(limit).map(|plugin| dokime_protocol::client::AdmittedPlugin { plugin: Some(plugin) }).collect();
        let next = entries.next().and_then(|_| {
            plugins.last().map(|plugin| dokime_protocol::client::AdmittedPluginCursor {
                epoch: identity.epoch.clone(),
                revision,
                after_plugin_id: plugin.plugin.as_ref().unwrap().entry.clone(),
            })
        });
        Ok(dokime_protocol::client::AdmittedPluginPage {
            plugins,
            next,
            total_count,
            epoch: identity.epoch,
            revision,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
