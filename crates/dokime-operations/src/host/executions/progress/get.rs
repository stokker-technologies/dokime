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

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<harmos_protocol::model::OperationContract> {
    crate::host::declaration::<ProgressGetRequest, ProgressGetResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: ProgressGetRequest) -> Result<ProgressGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let name = crate::resources::resolve_execution(None, request.name.as_deref(), ID)?;
        let saved = context.host.store().resource_value(name, "progress", request.at.as_ref().and_then(|at| at.positions.first()))?;
        Ok(ProgressGetResponse {
            value: Some(ExecutionProgressValue::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            resolved_at: Some(harmos_protocol::model::ReadCut {
                positions: vec![saved.resolved_at],
                revisions: Vec::new(),
            }),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
