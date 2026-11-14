/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Suites Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Suites/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SuitesGetRequest, dokime_protocol::host::SuitesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SuitesGetRequest,
) -> Result<dokime_protocol::host::SuitesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let reply =
            crate::host::suites::list::handle(context, dokime_protocol::host::SuitesListRequest { parent: request.parent, at: request.at })
                .await?;
        let value = reply
            .values
            .into_iter()
            .find(|value| value.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.name))
            .ok_or_else(|| Error::NotFound("Get Suites: Select a Declared Identifier".into()))?;
        Ok(dokime_protocol::host::SuitesGetResponse {
            value: Some(value),
            resolved_at: reply.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
