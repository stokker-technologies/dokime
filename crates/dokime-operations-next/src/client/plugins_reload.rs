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

pub const ID: &str = "dokime.plugins.reload";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::PluginsReloadRequest, v1::ClientEmpty>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: dokime_protocol::client::PluginsReloadRequest) -> Result<v1::ClientEmpty> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.install_id.is_some() == request.plugin_id.is_some() {
            return Err(Error::Invalid("Reload Plugin: Supply Exactly One Installation or Plugin Entry Identifier".into()));
        }
        let identifier = request.plugin_id.or(request.install_id).unwrap();
        let _guard = context.administration.lock().await;
        context.host.reload_plugin_entry(&identifier).await?;
        Ok(v1::ClientEmpty {})
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
