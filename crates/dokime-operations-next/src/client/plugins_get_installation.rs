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

pub const ID: &str = "dokime.plugins.get_installation";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<dokime_protocol::client::PluginsGetInstallationRequest, dokime_protocol::client::PluginInstallation>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PluginsGetInstallationRequest,
) -> Result<dokime_protocol::client::PluginInstallation> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        context
            .host
            .installations()?
            .into_iter()
            .find(|installation| installation.install_id == request.install_id)
            .ok_or_else(|| Error::NotFound("Inspect Plugin: Refresh the Installation List and Select an Existing Installation".into()))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
