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

pub const ID: &str = "dokime.plugins.remove";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RemovePlugin, v1::RemovePluginResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::RemovePlugin) -> Result<v1::RemovePluginResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        context.host.remove_plugin_entry(&request.entry).await?;
        Ok(v1::RemovePluginResponse {})
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
