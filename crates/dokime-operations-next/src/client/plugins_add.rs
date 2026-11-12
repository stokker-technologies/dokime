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

pub const ID: &str = "dokime.plugins.add";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::AddRemotePlugin, v1::Plugin>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::AddRemotePlugin) -> Result<v1::Plugin> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.url.is_empty() {
            return Err(Error::Invalid("Add a Plugin URL".into()));
        }
        context.host.add_remote(&request.url).await
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
