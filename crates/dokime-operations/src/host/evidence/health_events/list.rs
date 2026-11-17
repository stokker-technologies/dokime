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
use harmos_protocol::host::PluginState;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::HealthEventsListRequest;
use dokime_protocol::host::evidence::HealthEventsListResponse;
use dokime_protocol::host::evidence::HealthItem;
use dokime_protocol::model::Severity;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.HealthEvents/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: HealthEventsListRequest) -> Result<HealthEventsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!(
            "health/{}",
            crate::host::views::hex(&request.selection.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default())
        );
        let owner = Scope::Root(&context.host.identity().backend_id).identifier()?;
        let (saved, cut) = crate::host::views::read(context, owner, &key, request.at.as_ref(), async {
            context.host.refresh_plugin_states().await?;
            if let Some(owner) = request.selection.as_ref().and_then(|selection| selection.owner.as_ref())
                && owner.owner.as_ref().and_then(|owner| owner.kind.as_ref()).is_none()
            {
                return Err(Error::Invalid("List Health: Supply the Selected Annotation Owner".into()));
            }
            let include_global = request.selection.as_ref().is_none_or(|selection| selection.owner.is_none() || selection.include_context);
            let mut items = Vec::new();
            if include_global {
                for plugin in context.host.plugins().await? {
                    if matches!(PluginState::try_from(plugin.state), Ok(PluginState::Stopped | PluginState::Ready)) {
                        continue;
                    }
                    items.push(HealthItem {
                        health_item_id: plugin.name,
                        message: plugin
                            .failure
                            .map_or_else(|| "Start or Reload the Plugin to Restore Its Services".into(), |failure| failure.message),
                        severity: Severity::Failure as i32,
                        ..Default::default()
                    });
                }
            }
            items.sort_by(|left, right| left.health_item_id.cmp(&right.health_item_id));
            Ok(HealthEventsListResponse { events: items, ..Default::default() })
        })
        .await?;
        let (items, next_page_token) = crate::host::views::page(
            saved.events,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        Ok(HealthEventsListResponse {
            events: items,
            resolved_at: Some(cut),
            next_page_token,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(HealthEventsListRequest, HealthEventsListResponse, context, "dokime.host.v1.evidence", "HealthEvents", "List");
