/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::LogsAppendResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Logs/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(resources: &Resources, request: LogsAppendRequest) -> Result<LogsAppendResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let source = request.at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    let (_, receipt) =
        resources.evidence_requested(ID, &request.request_id, &request.encode_to_vec(), "log", request.encode_to_vec(), source)?;
    Ok(LogsAppendResponse { receipt: Some(receipt) })
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(LogsAppendRequest, LogsAppendResponse, resources, "dokime.host.v1.evidence", "Logs", "Append");
