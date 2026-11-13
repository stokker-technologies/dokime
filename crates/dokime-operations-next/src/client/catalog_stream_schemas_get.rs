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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.catalog.StreamSchemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::catalog::StreamSchemasGetRequest, v1::catalog::StreamSchemasGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::catalog::StreamSchemasGetRequest) -> Result<v1::catalog::StreamSchemasGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let reply=super::catalog_stream_schemas_list::handle(context,v1::catalog::StreamSchemasListRequest { parent:request.parent,at:request.at }).await?;
let value=reply.values.into_iter().find(|value|value.stream==request.name).ok_or_else(||Error::NotFound("Get StreamSchemas: Select a Declared Identifier".into()))?;
Ok(v1::catalog::StreamSchemasGetResponse {value:Some(value),resolved_at:reply.resolved_at})
    }).await
}

/* === Private Functions === */

/* === Tests === */
