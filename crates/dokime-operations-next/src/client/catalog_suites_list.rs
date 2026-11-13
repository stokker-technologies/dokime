/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.catalog.Suites/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::catalog::SuitesListRequest, v1::catalog::SuitesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::catalog::SuitesListRequest) -> Result<v1::catalog::SuitesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let at = request.at.clone();
let key = format!("catalog/suites/{}", request.parent);
let (saved, cut) = super::views::read(context,&key,at.as_ref(),async {
 let inventory = super::catalog_inventory::Inventory::read(context).await?;
 let values = inventory.suites.into_iter().map(|(_,value)|value).collect::<Vec<_>>();
 Ok(v1::catalog::SuitesListResponse { values, ..Default::default() })
}).await?;
Ok(v1::catalog::SuitesListResponse { resolved_at:Some(cut),..saved })
    }).await
}

/* === Private Functions === */

/* === Tests === */
