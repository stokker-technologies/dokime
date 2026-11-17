/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Progress Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::model::ExecutionProgressValue;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Progress/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ProgressGetRequest) -> Result<ProgressGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let name = context.resolve_execution(request.name.as_deref(), ID)?;
        let saved = context.host.store().resource_value(name, "progress", request.at.as_ref())?;
        Ok(ProgressGetResponse {
            value: Some(ExecutionProgressValue::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            resolved_at: Some(saved.resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ProgressGetRequest, ProgressGetResponse, context, "dokime.host.v1.executions", "Progress", "Get");
