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

/* === Constants === */

pub const ID: &str = "dokime.health.list_health";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HealthListHealthRequest, v1::ListHealthResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HealthListHealthRequest) -> Result<v1::ListHealthResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        use prost::Message;
        use sha2::{Digest, Sha256};
        if let Some(owner) = request.selection.as_ref().and_then(|selection| selection.owner.as_ref())
            && owner.owner.as_ref().and_then(|owner| owner.kind.as_ref()).is_none()
        {
            return Err(Error::Invalid("List Health: Supply the Selected Annotation Owner".into()));
        }
        let include_global = request.selection.as_ref().is_none_or(|selection| selection.owner.is_none() || selection.include_context);
        let mut items = Vec::new();
        if include_global {
            let disabled: Vec<_> = context
                .host
                .installations()?
                .into_iter()
                .filter(|installation| installation.state == v1::PluginInstallationState::Disabled as i32)
                .map(|installation| installation.install_id)
                .collect();
            for plugin in context.host.plugins()? {
                if disabled.contains(&plugin.entry) {
                    continue;
                }
                if plugin.state == v1::PluginState::Ready as i32 {
                    continue;
                }
                items.push(v1::HealthItem {
                    health_item_id: plugin.entry,
                    message: if plugin.reason.is_empty() {
                        "Start or Reload the Plugin to Restore Its Services".into()
                    } else {
                        plugin.reason
                    },
                    severity: v1::Severity::Error as i32,
                    ..Default::default()
                });
            }
        }
        items.sort_by(|left, right| left.health_item_id.cmp(&right.health_item_id));
        let digest: [u8; 32] = Sha256::digest(
            v1::ListHealthResponse {
                items: items.clone(),
                ..Default::default()
            }
            .encode_to_vec(),
        )
        .into();
        let cursor = v1::Cursor {
            epoch: context.host.identity().epoch,
            stream: "health".into(),
            position: u64::from_le_bytes(digest[..8].try_into().unwrap()),
        };
        if request.cursor.as_ref().is_some_and(|given| given != &cursor) {
            return Err(Error::Stale("Health Changed: Restart Listing with a Fresh Cursor".into()));
        }
        if !request.after_id.is_empty() && !items.iter().any(|item| item.health_item_id == request.after_id) {
            return Err(Error::Invalid("Continue Health: Select an Existing Health Item".into()));
        }
        let limit = super::page_size(request.limit, context.settings.page_items)?;
        let mut selected = items.into_iter().filter(|item| item.health_item_id > request.after_id);
        let items: Vec<_> = selected.by_ref().take(limit).collect();
        let next_after_id = selected.next().and_then(|_| items.last().map(|item| item.health_item_id.clone())).unwrap_or_default();
        Ok(v1::ListHealthResponse {
            items,
            cursor: Some(cursor),
            next_after_id,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
