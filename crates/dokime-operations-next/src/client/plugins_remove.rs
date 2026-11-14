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
    super::declaration::<dokime_protocol::client::RemovePlugin, dokime_protocol::client::RemovePluginResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::RemovePlugin,
) -> Result<dokime_protocol::client::RemovePluginResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        context.host.remove_plugin_entry(&request.entry).await?;
        Ok(dokime_protocol::client::RemovePluginResponse {})
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
