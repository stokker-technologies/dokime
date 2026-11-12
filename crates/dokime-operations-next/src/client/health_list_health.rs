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
        if request.selection.is_some() || request.cursor.is_some() || !request.after_id.is_empty() {
            return Err(Error::Invalid("ListHealth Selection and Cursors Are Not Yet Served by the New Host".into()));
        }
        let mut items = Vec::new();
        for plugin in context.host.plugins()? {
            if plugin.state == v1::PluginState::Ready as i32 {
                continue;
            }
            items.push(v1::HealthItem {
                health_item_id: plugin.entry,
                message: plugin.reason,
                severity: v1::Severity::Error as i32,
                ..Default::default()
            });
        }
        super::projection::bounded(items.len(), super::projection::limit(request.limit, context.settings.page_items))?;
        Ok(v1::ListHealthResponse { items, ..Default::default() })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
