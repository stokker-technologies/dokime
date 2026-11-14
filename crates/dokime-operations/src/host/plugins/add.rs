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

pub const ID: &str = "dokime.plugins.add";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::AddRemotePlugin, dokime_protocol::client::Plugin>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: dokime_protocol::client::AddRemotePlugin) -> Result<dokime_protocol::client::Plugin> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.url.is_empty() {
            return Err(Error::Invalid("Add a Plugin URL".into()));
        }
        let _guard = context.administration.lock().await;
        context.host.add_remote(&request.url).await
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
