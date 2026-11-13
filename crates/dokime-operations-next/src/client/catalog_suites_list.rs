/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Suites List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Suites/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SuitesListRequest, dokime_protocol::client::SuitesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SuitesListRequest,
) -> Result<dokime_protocol::client::SuitesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/suites/{}", request.parent);
        let (saved, cut) = super::views::read(context, &key, at.as_ref(), async {
            let inventory = super::catalog_inventory::Inventory::read(context).await?;
            let values = inventory.suites.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
            Ok(dokime_protocol::client::SuitesListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(dokime_protocol::client::SuitesListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
