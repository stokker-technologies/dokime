/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Stream Schemas Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.StreamSchemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::StreamSchemasGetRequest, dokime_protocol::client::StreamSchemasGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::StreamSchemasGetRequest,
) -> Result<dokime_protocol::client::StreamSchemasGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        if request.schema_version != 0 && request.schema_version != 1 {
            return Err(Error::Invalid("Get Stream Schema: Select Schema Version 1".into()));
        }
        let reply = super::catalog_stream_schemas_list::handle(
            context,
            dokime_protocol::client::StreamSchemasListRequest { parent: request.parent, at: request.at },
        )
        .await?;
        let value = reply
            .values
            .into_iter()
            .find(|value| value.stream == request.name)
            .ok_or_else(|| Error::NotFound("Get StreamSchemas: Select a Declared Identifier".into()))?;
        Ok(dokime_protocol::client::StreamSchemasGetResponse {
            value: Some(value),
            resolved_at: reply.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
