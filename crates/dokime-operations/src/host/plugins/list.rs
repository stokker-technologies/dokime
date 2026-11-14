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

pub const ID: &str = "dokime.plugins.list";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::ListPlugins, dokime_protocol::client::PluginEntries>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: dokime_protocol::client::ListPlugins) -> Result<dokime_protocol::client::PluginEntries> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        let _ = request;
        Ok(dokime_protocol::client::PluginEntries { plugins: context.host.plugins()? })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
