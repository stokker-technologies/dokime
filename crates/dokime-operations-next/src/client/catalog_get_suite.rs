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

pub const ID: &str = "dokime.catalog.get_suite";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogGetSuiteRequest, v1::Suite>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogGetSuiteRequest) -> Result<v1::Suite> {
    super::complete(ID, context.settings.read_timeout, async {
        context.host.refresh_plugin_states().await?;
        context
            .host
            .suites()?
            .into_iter()
            .find(|(_, suite)| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.suite_id))
            .map(|(_, suite)| suite)
            .ok_or_else(|| Error::NotFound(format!("GetSuite: Admit Suite {} First", request.suite_id)))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
