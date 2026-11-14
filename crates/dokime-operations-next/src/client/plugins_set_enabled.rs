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

pub const ID: &str = "dokime.plugins.set_enabled";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::PluginsSetEnabledRequest, dokime_protocol::client::PluginInstallation>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PluginsSetEnabledRequest,
) -> Result<dokime_protocol::client::PluginInstallation> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let host = context.host.clone();
        let administration = context.administration.clone();
        tokio::spawn(async move {
            let _guard = administration.lock().await;
            host.set_plugin_enabled(&request.install_id, request.enabled).await
        })
        .await
        .map_err(|_| Error::Invalid("Enable Plugin: Refresh Its Installation State Before Retrying".into()))?
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
