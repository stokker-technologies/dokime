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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.StreamSchemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::StreamSchemasGetRequest, dokime_protocol::host::StreamSchemasGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::StreamSchemasGetRequest,
) -> Result<dokime_protocol::host::StreamSchemasGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        if request.schema_version != 0 && request.schema_version != 1 {
            return Err(Error::Invalid("Get Stream Schema: Select Schema Version 1".into()));
        }
        let reply = crate::host::stream_schemas::list::handle(
            context,
            dokime_protocol::host::StreamSchemasListRequest { parent: request.parent, at: request.at },
        )
        .await?;
        let value = reply
            .values
            .into_iter()
            .find(|value| value.stream == request.name)
            .ok_or_else(|| Error::NotFound("Get StreamSchemas: Select a Declared Identifier".into()))?;
        Ok(dokime_protocol::host::StreamSchemasGetResponse {
            value: Some(value),
            resolved_at: reply.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
