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

pub const ID: &str = "dokime.catalog.list_suites";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogListSuitesRequest, v1::ListSuitesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, _request: v1::CatalogListSuitesRequest) -> Result<v1::ListSuitesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context).await?;
        let suites = inventory
            .suites
            .into_iter()
            .filter_map(|(source, suite)| {
                let metadata = suite.metadata?;
                Some(v1::SuiteIdentity {
                    suite_id: metadata.identifier.clone(),
                    source,
                    metadata: Some(metadata),
                })
            })
            .collect();
        Ok(v1::ListSuitesReply { suites })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
