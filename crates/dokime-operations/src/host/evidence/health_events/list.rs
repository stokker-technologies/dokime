/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Health Events List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.HealthEvents/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<
        dokime_protocol::host::evidence::HealthEventsListRequest,
        dokime_protocol::host::evidence::HealthEventsListResponse,
    >(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::evidence::HealthEventsListRequest,
) -> Result<dokime_protocol::host::evidence::HealthEventsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!(
            "health/{}",
            crate::host::views::hex(&request.selection.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default())
        );
        let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            context.host.refresh_plugin_states().await?;
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
                    .filter(|installation| installation.state == harmos_protocol::host::PluginState::Disabled as i32)
                    .map(|installation| installation.install_id)
                    .collect();
                for plugin in context.host.plugins()? {
                    if disabled.contains(&plugin.entry) {
                        continue;
                    }
                    if plugin.state == harmos_protocol::host::PluginState::Ready as i32 {
                        continue;
                    }
                    items.push(dokime_protocol::host::evidence::HealthItem {
                        health_item_id: plugin.entry,
                        message: if plugin.reason.is_empty() {
                            "Start or Reload the Plugin to Restore Its Services".into()
                        } else {
                            plugin.reason
                        },
                        severity: dokime_protocol::model::Severity::Error as i32,
                        ..Default::default()
                    });
                }
            }
            items.sort_by(|left, right| left.health_item_id.cmp(&right.health_item_id));
            Ok(dokime_protocol::host::evidence::HealthEventsListResponse { events: items, ..Default::default() })
        })
        .await?;
        let (items, next_page_token) = crate::host::views::page(
            saved.events,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        Ok(dokime_protocol::host::evidence::HealthEventsListResponse {
            events: items,
            resolved_at: Some(cut),
            next_page_token,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
