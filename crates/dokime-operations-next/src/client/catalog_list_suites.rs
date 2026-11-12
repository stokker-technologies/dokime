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
        if request.cursor.is_some() {
            return Err(Error::Invalid("ListSuites Cursor Is Not Yet Served by the New Host".into()));
        }
        let mut suites: Vec<_> = context
            .host
            .suites()?
            .into_iter()
            .filter_map(|(plugin, suite)| {
                let metadata = suite.metadata?;
                if metadata.identifier <= request.after_suite_id {
                    return None;
                }
                Some(v1::SuiteIdentity {
                    suite_id: metadata.identifier.clone(),
                    source: plugin,
                    metadata: Some(metadata),
                })
            })
            .collect();
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let next_after_suite_id = if suites.len() > limit {
            suites.truncate(limit);
            suites.last().map(|suite| suite.suite_id.clone())
        } else {
            None
        };
        Ok(v1::ListSuitesReply {
            suites,
            next_after_suite_id,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
