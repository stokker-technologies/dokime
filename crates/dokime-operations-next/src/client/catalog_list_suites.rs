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
pub async fn handle(context: &Context, request: v1::CatalogListSuitesRequest) -> Result<v1::ListSuitesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context).await?;
        inventory.check(request.cursor.as_ref())?;
        if !request.after_suite_id.is_empty()
            && !inventory
                .suites
                .iter()
                .any(|(_, suite)| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.after_suite_id))
        {
            return Err(Error::Invalid("Continue Suites: Select an Existing Suite Identifier".into()));
        }
        let limit = super::page_size(request.limit, context.settings.page_items)?;
        let mut entries = inventory.suites.into_iter().filter_map(|(source, suite)| {
            let metadata = suite.metadata?;
            (metadata.identifier > request.after_suite_id).then(|| v1::SuiteIdentity {
                suite_id: metadata.identifier.clone(),
                source,
                metadata: Some(metadata),
            })
        });
        let suites: Vec<_> = entries.by_ref().take(limit).collect();
        let next_after_suite_id = entries.next().and_then(|_| suites.last().map(|item| item.suite_id.clone()));
        Ok(v1::ListSuitesReply {
            suites,
            next_after_suite_id,
            cursor: Some(inventory.cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
